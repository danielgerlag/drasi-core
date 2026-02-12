"""Event and log monitoring: Subscribe to component lifecycle events and logs.

Demonstrates the observability features of DrasiLib:
  - subscribe_source_events / subscribe_query_events / subscribe_reaction_events
  - subscribe_source_logs / subscribe_query_logs / subscribe_reaction_logs

These subscriptions return an object with:
  - .history  : list of past events/logs at subscription time
  - async for : live stream of new events/logs

This example shows how to:
1. Check component statuses
2. Subscribe to lifecycle events (history + live streaming)
3. Subscribe to component logs
4. Observe events as components start, process data, and stop
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import PyApplicationSource, PyPropertyMapBuilder
from drasi_reaction_application import PyApplicationReaction


async def monitor_events(lib, component_type: str, component_id: str):
    """Subscribe to lifecycle events and print history + live events."""
    subscribe = {
        "source": lib.subscribe_source_events,
        "query": lib.subscribe_query_events,
        "reaction": lib.subscribe_reaction_events,
    }[component_type]

    sub = await subscribe(component_id)

    # Print historical events (events that occurred before subscription)
    for event in sub.history:
        print(f"  [history] {component_type}/{component_id}: "
              f"status={event.status}, message={event.message}")

    # Stream live events as they happen
    async for event in sub:
        print(f"  [live]    {component_type}/{component_id}: "
              f"status={event.status}, message={event.message}")


async def monitor_logs(lib, component_type: str, component_id: str):
    """Subscribe to component logs and print history + live logs."""
    subscribe = {
        "source": lib.subscribe_source_logs,
        "query": lib.subscribe_query_logs,
        "reaction": lib.subscribe_reaction_logs,
    }[component_type]

    sub = await subscribe(component_id)

    # Print historical logs
    for log in sub.history:
        print(f"  [log-history] {component_type}/{component_id}: "
              f"[{log.level}] {log.message}")

    # Stream live logs
    async for log in sub:
        print(f"  [log-live]    {component_type}/{component_id}: "
              f"[{log.level}] {log.message}")


async def main():
    print("=== Event & Log Monitoring Example ===\n")

    # Step 1: Create components
    print("Step 1: Creating components...")
    source = PyApplicationSource("monitored-source")
    handle = source.get_handle()

    reaction_builder = PyApplicationReaction.builder("monitored-reaction")
    reaction_builder.with_query("monitored-query")
    reaction_builder.with_auto_start(True)
    reaction, reaction_handle = reaction_builder.build()
    print("  ✓ Source, query, and reaction created\n")

    # Step 2: Build DrasiLib
    print("Step 2: Building DrasiLib...")
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("monitoring-example")
    lib_builder.with_source(source.into_source_wrapper())

    query = Query.cypher("monitored-query")
    query.query("MATCH (n:Item) RETURN n.name AS name")
    query.from_source("monitored-source")
    query.auto_start(True)
    lib_builder.with_query(query.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()

    # Step 3: Start the system
    await lib.start()
    print("  ✓ DrasiLib started\n")

    # Step 4: Check component statuses
    print("Step 4: Checking component statuses...")
    source_status = await lib.get_source_status("monitored-source")
    query_status = await lib.get_query_status("monitored-query")
    reaction_status = await lib.get_reaction_status("monitored-reaction")
    print(f"  Source status:   {source_status}")
    print(f"  Query status:    {query_status}")
    print(f"  Reaction status: {reaction_status}")

    # Step 5: Subscribe to events and logs as background tasks
    print("\nStep 5: Subscribing to events and logs...")
    print("  (Background tasks will print events/logs as they arrive)\n")
    tasks = [
        asyncio.create_task(
            monitor_events(lib, "source", "monitored-source")
        ),
        asyncio.create_task(
            monitor_events(lib, "query", "monitored-query")
        ),
        asyncio.create_task(
            monitor_events(lib, "reaction", "monitored-reaction")
        ),
        asyncio.create_task(
            monitor_logs(lib, "source", "monitored-source")
        ),
        asyncio.create_task(
            monitor_logs(lib, "query", "monitored-query")
        ),
    ]

    # Give subscriptions time to print history
    await asyncio.sleep(0.3)

    # Step 6: Push data to trigger activity
    print("\nStep 6: Pushing data to trigger events...")
    for i in range(3):
        props = PyPropertyMapBuilder()
        props.with_string("name", f"item-{i}")
        await handle.send_node_insert(f"item-{i}", ["Item"], props.build())
        print(f"  Inserted item-{i}")
        await asyncio.sleep(0.2)

    # Step 7: List all components and their statuses
    print("\nStep 7: Listing all components...")
    for sid, status in await lib.list_sources():
        print(f"  Source:   {sid} — {status}")
    for qid, status in await lib.list_queries():
        print(f"  Query:    {qid} — {status}")
    for rid, status in await lib.list_reactions():
        print(f"  Reaction: {rid} — {status}")

    # Step 8: Check system running state
    running = await lib.is_running()
    print(f"\n  System running: {running}")

    # Give monitors a moment to receive remaining events
    await asyncio.sleep(0.5)

    # Step 9: Cancel monitoring tasks and stop
    print("\nStep 9: Stopping monitoring and DrasiLib...")
    for task in tasks:
        task.cancel()
    await asyncio.gather(*tasks, return_exceptions=True)
    print("  ✓ Monitoring tasks cancelled")

    await lib.stop()
    print("  ✓ DrasiLib stopped cleanly")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    asyncio.run(main())
