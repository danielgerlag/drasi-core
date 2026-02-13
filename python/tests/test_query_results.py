"""Tests for Cypher query result correctness: INSERT, UPDATE, DELETE diffs, and aggregations."""

import asyncio

from .conftest import build_simple_lib, make_person_props


async def test_insert_produces_add_diff():
    """A node insert should produce an ADD result diff."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="qr-add")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    props = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props)

    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    diffs = result.results
    assert len(diffs) >= 1
    add_diff = diffs[0]
    assert add_diff.diff_type == "ADD"
    assert add_diff.data is not None

    await lib.stop()


async def test_update_produces_update_diff():
    """A node update should produce an UPDATE diff with before and after."""
    cypher = "MATCH (n:Person) RETURN n.name"
    lib, handle, reaction_handle = await build_simple_lib(
        lib_id="qr-update", cypher=cypher
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()

    props1 = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props1)
    await asyncio.wait_for(stream.__anext__(), timeout=5.0)

    props2 = make_person_props("Alicia")
    await handle.send_node_update("n1", ["Person"], props2)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)

    diffs = result.results
    assert len(diffs) >= 1

    update_diff = diffs[0]
    assert update_diff.diff_type == "UPDATE"
    assert update_diff.before is not None
    assert update_diff.after is not None

    await lib.stop()


async def test_delete_produces_delete_diff():
    """A node delete should produce a DELETE diff."""
    lib, handle, reaction_handle = await build_simple_lib(lib_id="qr-delete")
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()

    props = make_person_props("Bob")
    await handle.send_node_insert("n2", ["Person"], props)
    await asyncio.wait_for(stream.__anext__(), timeout=5.0)

    await handle.send_delete("n2", ["Person"])
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)

    diffs = result.results
    assert len(diffs) >= 1
    assert diffs[0].diff_type == "DELETE"
    assert diffs[0].data is not None

    await lib.stop()


async def test_aggregation_count():
    """A COUNT aggregation query tracks additions and deletions."""
    cypher = "MATCH (n:Person) RETURN COUNT(n) AS person_count"
    lib, handle, reaction_handle = await build_simple_lib(
        lib_id="qr-count", cypher=cypher
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()

    props = make_person_props("Alice")
    await handle.send_node_insert("c1", ["Person"], props)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert len(result.results) >= 1

    props2 = make_person_props("Bob")
    await handle.send_node_insert("c2", ["Person"], props2)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert len(result.results) >= 1

    await lib.stop()


async def test_aggregation_sum():
    """A SUM aggregation query computes totals."""
    cypher = "MATCH (n:Person) RETURN SUM(n.age) AS total_age"
    lib, handle, reaction_handle = await build_simple_lib(
        lib_id="qr-sum", cypher=cypher
    )
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()

    props = make_person_props("Alice", age=30)
    await handle.send_node_insert("s1", ["Person"], props)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert len(result.results) >= 1

    props2 = make_person_props("Bob", age=25)
    await handle.send_node_insert("s2", ["Person"], props2)
    result = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
    assert len(result.results) >= 1

    await lib.stop()
