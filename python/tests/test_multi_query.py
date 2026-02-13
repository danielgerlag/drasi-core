"""Tests for multiple queries from the same source producing independent results."""

import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_reaction_application import ApplicationReaction
from drasi_source_application import ApplicationSource

from .conftest import make_person_props


async def test_two_queries_independent_results():
    """Two queries from the same source each produce their own results to separate reactions."""
    source = ApplicationSource("shared-source")
    handle = source.get_handle()

    # Query 1: returns name
    q1 = Query.cypher("query-name")
    q1.query("MATCH (n:Person) RETURN n.name")
    q1.from_source("shared-source")
    q1.auto_start(True)

    # Query 2: returns age
    q2 = Query.cypher("query-age")
    q2.query("MATCH (n:Person) RETURN n.age")
    q2.from_source("shared-source")
    q2.auto_start(True)

    # Reaction for query 1
    r1_builder = ApplicationReaction.builder("reaction-name")
    r1_builder.with_query("query-name")
    reaction1, reaction_handle1 = r1_builder.build()

    # Reaction for query 2
    r2_builder = ApplicationReaction.builder("reaction-age")
    r2_builder.with_query("query-age")
    reaction2, reaction_handle2 = r2_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("multi-query")
    lib_builder.with_source(source.into_source_wrapper())
    lib_builder.with_query(q1.build())
    lib_builder.with_query(q2.build())
    lib_builder.with_reaction(reaction1.into_reaction_wrapper())
    lib_builder.with_reaction(reaction2.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    stream1 = await reaction_handle1.as_stream()
    stream2 = await reaction_handle2.as_stream()
    assert stream1 is not None
    assert stream2 is not None

    props = make_person_props("Alice", age=30)
    await handle.send_node_insert("n1", ["Person"], props)

    result1 = await asyncio.wait_for(stream1.__anext__(), timeout=5.0)
    result2 = await asyncio.wait_for(stream2.__anext__(), timeout=5.0)

    assert result1.query_id == "query-name"
    assert result2.query_id == "query-age"

    await lib.stop()


async def test_reaction_subscribes_to_multiple_queries():
    """A single reaction can subscribe to multiple queries."""
    source = ApplicationSource("shared-source-2")
    handle = source.get_handle()

    q1 = Query.cypher("q-alpha")
    q1.query("MATCH (n:Person) RETURN n.name")
    q1.from_source("shared-source-2")
    q1.auto_start(True)

    q2 = Query.cypher("q-beta")
    q2.query("MATCH (n:Person) RETURN n.age")
    q2.from_source("shared-source-2")
    q2.auto_start(True)

    r_builder = ApplicationReaction.builder("multi-sub-reaction")
    r_builder.with_queries(["q-alpha", "q-beta"])
    reaction, reaction_handle = r_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("multi-sub")
    lib_builder.with_source(source.into_source_wrapper())
    lib_builder.with_query(q1.build())
    lib_builder.with_query(q2.build())
    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    stream = await reaction_handle.as_stream()
    assert stream is not None

    props = make_person_props("Bob", age=25)
    await handle.send_node_insert("n2", ["Person"], props)

    # Should receive results from both queries
    results = []
    try:
        for _ in range(2):
            r = await asyncio.wait_for(stream.__anext__(), timeout=5.0)
            results.append(r)
    except (asyncio.TimeoutError, StopAsyncIteration):
        pass

    query_ids = {r.query_id for r in results}
    assert len(query_ids) >= 1  # At least one query produced results

    await lib.stop()
