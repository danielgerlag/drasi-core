"""Crypto Order Book — Streamlit UI powered by Drasi continuous queries.

Data flow:
  User submits bid/ask for an asset → ApplicationSource → Drasi queries → Reactions → UI

Assets: BTC, ETH, SOL — seeded as Asset nodes on startup.  Each order gets a
[:FOR] relationship to its asset node.

Queries:
  open-bids      : Bids connected to an Asset via [:FOR] with status 'open'
  open-asks      : Asks connected to an Asset via [:FOR] with status 'open'
  all-trades     : All Trade nodes
  matched-orders : Cross-node Cypher query that finds Bid/Ask pairs connected
                   to the same Asset where bid.price >= ask.price — this is the
                   core matching engine, implemented entirely as a Drasi query

When the matched-orders reaction fires, a background worker executes the trade:
marks both orders as 'filled' and inserts a Trade node — all through the
ApplicationSource so the other queries react immediately.
"""

import asyncio
import threading
import time
import uuid
from datetime import datetime, timezone

import streamlit as st

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import (
    PyApplicationSource,
    PyPropertyMapBuilder,
)
from drasi_reaction_application import PyApplicationReaction


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

ASSETS = ["BTC", "ETH", "SOL"]
ASSET_NAMES = {"BTC": "Bitcoin", "ETH": "Ethereum", "SOL": "Solana"}
DEFAULT_PRICES = {"BTC": 50000.0, "ETH": 3000.0, "SOL": 150.0}


# ---------------------------------------------------------------------------
# Async helper — run coroutines on a dedicated background thread/loop
# to avoid conflicts with Streamlit's own event loop.
# ---------------------------------------------------------------------------

_bg_loop: asyncio.AbstractEventLoop | None = None
_bg_thread: threading.Thread | None = None


def _get_bg_loop() -> asyncio.AbstractEventLoop:
    global _bg_loop, _bg_thread
    if _bg_loop is None or _bg_loop.is_closed():
        _bg_loop = asyncio.new_event_loop()
        _bg_thread = threading.Thread(target=_bg_loop.run_forever, daemon=True)
        _bg_thread.start()
    return _bg_loop


def run_async(coro):
    loop = _get_bg_loop()
    return asyncio.run_coroutine_threadsafe(coro, loop).result()


def fire_and_forget(coro):
    asyncio.run_coroutine_threadsafe(coro, _get_bg_loop())


# ---------------------------------------------------------------------------
# Shared state — lives inside @st.cache_resource so it survives reruns
# ---------------------------------------------------------------------------

class OrderBookState:
    def __init__(self):
        self.lock = threading.Lock()
        self.open_bids: dict = {}   # id -> {id, price, quantity, asset, ...}
        self.open_asks: dict = {}
        self.trades: dict = {}      # id -> {price, quantity, asset, bid_id, ask_id, ...}
        self.remaining_qty: dict = {}  # order_id -> actual remaining qty (local ledger)
        self.filled_orders: set = set()  # order IDs that have been fully consumed


# ---------------------------------------------------------------------------
# Drasi setup
# ---------------------------------------------------------------------------

def _build_drasi():
    state = OrderBookState()

    source = PyApplicationSource("orderbook")
    source_handle = source.get_handle()

    def make_reaction(name, queries):
        b = PyApplicationReaction.builder(name)
        for q in queries:
            b.with_query(q)
        b.with_auto_start(True)
        return b.build()

    bids_rx, bids_h = make_reaction("bids-rx", ["open-bids"])
    asks_rx, asks_h = make_reaction("asks-rx", ["open-asks"])
    trades_rx, trades_h = make_reaction("trades-rx", ["all-trades"])
    match_rx, match_h = make_reaction("match-rx", ["matched-orders"])

    def cypher(qid, text):
        q = Query.cypher(qid)
        q.query(text)
        q.from_source("orderbook")
        q.auto_start(True)
        return q.build()

    q_bids = cypher(
        "open-bids",
        """
        MATCH (b:Bid)-[:FOR]->(a:Asset)
        WHERE b.status = 'open'
        RETURN b.id AS id,
               b.price AS price,
               b.quantity AS quantity,
               a.id AS asset,
               b.submitted_at AS submitted_at
        """,
    )
    q_asks = cypher(
        "open-asks",
        """
        MATCH (s:Ask)-[:FOR]->(a:Asset)
        WHERE s.status = 'open'
        RETURN s.id AS id,
               s.price AS price,
               s.quantity AS quantity,
               a.id AS asset,
               s.submitted_at AS submitted_at
        """,
    )
    q_trades = cypher(
        "all-trades",
        """
        MATCH (t:Trade)
        RETURN t.id AS id,
               t.price AS price,
               t.quantity AS quantity,
               t.asset AS asset,
               t.bid_id AS bid_id,
               t.ask_id AS ask_id,
               t.executed_at AS executed_at
        """,
    )
    # The matching query — finds Bid/Ask pairs on the same Asset
    # Computes trade_qty and remainders so the worker can do partial fills
    q_matched = cypher(
        "matched-orders",
        """
        MATCH (b:Bid)-[:FOR]->(a:Asset)<-[:FOR]-(s:Ask)
        WHERE b.status = 'open'
          AND s.status = 'open'
          AND b.price >= s.price
        RETURN b.id AS bid_id,
               b.price AS bid_price,
               b.quantity AS bid_qty,
               s.id AS ask_id,
               s.price AS ask_price,
               s.quantity AS ask_qty,
               a.id AS asset,
               CASE WHEN b.quantity <= s.quantity
                    THEN b.quantity
                    ELSE s.quantity
               END AS trade_qty,
               b.quantity - CASE WHEN b.quantity <= s.quantity
                                 THEN b.quantity
                                 ELSE s.quantity
                            END AS bid_remainder,
               s.quantity - CASE WHEN b.quantity <= s.quantity
                                 THEN b.quantity
                                 ELSE s.quantity
                            END AS ask_remainder
        """,
    )

    builder = DrasiLibBuilder()
    builder.with_id("crypto-orderbook")
    builder.with_source(source.into_source_wrapper())
    for qcfg in [q_bids, q_asks, q_trades, q_matched]:
        builder.with_query(qcfg)
    for r in [bids_rx, asks_rx, trades_rx, match_rx]:
        builder.with_reaction(r.into_reaction_wrapper())

    async def _init():
        lib = await builder.build()
        await lib.start()

        # Seed asset nodes
        for ticker in ASSETS:
            p = PyPropertyMapBuilder()
            p.with_string("id", ticker)
            p.with_string("name", ASSET_NAMES[ticker])
            await source_handle.send_node_insert(ticker, ["Asset"], p.build())

        bs = await bids_h.as_stream()
        aks = await asks_h.as_stream()
        ts = await trades_h.as_stream()
        ms = await match_h.as_stream()
        return lib, bs, aks, ts, ms

    lib, bids_stream, asks_stream, trades_stream, match_stream = run_async(_init())
    _start_workers(state, source_handle,
                   bids_stream, asks_stream, trades_stream, match_stream)

    return state, source_handle


def _start_workers(state, source_handle,
                   bids_stream, asks_stream, trades_stream, match_stream):

    async def _bids_worker():
        if bids_stream is None:
            return
        async for result in bids_stream:
            for r in result.get("results", []):
                data = r.get("data", {})
                rid = data.get("id", "")
                rtype = r.get("type", "")
                with state.lock:
                    if rtype in ("ADD", "UPDATE"):
                        state.open_bids[rid] = data
                        # Only seed the ledger for brand-new orders; never
                        # overwrite — the match worker owns qty adjustments.
                        if rid not in state.remaining_qty:
                            state.remaining_qty[rid] = float(data.get("quantity", 0))
                    elif rtype == "DELETE":
                        state.open_bids.pop(rid, None)
                        state.remaining_qty.pop(rid, None)

    async def _asks_worker():
        if asks_stream is None:
            return
        async for result in asks_stream:
            for r in result.get("results", []):
                data = r.get("data", {})
                rid = data.get("id", "")
                rtype = r.get("type", "")
                with state.lock:
                    if rtype in ("ADD", "UPDATE"):
                        state.open_asks[rid] = data
                        if rid not in state.remaining_qty:
                            state.remaining_qty[rid] = float(data.get("quantity", 0))
                    elif rtype == "DELETE":
                        state.open_asks.pop(rid, None)
                        state.remaining_qty.pop(rid, None)

    async def _trades_worker():
        if trades_stream is None:
            return
        async for result in trades_stream:
            for r in result.get("results", []):
                data = r.get("data", {})
                rid = data.get("id", "")
                rtype = r.get("type", "")
                with state.lock:
                    if rtype == "ADD":
                        state.trades[rid] = data

    async def _match_worker():
        """Executes trades when the matched-orders Drasi query fires.

        Processes both ADD and UPDATE events. When a partial fill reduces an
        order's quantity, the query engine emits UPDATEs for remaining matches.
        The local ledger (state.remaining_qty) prevents double-trading.
        """
        if match_stream is None:
            return
        async for result in match_stream:
            for r in result.get("results", []):
                if r.get("type") not in ("ADD", "UPDATE"):
                    continue
                data = r.get("data", {})
                bid_id = data.get("bid_id", "")
                ask_id = data.get("ask_id", "")
                bid_price = float(data.get("bid_price", 0))
                ask_price = float(data.get("ask_price", 0))
                asset = data.get("asset", "")

                # Determine actual trade qty from the local ledger.
                # If an order isn't in the ledger yet (race: match fires before
                # the bids/asks worker populates it), seed it from the query data.
                # Never re-seed an order that was already fully consumed.
                with state.lock:
                    if bid_id in state.filled_orders or ask_id in state.filled_orders:
                        continue

                    if bid_id not in state.remaining_qty:
                        state.remaining_qty[bid_id] = float(data.get("bid_qty", 0))
                    if ask_id not in state.remaining_qty:
                        state.remaining_qty[ask_id] = float(data.get("ask_qty", 0))

                    actual_bid_qty = state.remaining_qty[bid_id]
                    actual_ask_qty = state.remaining_qty[ask_id]

                    if actual_bid_qty <= 0 or actual_ask_qty <= 0:
                        continue  # already fully consumed by a prior match

                    trade_qty = min(actual_bid_qty, actual_ask_qty)
                    bid_new_qty = actual_bid_qty - trade_qty
                    ask_new_qty = actual_ask_qty - trade_qty

                    # Debit the ledger immediately so concurrent matches see
                    # the reduced quantities.
                    state.remaining_qty[bid_id] = bid_new_qty
                    state.remaining_qty[ask_id] = ask_new_qty

                    if bid_new_qty <= 0:
                        state.filled_orders.add(bid_id)
                    if ask_new_qty <= 0:
                        state.filled_orders.add(ask_id)

                trade_id = str(uuid.uuid4())[:8]
                now = datetime.now(timezone.utc).isoformat()

                # Update bid: filled if fully consumed, else reduce quantity
                bp = PyPropertyMapBuilder()
                bp.with_string("id", bid_id)
                bp.with_float("price", bid_price)
                if bid_new_qty <= 0:
                    bp.with_float("quantity", trade_qty)
                    bp.with_string("status", "filled")
                else:
                    bp.with_float("quantity", bid_new_qty)
                    bp.with_string("status", "open")
                await source_handle.send_node_update(bid_id, ["Bid"], bp.build())

                # Update ask: filled if fully consumed, else reduce quantity
                ap = PyPropertyMapBuilder()
                ap.with_string("id", ask_id)
                ap.with_float("price", ask_price)
                if ask_new_qty <= 0:
                    ap.with_float("quantity", trade_qty)
                    ap.with_string("status", "filled")
                else:
                    ap.with_float("quantity", ask_new_qty)
                    ap.with_string("status", "open")
                await source_handle.send_node_update(ask_id, ["Ask"], ap.build())

                # Insert trade node
                tp = PyPropertyMapBuilder()
                tp.with_string("id", trade_id)
                tp.with_float("price", ask_price)
                tp.with_float("quantity", trade_qty)
                tp.with_string("asset", asset)
                tp.with_string("bid_id", bid_id)
                tp.with_string("ask_id", ask_id)
                tp.with_string("executed_at", now)
                await source_handle.send_node_insert(trade_id, ["Trade"], tp.build())

    fire_and_forget(_bids_worker())
    fire_and_forget(_asks_worker())
    fire_and_forget(_trades_worker())
    fire_and_forget(_match_worker())


@st.cache_resource
def get_drasi():
    return _build_drasi()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def now_iso():
    return datetime.now(timezone.utc).isoformat()


def submit_order(handle, side: str, asset: str, price: float, quantity: float):
    """Insert order node + [:FOR] relationship to the asset."""
    oid = str(uuid.uuid4())[:8]
    label = "Bid" if side == "BID" else "Ask"
    rel_id = f"{oid}-for"

    props = PyPropertyMapBuilder()
    props.with_string("id", oid)
    props.with_float("price", price)
    props.with_float("quantity", quantity)
    props.with_string("status", "open")
    props.with_string("submitted_at", now_iso())

    rel_props = PyPropertyMapBuilder()
    rel_props.with_string("id", rel_id)

    async def _send():
        await handle.send_node_insert(oid, [label], props.build())
        await handle.send_relation_insert(
            rel_id, ["FOR"], rel_props.build(), oid, asset,
        )

    run_async(_send())
    return oid


# ---------------------------------------------------------------------------
# Main UI
# ---------------------------------------------------------------------------

st.set_page_config(page_title="Crypto Order Book", layout="wide")
st.title("₿ Crypto Order Book")
st.caption("Powered by Drasi continuous queries")

state, source_handle = get_drasi()

# ── Asset picker ───────────────────────────────────────────────────────────
selected_asset = st.selectbox(
    "Asset",
    ASSETS,
    format_func=lambda a: f"{a} — {ASSET_NAMES[a]}",
)

# ── Order entry ────────────────────────────────────────────────────────────
st.subheader(f"Submit {selected_asset} Order")
col1, col2, col3 = st.columns([1, 1, 1])
with col1:
    side = st.selectbox("Side", ["BID", "ASK"])
with col2:
    price = st.number_input(
        "Price (USD)",
        min_value=0.01,
        value=DEFAULT_PRICES[selected_asset],
        step=DEFAULT_PRICES[selected_asset] / 100,
    )
with col3:
    quantity = st.number_input("Quantity", min_value=0.001, value=1.0, step=0.1)

if st.button("Submit Order", type="primary", use_container_width=True):
    oid = submit_order(source_handle, side, selected_asset, price, quantity)
    st.toast(f"{side} {selected_asset} submitted: {oid}")
    time.sleep(0.5)

# ── Snapshot state filtered by selected asset ──────────────────────────────
with state.lock:
    all_bids = list(state.open_bids.values())
    all_asks = list(state.open_asks.values())
    all_trades = list(state.trades.values())

asset_bids = [b for b in all_bids if b.get("asset") == selected_asset]
asset_asks = [a for a in all_asks if a.get("asset") == selected_asset]
asset_trades = [t for t in all_trades if t.get("asset") == selected_asset]

# ── Order Book ─────────────────────────────────────────────────────────────
st.subheader(f"{selected_asset} Order Book")
bid_col, ask_col = st.columns(2)

with bid_col:
    st.markdown("**🟢 Bids (Buy)**")
    bids = sorted(asset_bids, key=lambda x: float(x.get("price", 0)), reverse=True)
    if bids:
        st.table([{"Price": b["price"], "Qty": b["quantity"], "ID": b["id"]} for b in bids])
    else:
        st.info("No open bids")

with ask_col:
    st.markdown("**🔴 Asks (Sell)**")
    asks = sorted(asset_asks, key=lambda x: float(x.get("price", 0)))
    if asks:
        st.table([{"Price": a["price"], "Qty": a["quantity"], "ID": a["id"]} for a in asks])
    else:
        st.info("No open asks")

# ── Trades ─────────────────────────────────────────────────────────────────
st.subheader(f"{selected_asset} Trade History")
trades_list = sorted(asset_trades, key=lambda x: x.get("executed_at", ""), reverse=True)
if trades_list:
    st.table([
        {
            "Time": t.get("executed_at", "")[:19],
            "Price": t["price"],
            "Qty": t["quantity"],
            "Bid": t["bid_id"],
            "Ask": t["ask_id"],
        }
        for t in trades_list
    ])
else:
    st.info(f"No {selected_asset} trades yet — submit matching bids and asks!")

# ── Summary across all assets ──────────────────────────────────────────────
with st.expander("All Assets Summary"):
    for asset in ASSETS:
        ab = [b for b in all_bids if b.get("asset") == asset]
        aa = [a for a in all_asks if a.get("asset") == asset]
        at = [t for t in all_trades if t.get("asset") == asset]
        st.markdown(f"**{asset}**: {len(ab)} bids, {len(aa)} asks, {len(at)} trades")

st.caption("Click 'Submit Order' or press R to refresh.")
