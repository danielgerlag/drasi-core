"""Tests for component status transitions and querying."""

import asyncio

from drasi_lib import ComponentStatus

from .conftest import build_simple_lib


async def test_source_status_after_start():
    """Source status should be Running after lib.start()."""
    lib, _, _ = await build_simple_lib(lib_id="status-1")
    await lib.start()
    await asyncio.sleep(0.1)

    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Running

    await lib.stop()


async def test_query_status_after_start():
    """Query with auto_start=True should be Running after lib.start()."""
    lib, _, _ = await build_simple_lib(lib_id="status-2")
    await lib.start()
    await asyncio.sleep(0.1)

    status = await lib.get_query_status("test-query")
    assert status == ComponentStatus.Running

    await lib.stop()


async def test_reaction_status_after_start():
    """Reaction status should be Running after lib.start()."""
    lib, _, _ = await build_simple_lib(lib_id="status-3")
    await lib.start()
    await asyncio.sleep(0.1)

    status = await lib.get_reaction_status("test-reaction")
    assert status == ComponentStatus.Running

    await lib.stop()


async def test_source_stopped_after_stop():
    """Source status should be Stopped after lib.stop()."""
    lib, _, _ = await build_simple_lib(lib_id="status-4")
    await lib.start()
    await lib.stop()
    await asyncio.sleep(0.1)

    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Stopped

    await lib.stop()


async def test_component_status_equality():
    """ComponentStatus enum values support equality comparison."""
    assert ComponentStatus.Running == ComponentStatus.Running
    assert ComponentStatus.Stopped == ComponentStatus.Stopped
    assert ComponentStatus.Running != ComponentStatus.Stopped
    assert ComponentStatus.Starting != ComponentStatus.Error


async def test_stop_and_restart_source():
    """Stop a source individually and then restart it."""
    lib, _, _ = await build_simple_lib(lib_id="status-5")
    await lib.start()
    await asyncio.sleep(0.1)

    await lib.stop_source("test-source")
    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Stopped

    await lib.start_source("test-source")
    await asyncio.sleep(0.1)
    status = await lib.get_source_status("test-source")
    assert status == ComponentStatus.Running

    await lib.stop()


async def test_stop_and_restart_query():
    """Stop a query individually and then restart it."""
    lib, _, _ = await build_simple_lib(lib_id="status-6")
    await lib.start()
    await asyncio.sleep(0.1)

    await lib.stop_query("test-query")
    status = await lib.get_query_status("test-query")
    assert status == ComponentStatus.Stopped

    await lib.start_query("test-query")
    await asyncio.sleep(0.1)
    status = await lib.get_query_status("test-query")
    assert status == ComponentStatus.Running

    await lib.stop()
