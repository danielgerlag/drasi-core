"""Tests for ApplicationReaction: stream results from queries."""

import asyncio

from .conftest import build_simple_lib, make_person_props


async def test_as_stream_returns_result_stream():
    """Calling as_stream on the reaction handle returns a ResultStream."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-rxn-1")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    await lib.stop()


async def test_stream_receives_data():
    """The result stream yields data after source pushes."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-rxn-2")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props = make_person_props("Eve")
    await handle.send_node_insert("e1", ["Person"], props)

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert "query_id" in result
    assert "results" in result
    assert "timestamp" in result
    assert isinstance(result["results"], list)

    await lib.stop()


async def test_stream_multiple_results():
    """Stream yields multiple results for multiple inserts."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-rxn-3")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    for i in range(3):
        props = make_person_props(f"User{i}")
        await handle.send_node_insert(f"u{i}", ["Person"], props)

    results = []
    for _ in range(3):
        r = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
        results.append(r)

    assert len(results) == 3
    for r in results:
        assert r["query_id"] == "test-query"

    await lib.stop()


async def test_reaction_handle_id():
    """The reaction handle reports the correct reaction_id."""
    lib, _, reaction_handle = await build_simple_lib(
        lib_id="app-rxn-4", reaction_id="my-reaction"
    )
    assert reaction_handle.reaction_id() == "my-reaction"
    await lib.stop()
