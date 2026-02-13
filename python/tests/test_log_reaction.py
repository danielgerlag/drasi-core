"""Tests for LogReaction: verify it doesn't crash when configured with templates."""

import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_reaction_log import LogReaction
from drasi_source_application import ApplicationSource

from .conftest import make_person_props


async def test_log_reaction_basic():
    """LogReaction with default template runs without errors."""
    source = ApplicationSource("log-source")
    handle = source.get_handle()

    log_builder = LogReaction.builder("log-reaction")
    log_builder.with_query("log-query")
    log_reaction = log_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("log-test-1")
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher("log-query")
    q.query("MATCH (n:Person) RETURN n.name")
    q.from_source("log-source")
    q.auto_start(True)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(log_reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    props = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props)
    await asyncio.sleep(0.5)

    await lib.stop()


async def test_log_reaction_with_template():
    """LogReaction with Handlebars template processes data without crashing."""
    source = ApplicationSource("log-source-2")
    handle = source.get_handle()

    log_builder = LogReaction.builder("log-reaction-2")
    log_builder.with_query("log-query-2")
    log_builder.with_default_template(
        added="Added: {{data}}",
        updated="Updated: {{after}}",
        deleted="Deleted: {{data}}",
    )
    log_reaction = log_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("log-test-2")
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher("log-query-2")
    q.query("MATCH (n:Person) RETURN n.name")
    q.from_source("log-source-2")
    q.auto_start(True)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(log_reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    props = make_person_props("Bob")
    await handle.send_node_insert("n2", ["Person"], props)
    await asyncio.sleep(0.5)

    await lib.stop()


async def test_log_reaction_with_route():
    """LogReaction with a per-query route template."""
    source = ApplicationSource("log-source-3")
    handle = source.get_handle()

    log_builder = LogReaction.builder("log-reaction-3")
    log_builder.with_query("log-query-3")
    log_builder.with_route(
        "log-query-3",
        added="Route Added: {{data}}",
    )
    log_reaction = log_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("log-test-3")
    lib_builder.with_source(source.into_source_wrapper())

    q = Query.cypher("log-query-3")
    q.query("MATCH (n:Person) RETURN n.name")
    q.from_source("log-source-3")
    q.auto_start(True)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(log_reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    props = make_person_props("Charlie")
    await handle.send_node_insert("n3", ["Person"], props)
    await asyncio.sleep(0.5)

    await lib.stop()
