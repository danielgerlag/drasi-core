"""Tests for log subscriptions."""

import asyncio

from drasi_lib import LogLevel

from .conftest import build_simple_lib, make_person_props


async def test_subscribe_query_logs():
    """Subscribe to query logs and verify subscription is returned."""
    lib, _, _ = await build_simple_lib(lib_id="log-sub-1")

    subscription = await lib.subscribe_query_logs("test-query")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_subscribe_source_logs():
    """Subscribe to source logs."""
    lib, _, _ = await build_simple_lib(lib_id="log-sub-2")

    subscription = await lib.subscribe_source_logs("test-source")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_subscribe_reaction_logs():
    """Subscribe to reaction logs."""
    lib, _, _ = await build_simple_lib(lib_id="log-sub-3")

    subscription = await lib.subscribe_reaction_logs("test-reaction")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_log_message_fields():
    """LogMessage objects have the expected attributes."""
    lib, handle, _ = await build_simple_lib(lib_id="log-sub-4")

    subscription = await lib.subscribe_query_logs("test-query")
    await lib.start()
    await asyncio.sleep(0.1)

    # Trigger some activity to generate logs
    props = make_person_props("Alice")
    await handle.send_node_insert("n1", ["Person"], props)
    await asyncio.sleep(0.5)

    history = subscription.history
    if len(history) > 0:
        msg = history[0]
        assert hasattr(msg, "timestamp")
        assert hasattr(msg, "level")
        assert hasattr(msg, "message")
        assert hasattr(msg, "component_id")
        assert hasattr(msg, "component_type")
        assert hasattr(msg, "instance_id")

    await lib.stop()


async def test_log_level_enum_values():
    """LogLevel enum values are accessible."""
    assert LogLevel.Trace is not None
    assert LogLevel.Debug is not None
    assert LogLevel.Info is not None
    assert LogLevel.Warn is not None
    assert LogLevel.Error is not None
    assert LogLevel.Info != LogLevel.Error
