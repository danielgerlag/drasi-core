"""Pac-Man game powered by Drasi continuous queries (Web UI).

Flask serves the HTML5 Canvas game.  The Drasi SSE Reaction streams
query results directly to the browser.  Game simulation (ghost AI,
movement) runs server-side and pushes state updates to Drasi, which
evaluates collision queries and delivers the results over SSE.

Usage:
    cd python/
    uv run python examples/pacman-game/game_web.py

Then open http://localhost:5050 in a browser.
"""

import asyncio
import json
import os
import random
import threading
import time

from flask import Flask, jsonify, request, Response

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import ApplicationSource, PropertyMapBuilder
from drasi_source_http import (
    HttpSource,
    HttpSourceConfig,
    WebhookConfig,
    CorsConfig,
    WebhookRoute,
    WebhookMapping,
    MappingCondition,
    ElementTemplate,
)
from drasi_reaction_sse import SseReaction

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

COLS, ROWS = 21, 21
FPS = 4
SSE_PORT = 8082
INPUT_PORT = 8083
FLASK_PORT = 5050

MAZE = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,0,1],
    [1,2,1,1,0,1,1,1,0,0,0,0,0,1,1,1,0,1,1,2,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,0,1],
    [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],
    [1,1,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,1,1,0,0,0,1,1,0,1,0,1,1,1,1],
    [0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0],
    [1,1,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,0,1],
    [1,2,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,2,1],
    [1,1,0,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,0,1,1],
    [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],
    [1,0,1,1,1,1,1,1,0,0,1,0,0,1,1,1,1,1,1,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
]

GHOST_START = {
    "blinky": (10, 9),
    "pinky":  (9, 10),
    "inky":   (10, 10),
    "clyde":  (11, 10),
}
PACMAN_START = (10, 16)
POWER_DURATION = 40  # ticks


# ---------------------------------------------------------------------------
# Async helper
# ---------------------------------------------------------------------------

_bg_loop: asyncio.AbstractEventLoop | None = None
_bg_thread: threading.Thread | None = None


def _get_bg_loop():
    global _bg_loop, _bg_thread
    if _bg_loop is None or _bg_loop.is_closed():
        _bg_loop = asyncio.new_event_loop()
        _bg_thread = threading.Thread(target=_bg_loop.run_forever, daemon=True)
        _bg_thread.start()
    return _bg_loop


def run_async(coro):
    loop = _get_bg_loop()
    fut = asyncio.run_coroutine_threadsafe(coro, loop)
    return fut.result(timeout=30)


def fire_and_forget(coro):
    asyncio.run_coroutine_threadsafe(coro, _get_bg_loop())


# ---------------------------------------------------------------------------
# Game state
# ---------------------------------------------------------------------------

class GameState:
    def __init__(self, handle):
        self.handle = handle
        self.pacman_pos = list(PACMAN_START)
        self.pacman_dir = (0, 0)
        self.next_dir = (0, 0)
        self.powered = False
        self.power_timer = 0
        self.lives = 3
        self.score = 0
        self.game_over = False
        self.win = False
        self.ghosts = {}
        for name, pos in GHOST_START.items():
            self.ghosts[name] = {
                "pos": list(pos),
                "alive": True,
                "dir": random.choice([(0, -1), (0, 1), (-1, 0), (1, 0)]),
                "respawn_timer": 0,
            }
        self.dots = set()
        self.pellets = set()
        for r in range(ROWS):
            for c in range(COLS):
                if MAZE[r][c] == 0:
                    self.dots.add((c, r))
                elif MAZE[r][c] == 2:
                    self.pellets.add((c, r))
        self.total_dots = len(self.dots) + len(self.pellets)
        self.events = []
        self.event_lock = threading.Lock()
        self.lock = threading.Lock()
        # Track current cell for relationship management
        self._pacman_cell = None
        self._ghost_cells = {}

    def push_event(self, etype, data=None):
        with self.event_lock:
            self.events.append((etype, data or {}))

    def pop_events(self):
        with self.event_lock:
            evts = list(self.events)
            self.events.clear()
        return evts

    def snapshot(self):
        """Return JSON-serialisable snapshot for the browser."""
        with self.lock:
            return {
                "pacman": {"x": self.pacman_pos[0], "y": self.pacman_pos[1],
                           "powered": self.powered},
                "ghosts": {
                    n: {"x": g["pos"][0], "y": g["pos"][1],
                        "alive": g["alive"]}
                    for n, g in self.ghosts.items()
                },
                "dots": list(self.dots),
                "pellets": list(self.pellets),
                "score": self.score,
                "lives": self.lives,
                "power_timer": self.power_timer,
                "game_over": self.game_over,
                "win": self.win,
            }


def can_move(x, y):
    nx = x % COLS
    ny = y % ROWS
    if 0 <= ny < ROWS and 0 <= nx < COLS:
        return MAZE[ny][nx] != 1
    return False


# ---------------------------------------------------------------------------
# Send state to Drasi
# ---------------------------------------------------------------------------

def cell_id(x, y):
    return f"cell-{x}-{y}"


def rel_id(entity_id, label):
    return f"{entity_id}-{label}"


def send_pacman_state(gs: GameState):
    new_cell = cell_id(gs.pacman_pos[0], gs.pacman_pos[1])
    old_cell = gs._pacman_cell

    async def _send():
        # Update Pacman node properties
        p = PropertyMapBuilder()
        p.with_string("id", "pacman")
        p.with_string("powered", str(gs.powered).lower())
        await gs.handle.send_node_update("pacman", ["Pacman"], p.build())

        # Move IS_AT relationship if cell changed
        if new_cell != old_cell:
            rid = rel_id("pacman", "IS_AT")
            if old_cell is not None:
                await gs.handle.send_delete(rid, ["IS_AT"])
            rp = PropertyMapBuilder()
            rp.with_string("id", rid)
            await gs.handle.send_relation_insert(
                rid, ["IS_AT"], rp.build(), "pacman", new_cell
            )
            gs._pacman_cell = new_cell

    fire_and_forget(_send())


def send_ghost_state(gs: GameState, name: str):
    g = gs.ghosts[name]
    new_cell = cell_id(g["pos"][0], g["pos"][1])
    old_cell = gs._ghost_cells.get(name)

    async def _send():
        # Update Ghost node properties
        p = PropertyMapBuilder()
        p.with_string("id", name)
        p.with_string("alive", str(g["alive"]).lower())
        await gs.handle.send_node_update(name, ["Ghost"], p.build())

        # Move IS_AT relationship if cell changed
        if new_cell != old_cell:
            rid = rel_id(name, "IS_AT")
            if old_cell is not None:
                await gs.handle.send_delete(rid, ["IS_AT"])
            rp = PropertyMapBuilder()
            rp.with_string("id", rid)
            await gs.handle.send_relation_insert(
                rid, ["IS_AT"], rp.build(), name, new_cell
            )
            gs._ghost_cells[name] = new_cell

    fire_and_forget(_send())


def send_dot_eaten(gs: GameState, x, y):
    dot_id = f"dot-{x}-{y}"
    p = PropertyMapBuilder()
    p.with_string("id", dot_id)
    p.with_integer("x", x)
    p.with_integer("y", y)
    p.with_string("eaten", "true")

    async def _send():
        await gs.handle.send_node_update(dot_id, ["Dot"], p.build())
    fire_and_forget(_send())


def send_pellet_eaten(gs: GameState, x, y):
    """Remove the pellet's ON relationship and delete the Pellet node."""
    pid = f"pellet-{x}-{y}"

    async def _send():
        await gs.handle.send_delete(rel_id(pid, "ON"), ["ON"])
        await gs.handle.send_delete(pid, ["Pellet"])

    fire_and_forget(_send())


def init_drasi_state(gs: GameState):
    async def _init():
        # Create Cell nodes for every walkable cell
        for r in range(ROWS):
            for c in range(COLS):
                if MAZE[r][c] != 1:
                    cid = cell_id(c, r)
                    cp = PropertyMapBuilder()
                    cp.with_string("id", cid)
                    cp.with_integer("x", c)
                    cp.with_integer("y", r)
                    await gs.handle.send_node_insert(cid, ["Cell"], cp.build())

        # Create Pacman node
        p = PropertyMapBuilder()
        p.with_string("id", "pacman")
        p.with_string("powered", "false")
        await gs.handle.send_node_insert("pacman", ["Pacman"], p.build())

        # Pacman IS_AT relationship
        pac_cell = cell_id(*PACMAN_START)
        rid = rel_id("pacman", "IS_AT")
        rp = PropertyMapBuilder()
        rp.with_string("id", rid)
        await gs.handle.send_relation_insert(
            rid, ["IS_AT"], rp.build(), "pacman", pac_cell
        )
        gs._pacman_cell = pac_cell

        # Create Ghost nodes + IS_AT relationships
        for name, info in gs.ghosts.items():
            g = PropertyMapBuilder()
            g.with_string("id", name)
            g.with_string("alive", "true")
            await gs.handle.send_node_insert(name, ["Ghost"], g.build())

            gc = cell_id(info["pos"][0], info["pos"][1])
            rid = rel_id(name, "IS_AT")
            rp = PropertyMapBuilder()
            rp.with_string("id", rid)
            await gs.handle.send_relation_insert(
                rid, ["IS_AT"], rp.build(), name, gc
            )
            gs._ghost_cells[name] = gc

        # Create Pellet nodes + ON relationships to cells
        for (x, y) in gs.pellets:
            pid = f"pellet-{x}-{y}"
            pp = PropertyMapBuilder()
            pp.with_string("id", pid)
            await gs.handle.send_node_insert(pid, ["Pellet"], pp.build())

            rid = rel_id(pid, "ON")
            rp = PropertyMapBuilder()
            rp.with_string("id", rid)
            await gs.handle.send_relation_insert(
                rid, ["ON"], rp.build(), pid, cell_id(x, y)
            )

        # Create Dot nodes (simple tracking, no relationships needed)
        for (x, y) in gs.dots:
            d = PropertyMapBuilder()
            dot_id = f"dot-{x}-{y}"
            d.with_string("id", dot_id)
            d.with_integer("x", x)
            d.with_integer("y", y)
            d.with_string("eaten", "false")
            await gs.handle.send_node_insert(dot_id, ["Dot"], d.build())

    run_async(_init())


# ---------------------------------------------------------------------------
# Ghost AI
# ---------------------------------------------------------------------------

def move_ghost(gs, name):
    g = gs.ghosts[name]
    if not g["alive"]:
        g["respawn_timer"] -= 1
        if g["respawn_timer"] <= 0:
            g["pos"] = list(GHOST_START[name])
            g["alive"] = True
        return

    px, py = gs.pacman_pos
    gx, gy = g["pos"]
    dx, dy = g["dir"]

    reverse = (-dx, -dy)
    choices = []
    for d in [(0, -1), (0, 1), (-1, 0), (1, 0)]:
        if d == reverse:
            continue
        nx, ny = (gx + d[0]) % COLS, (gy + d[1]) % ROWS
        if can_move(nx, ny):
            choices.append(d)

    if not choices:
        choices = (
            [reverse]
            if can_move((gx + reverse[0]) % COLS, (gy + reverse[1]) % ROWS)
            else [(0, 0)]
        )

    if gs.powered:
        def away_dist(d):
            nx, ny = gx + d[0], gy + d[1]
            return -((nx - px) ** 2 + (ny - py) ** 2)
        choices.sort(key=away_dist)
    else:
        if random.random() < 0.3:
            random.shuffle(choices)
        else:
            def chase_dist(d):
                nx, ny = gx + d[0], gy + d[1]
                return (nx - px) ** 2 + (ny - py) ** 2
            choices.sort(key=chase_dist)

    g["dir"] = choices[0]
    nx = (gx + g["dir"][0]) % COLS
    ny = (gy + g["dir"][1]) % ROWS
    g["pos"] = [nx, ny]


# ---------------------------------------------------------------------------
# Drasi wiring
# ---------------------------------------------------------------------------

from drasi_reaction_application import ApplicationReaction

# Webhook config for the HTTP source: the browser POSTs {dir: "up/down/left/right"}
# to /input and the webhook maps it to an insert of a Direction node with dx/dy.
# Using "insert" rather than "update" so that the first keypress creates the node;
# subsequent inserts act as upserts since drasi-core replaces existing nodes.
INPUT_WEBHOOK_CONFIG = HttpSourceConfig(
    "0.0.0.0", INPUT_PORT,
    webhooks=WebhookConfig(
        [
            WebhookRoute("/input", [
                WebhookMapping(
                    "node",
                    ElementTemplate("direction", ["Direction"], {"dx": 0, "dy": -1}),
                    when=MappingCondition(field="payload.dir", equals="up"),
                    operation="insert",
                ),
                WebhookMapping(
                    "node",
                    ElementTemplate("direction", ["Direction"], {"dx": 0, "dy": 1}),
                    when=MappingCondition(field="payload.dir", equals="down"),
                    operation="insert",
                ),
                WebhookMapping(
                    "node",
                    ElementTemplate("direction", ["Direction"], {"dx": -1, "dy": 0}),
                    when=MappingCondition(field="payload.dir", equals="left"),
                    operation="insert",
                ),
                WebhookMapping(
                    "node",
                    ElementTemplate("direction", ["Direction"], {"dx": 1, "dy": 0}),
                    when=MappingCondition(field="payload.dir", equals="right"),
                    operation="insert",
                ),
            ]),
        ],
        error_behavior="accept_and_log",
        cors=CorsConfig(allow_origins=["*"]),
    ),
)


def build_drasi_full():
    """Build Drasi with:
      - ApplicationSource for game state (ghosts, pacman, cells, dots, pellets)
      - HttpSource (webhook) for player input (direction changes)
      - SSE Reaction → browser (collision/pellet events)
      - ApplicationReactions → server-side game logic
    """
    # --- Sources ---
    game_source = ApplicationSource("game")
    handle = game_source.get_handle()

    input_builder = HttpSource.builder("input")
    input_builder.with_config(INPUT_WEBHOOK_CONFIG)
    input_builder.with_auto_start(True)
    input_source = input_builder.build()

    # --- SSE reaction → browser ---
    sse_builder = SseReaction.builder("game-sse")
    sse_builder.with_port(SSE_PORT)
    sse_builder.with_sse_path("/events")
    sse_builder.with_queries([
        "ghost-catches-pacman",
        "pacman-eats-pellet",
        "pacman-catches-ghost",
        "dots-eaten",
    ])
    sse_builder.with_auto_start(True)
    sse_reaction = sse_builder.build()

    # --- ApplicationReactions → server-side game logic ---
    def make_rx(name, queries):
        b = ApplicationReaction.builder(name)
        for q in queries:
            b.with_query(q)
        b.with_auto_start(True)
        return b.build()

    ghost_catch_rx, ghost_catch_h = make_rx("ghost-catch-rx", ["ghost-catches-pacman"])
    pellet_rx, pellet_h = make_rx("pellet-rx", ["pacman-eats-pellet"])
    pacman_catch_rx, pacman_catch_h = make_rx("pacman-catch-rx", ["pacman-catches-ghost"])
    dots_rx, dots_h = make_rx("dots-rx", ["dots-eaten"])
    dir_rx, dir_h = make_rx("dir-rx", ["direction-changed"])

    # --- Queries on game source ---
    def game_cypher(qid, text):
        q = Query.cypher(qid)
        q.query(text)
        q.from_source("game")
        q.auto_start(True)
        return q.build()

    q_ghost_catch = game_cypher(
        "ghost-catches-pacman",
        """
            MATCH (g:Ghost)-[:IS_AT]->(c:Cell)<-[:IS_AT]-(p:Pacman)
            WHERE p.powered = 'false'
              AND g.alive = 'true'
            RETURN g.id AS ghost_id,
                   c.x  AS x,
                   c.y  AS y
        """,
    )
    q_pellet = game_cypher(
        "pacman-eats-pellet",
        """
            MATCH (p:Pacman)-[:IS_AT]->(c:Cell)<-[:ON]-(pel:Pellet)
            RETURN pel.id AS pellet_id,
                   c.x    AS x,
                   c.y    AS y
        """,
    )
    q_pacman_catch = game_cypher(
        "pacman-catches-ghost",
        """
            MATCH (g:Ghost)-[:IS_AT]->(c:Cell)<-[:IS_AT]-(p:Pacman)
            WHERE p.powered = 'true'
              AND g.alive = 'true'
            RETURN g.id AS ghost_id,
                   c.x  AS x,
                   c.y  AS y
        """,
    )
    q_dots = game_cypher(
        "dots-eaten",
        """
            MATCH (d:Dot)
            WHERE d.eaten = 'true'
            RETURN d.id AS dot_id
        """,
    )

    # --- Query on input source ---
    q_dir = Query.cypher("direction-changed")
    q_dir.query("""
        MATCH (d:Direction)
        RETURN d.dx AS dx,
               d.dy AS dy
    """)
    q_dir.from_source("input")
    q_dir.auto_start(True)
    q_dir_cfg = q_dir.build()

    # --- Assemble ---
    builder = DrasiLibBuilder()
    builder.with_id("pacman")
    builder.with_source(game_source.into_source_wrapper())
    builder.with_source(input_source.into_source_wrapper())
    for qcfg in [q_ghost_catch, q_pellet, q_pacman_catch, q_dots, q_dir_cfg]:
        builder.with_query(qcfg)
    builder.with_reaction(sse_reaction.into_reaction_wrapper())
    for r in [ghost_catch_rx, pellet_rx, pacman_catch_rx, dots_rx, dir_rx]:
        builder.with_reaction(r.into_reaction_wrapper())

    async def _init():
        lib = await builder.build()
        await lib.start()
        s1 = await ghost_catch_h.as_stream()
        s2 = await pellet_h.as_stream()
        s3 = await pacman_catch_h.as_stream()
        s4 = await dots_h.as_stream()
        s5 = await dir_h.as_stream()
        return lib, s1, s2, s3, s4, s5

    lib, s_ghost, s_pellet, s_pacman, s_dots, s_dir = run_async(_init())
    return lib, handle, s_ghost, s_pellet, s_pacman, s_dots, s_dir

def game_tick(gs: GameState):
    """Advance one game tick: move pacman, move ghosts, send state."""
    with gs.lock:
        if gs.game_over:
            return

        # Move Pacman
        nx = (gs.pacman_pos[0] + gs.next_dir[0]) % COLS
        ny = (gs.pacman_pos[1] + gs.next_dir[1]) % ROWS
        if can_move(nx, ny):
            gs.pacman_dir = gs.next_dir

        nx = (gs.pacman_pos[0] + gs.pacman_dir[0]) % COLS
        ny = (gs.pacman_pos[1] + gs.pacman_dir[1]) % ROWS
        if can_move(nx, ny):
            gs.pacman_pos = [nx, ny]

        # Eat dots
        pos_tuple = tuple(gs.pacman_pos)
        if pos_tuple in gs.dots:
            gs.dots.discard(pos_tuple)
            gs.score += 10
            send_dot_eaten(gs, pos_tuple[0], pos_tuple[1])

        # Power timer
        if gs.powered:
            gs.power_timer -= 1
            if gs.power_timer <= 0:
                gs.powered = False

        # Move ghosts
        for name in gs.ghosts:
            move_ghost(gs, name)

        # Send state to Drasi
        send_pacman_state(gs)
        for name in gs.ghosts:
            send_ghost_state(gs, name)

        # Process Drasi events (delivered via SSE to browser, but we also
        # need them server-side for game-state mutations like lives/score)
        for etype, data in gs.pop_events():
            if etype == "ghost_catches_pacman" and not gs.powered:
                gs.lives -= 1
                if gs.lives <= 0:
                    gs.game_over = True
                else:
                    gs.pacman_pos = list(PACMAN_START)
                    gs.pacman_dir = (0, 0)
                    gs.next_dir = (0, 0)
                    for n, pos in GHOST_START.items():
                        gs.ghosts[n]["pos"] = list(pos)
                        gs.ghosts[n]["alive"] = True

            elif etype == "pacman_eats_pellet":
                pt = tuple(gs.pacman_pos)
                if pt in gs.pellets:
                    gs.pellets.discard(pt)
                    gs.score += 50
                    gs.powered = True
                    gs.power_timer = POWER_DURATION
                    send_pellet_eaten(gs, pt[0], pt[1])

            elif etype == "pacman_catches_ghost" and gs.powered:
                ghost_id = data.get("ghost_id", "")
                if ghost_id in gs.ghosts and gs.ghosts[ghost_id]["alive"]:
                    gs.ghosts[ghost_id]["alive"] = False
                    gs.ghosts[ghost_id]["respawn_timer"] = 20
                    gs.score += 200

        # Win condition
        if not gs.dots and not gs.pellets:
            gs.game_over = True
            gs.win = True


def start_tick_loop(gs: GameState):
    def _loop():
        while True:
            game_tick(gs)
            broadcast_state(gs.snapshot())
            time.sleep(1.0 / FPS)
    t = threading.Thread(target=_loop, daemon=True)
    t.start()


def start_reaction_workers(gs, s_ghost, s_pellet, s_pacman, s_dots, s_dir):
    """Consume ApplicationReaction streams and mutate game state."""
    async def _ghost_worker():
        if s_ghost is None:
            return
        async for result in s_ghost:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    gs.push_event("ghost_catches_pacman", r.get("data", {}))

    async def _pellet_worker():
        if s_pellet is None:
            return
        async for result in s_pellet:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    gs.push_event("pacman_eats_pellet", r.get("data", {}))

    async def _pacman_worker():
        if s_pacman is None:
            return
        async for result in s_pacman:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    gs.push_event("pacman_catches_ghost", r.get("data", {}))

    async def _dots_worker():
        if s_dots is None:
            return
        async for result in s_dots:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    gs.push_event("dot_eaten", r.get("data", {}))

    async def _dir_worker():
        """Direction changes arrive from the HTTP source webhook."""
        if s_dir is None:
            return
        async for result in s_dir:
            for r in result.get("results", []):
                data = r.get("data", {})
                dx = data.get("dx")
                dy = data.get("dy")
                if dx is not None and dy is not None:
                    with gs.lock:
                        gs.next_dir = (int(dx), int(dy))

    fire_and_forget(_ghost_worker())
    fire_and_forget(_pellet_worker())
    fire_and_forget(_pacman_worker())
    fire_and_forget(_dots_worker())
    fire_and_forget(_dir_worker())


# ---------------------------------------------------------------------------
# Flask app
# ---------------------------------------------------------------------------

import queue

# Subscribers for game state SSE stream
_state_subscribers: list[queue.Queue] = []
_sub_lock = threading.Lock()


def broadcast_state(snapshot):
    """Push a state snapshot to all connected SSE clients."""
    data = json.dumps(snapshot, separators=(",", ":"))
    dead = []
    with _sub_lock:
        for q in _state_subscribers:
            try:
                # Non-blocking; drop if client is slow
                q.put_nowait(data)
            except queue.Full:
                dead.append(q)
        for q in dead:
            _state_subscribers.remove(q)


app = Flask(__name__)


_STATIC_DIR = os.path.dirname(os.path.abspath(__file__))


@app.route("/")
def index():
    with open(os.path.join(_STATIC_DIR, "index.html")) as f:
        return f.read()


@app.route("/stream")
def stream():
    """SSE endpoint that pushes game state every tick."""
    q = queue.Queue(maxsize=4)
    with _sub_lock:
        _state_subscribers.append(q)

    def generate():
        try:
            # Send initial state immediately
            yield f"data: {json.dumps(gs.snapshot(), separators=(',', ':'))}\n\n"
            while True:
                try:
                    data = q.get(timeout=5)
                    yield f"data: {data}\n\n"
                except queue.Empty:
                    # Heartbeat to keep connection alive
                    yield ": heartbeat\n\n"
        finally:
            with _sub_lock:
                if q in _state_subscribers:
                    _state_subscribers.remove(q)

    from flask import Response
    return Response(generate(), mimetype="text/event-stream",
                    headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"})


@app.route("/restart", methods=["POST"])
def restart():
    global gs
    with gs.lock:
        if gs.game_over:
            gs.__init__(gs.handle)
            init_drasi_state(gs)
    return jsonify(ok=True)


@app.route("/config")
def config():
    return jsonify(
        maze=MAZE, cols=COLS, rows=ROWS,
        sse_port=SSE_PORT, input_port=INPUT_PORT, fps=FPS,
    )


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("Initializing Drasi engine...")
    lib, handle, s_ghost, s_pellet, s_pacman, s_dots, s_dir = build_drasi_full()

    gs = GameState(handle)
    init_drasi_state(gs)
    start_reaction_workers(gs, s_ghost, s_pellet, s_pacman, s_dots, s_dir)
    start_tick_loop(gs)

    print(f"SSE endpoint: http://localhost:{SSE_PORT}/events")
    print(f"Game UI:      http://localhost:{FLASK_PORT}")
    print(f"Open the URL above in your browser to play!")
    app.run(host="0.0.0.0", port=FLASK_PORT, debug=False, use_reloader=False)
