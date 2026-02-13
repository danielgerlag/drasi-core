"""Tests for ApplicationSource: push nodes and relations through the source handle."""

import asyncio

from drasi_source_application import PropertyMapBuilder

from .conftest import build_simple_lib, make_person_props


async def test_push_node_insert():
    """Insert a node via the source handle and verify the reaction receives it."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-src-1")
    await lib.start()
    await asyncio.sleep(0.1)

    props = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["query_id"] == "test-query"
    assert len(result["results"]) > 0

    await lib.stop()


async def test_push_node_update():
    """Insert then update a node."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-src-2")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props1 = make_person_props("Bob")
    await handle.send_node_insert("n2", ["Person"], props1)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["results"][0]["type"] == "ADD"

    props2 = make_person_props("Bobby")
    await handle.send_node_update("n2", ["Person"], props2)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["results"][0]["type"] == "UPDATE"

    await lib.stop()


async def test_push_node_delete():
    """Insert then delete a node."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="app-src-3")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props = make_person_props("Charlie")
    await handle.send_node_insert("n3", ["Person"], props)
    await asyncio.wait_for(stream.__anext__(), timeout=5.0)

    await handle.send_delete("n3", ["Person"])
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["results"][0]["type"] == "DELETE"

    await lib.stop()


async def test_push_relation_insert():
    """Insert two nodes and a relation between them."""
    cypher = (
        "MATCH (a:Person)-[r:KNOWS]->(b:Person) "
        "RETURN a.name, b.name"
    )
    lib, handle, reaction_handle = await build_simple_lib(
        lib_id="app-src-4",
        cypher=cypher,
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props_a = make_person_props("Alice")
    await handle.send_node_insert("a1", ["Person"], props_a)

    props_b = make_person_props("Bob")
    await handle.send_node_insert("b1", ["Person"], props_b)

    rel_props = PropertyMapBuilder()
    rel_props.with_string("since", "2024")
    await handle.send_relation_insert("r1", ["KNOWS"], rel_props.build(), "a1", "b1")

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert result["query_id"] == "test-query"
    assert len(result["results"]) > 0

    await lib.stop()


async def test_source_handle_source_id():
    """The handle reports the correct source_id."""
    source_id = "my-source"
    lib, handle, _ = await build_simple_lib(lib_id="app-src-5", source_id=source_id)
    assert handle.source_id() == source_id
    await lib.stop()
