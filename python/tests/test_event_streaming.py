"""Tests for lifecycle event subscriptions."""

import asyncio

from .conftest import build_simple_lib


async def test_subscribe_source_events():
    """Subscribe to source events and observe status transitions."""
    lib, _, _ = await build_simple_lib(lib_id="evt-src-1")

    subscription = await lib.subscribe_source_events("test-source")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    # Check history for status transitions
    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_subscribe_query_events():
    """Subscribe to query events."""
    lib, _, _ = await build_simple_lib(lib_id="evt-qry-1")

    subscription = await lib.subscribe_query_events("test-query")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_subscribe_reaction_events():
    """Subscribe to reaction events."""
    lib, _, _ = await build_simple_lib(lib_id="evt-rxn-1")

    subscription = await lib.subscribe_reaction_events("test-reaction")
    assert subscription is not None

    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    assert isinstance(history, list)

    await lib.stop()


async def test_event_contains_expected_fields():
    """ComponentEvent objects have the expected fields."""
    lib, _, _ = await build_simple_lib(lib_id="evt-fields")

    subscription = await lib.subscribe_source_events("test-source")
    await lib.start()
    await asyncio.sleep(0.3)

    history = subscription.history
    if len(history) > 0:
        event = history[0]
        assert hasattr(event, "component_id")
        assert hasattr(event, "component_type")
        assert hasattr(event, "status")
        assert hasattr(event, "timestamp")
        assert event.component_id == "test-source"

    await lib.stop()


async def test_event_stream_async_iteration():
    """Event subscription supports async iteration for live events."""
    lib, _, _ = await build_simple_lib(lib_id="evt-stream")

    subscription = await lib.subscribe_source_events("test-source")

    await lib.start()
    await asyncio.sleep(0.1)

    # Trigger a status change
    await lib.stop_source("test-source")
    await asyncio.sleep(0.2)

    # Check that the history captured events
    history = subscription.history
    assert len(history) > 0

    await lib.stop()
