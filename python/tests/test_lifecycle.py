"""Tests for DrasiLib lifecycle: build, start, stop, restart."""


from drasi_lib import ComponentStatus, DrasiLibBuilder

from .conftest import build_simple_lib


async def test_start_is_running_stop():
    """Build → start → is_running → stop cycle."""
    lib, _, _ = await build_simple_lib(lib_id="lifecycle-1")

    assert not await lib.is_running()

    await lib.start()
    assert await lib.is_running()

    await lib.stop()
    assert not await lib.is_running()


async def test_restart():
    """Stop and restart the lib."""
    lib, _, _ = await build_simple_lib(lib_id="lifecycle-2")

    await lib.start()
    assert await lib.is_running()

    await lib.stop()
    assert not await lib.is_running()

    await lib.start()
    assert await lib.is_running()

    await lib.stop()


async def test_build_with_no_sources():
    """DrasiLib can be built with only a query (no sources or reactions)."""
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("empty-lib")

    lib = await lib_builder.build()
    await lib.start()
    assert await lib.is_running()
    await lib.stop()


async def test_component_listing():
    """After building, sources, queries, and reactions appear in list methods."""
    lib, _, _ = await build_simple_lib(lib_id="lifecycle-list")
    await lib.start()

    sources = await lib.list_sources()
    assert len(sources) == 1
    assert sources[0][0] == "test-source"

    queries = await lib.list_queries()
    assert len(queries) == 1
    assert queries[0][0] == "test-query"

    reactions = await lib.list_reactions()
    assert len(reactions) == 1
    assert reactions[0][0] == "test-reaction"

    await lib.stop()


async def test_stop_individual_components():
    """Individual source, query, reaction can be stopped and restarted."""
    lib, _, _ = await build_simple_lib(lib_id="lifecycle-indiv")
    await lib.start()

    await lib.stop_source("test-source")
    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Stopped

    await lib.start_source("test-source")
    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Running

    await lib.stop()
