"""HTTP Source example: Receive graph changes via HTTP endpoints.

Starts an HttpSource that listens on port 8080 for data change events.
A continuous Cypher query filters for sensors with high temperature readings,
and results are printed as they arrive.

Usage:
    1. python main.py                  — start the HTTP source
    2. ./send_changes.sh               — (separate terminal) send sample events
    3. Observe the output in main.py
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_http import HttpSource
from drasi_reaction_application import ApplicationReaction


async def main():
    # Configure the HTTP source to listen on port 8080
    http_builder = HttpSource.builder("sensor-source")
    http_builder.with_host("0.0.0.0")
    http_builder.with_port(8080)
    http_builder.with_timeout_ms(30000)
    http_builder.with_auto_start(True)
    http_source = http_builder.build()

    # Create an ApplicationReaction to receive query results
    reaction_builder = ApplicationReaction.builder("sensor-alerts")
    reaction_builder.with_query("hot-sensors")
    reaction_builder.with_auto_start(True)
    reaction, reaction_handle = reaction_builder.build()

    # Build DrasiLib with a query for high-temperature sensors
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("http-source-example")
    lib_builder.with_source(http_source.into_source_wrapper())

    query = Query.cypher("hot-sensors")
    query.query(
        "MATCH (s:Sensor) WHERE s.value > 80 "
        "RETURN s.name AS sensor_name, s.value AS temperature, s.location AS location"
    )
    query.from_source("sensor-source")
    query.auto_start(True)
    lib_builder.with_query(query.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    print("HTTP source listening on http://0.0.0.0:8080")
    print("Send events to: POST /sources/sensor-source/events")
    print()
    print("Try: ./send_changes.sh")
    print("Press Ctrl+C to stop...\n")

    # Stream results as they arrive
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            async for result in stream:
                print(f"  Hot sensor alert: {result}")
        except (asyncio.CancelledError, KeyboardInterrupt):
            pass

    await lib.stop()
    print("DrasiLib stopped")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutdown requested")
