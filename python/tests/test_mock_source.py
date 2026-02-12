"""Tests for MockSource for controlled testing scenarios."""

import asyncio

from drasi_lib import ComponentStatus, DrasiLibBuilder, Query
from drasi_reaction_application import PyApplicationReaction
from drasi_source_mock import PyMockSource


async def test_mock_source_builder():
    """Build a MockSource with builder pattern."""
    builder = PyMockSource.builder("mock-src-1")
    builder.with_interval_ms(100)
    builder.with_auto_start(True)
    mock_source = builder.build()
    assert mock_source is not None


async def test_mock_source_into_wrapper():
    """MockSource can be converted to a source wrapper."""
    builder = PyMockSource.builder("mock-src-2")
    mock_source = builder.build()
    wrapper = mock_source.into_source_wrapper()
    assert wrapper is not None


async def test_mock_source_in_lib():
    """MockSource can be registered with DrasiLib."""
    builder = PyMockSource.builder("mock-src-3")
    builder.with_auto_start(True)
    mock_source = builder.build()

    reaction_builder = PyApplicationReaction.builder("mock-reaction")
    reaction_builder.with_query("mock-query")
    reaction, _ = reaction_builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("mock-test")
    lib_builder.with_source(mock_source.into_source_wrapper())

    q = Query.cypher("mock-query")
    q.query("MATCH (n:Item) RETURN n.value")
    q.from_source("mock-src-3")
    q.auto_start(True)
    lib_builder.with_query(q.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.2)

    status = await lib.get_source_status("mock-src-3")
    assert status == ComponentStatus.Running

    await lib.stop()


async def test_mock_source_auto_start_false():
    """MockSource with auto_start=False remains stopped until started."""
    builder = PyMockSource.builder("mock-src-4")
    builder.with_auto_start(False)
    mock_source = builder.build()

    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("mock-noauto")
    lib_builder.with_source(mock_source.into_source_wrapper())

    q = Query.cypher("mock-query-2")
    q.query("MATCH (n:Item) RETURN n.value")
    q.from_source("mock-src-4")
    q.auto_start(True)
    lib_builder.with_query(q.build())

    lib = await lib_builder.build()
    await lib.start()
    await asyncio.sleep(0.1)

    status = await lib.get_source_status("mock-src-4")
    assert status == ComponentStatus.Stopped

    await lib.start_source("mock-src-4")
    await asyncio.sleep(0.1)
    status = await lib.get_source_status("mock-src-4")
    assert status == ComponentStatus.Running

    await lib.stop()
