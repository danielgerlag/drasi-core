"""Tests for Channel vs Broadcast dispatch modes."""

import asyncio

from drasi_core import DispatchMode
from drasi_lib import DrasiLibBuilder, Query
from drasi_reaction_application import PyApplicationReaction
from drasi_source_application import PyApplicationSource

from .conftest import make_person_props


async def _build_with_dispatch_mode(
    lib_id: str, dispatch_mode: DispatchMode
) -> tuple:
    """Helper to build a lib with a specific dispatch mode on the query."""
    source = PyApplicationSource(f"{lib_id}-source")
    handle = source.get_handle()

    reaction_builder = PyApplicationReaction.builder(f"{lib_id}-reaction")
    reaction_builder.with_query(f"{lib_id}-query")
    reaction, reaction_handle = reaction_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id(lib_id)
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher(f"{lib_id}-query")
    q.query("MATCH (n:Person) RETURN n.name")
    q.from_source(f"{lib_id}-source")
    q.auto_start(True)
    q.with_dispatch_mode(dispatch_mode)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    return lib, handle, reaction_handle


async def test_channel_dispatch_mode():
    """Channel dispatch mode delivers results without loss."""
    lib, handle, reaction_handle = await _build_with_dispatch_mode(
        "dispatch-ch", DispatchMode.Channel
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props)

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["query_id"] == "dispatch-ch-query"

    await lib.stop()


async def test_broadcast_dispatch_mode():
    """Broadcast dispatch mode delivers results (may drop under load)."""
    lib, handle, reaction_handle = await _build_with_dispatch_mode(
        "dispatch-bc", DispatchMode.Broadcast
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props = make_person_props("Bob")
    await handle.send_node_insert("n1", ["Person"], props)

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["query_id"] == "dispatch-bc-query"

    await lib.stop()


async def test_dispatch_mode_enum_values():
    """DispatchMode enum values are accessible."""
    assert DispatchMode.Channel is not None
    assert DispatchMode.Broadcast is not None
    assert DispatchMode.Channel != DispatchMode.Broadcast
