"""PostgreSQL Source example: Stream live changes from PostgreSQL via WAL replication.

This example connects to a local PostgreSQL instance (started via docker-compose)
and uses logical replication to stream row-level changes. A Cypher continuous
query filters for high-value orders (total_amount > 500), and results are
printed as they arrive.

Usage:
    1. ./setup.sh          — start PostgreSQL in Docker
    2. python main.py      — run this script
    3. python simulate_changes.py   — (separate terminal) trigger changes
    4. ./teardown.sh       — stop and clean up
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_postgres import PyPostgresSource
from drasi_reaction_application import PyApplicationReaction


async def main():
    # Configure the PostgreSQL source pointing at our Docker instance
    pg_builder = PyPostgresSource.builder("orders-source")
    pg_builder.with_host("localhost")
    pg_builder.with_port(5432)
    pg_builder.with_database("drasi_example")
    pg_builder.with_user("drasi")
    pg_builder.with_password("drasi_pass")
    pg_builder.add_table("public.orders")
    pg_builder.with_slot_name("drasi_orders_slot")
    pg_builder.with_auto_start(True)
    pg_source = pg_builder.build()

    # Create an ApplicationReaction to receive query results
    reaction_builder = PyApplicationReaction.builder("order-alerts")
    reaction_builder.with_query("high-value-orders")
    reaction_builder.with_auto_start(True)
    reaction, reaction_handle = reaction_builder.build()

    # Build DrasiLib with a Cypher query for high-value orders
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("postgres-source-example")
    lib_builder.with_source(pg_source.into_source_wrapper())

    query = Query.cypher("high-value-orders")
    query.query(
        "MATCH (o:orders) "
        "WHERE o.total_amount > 500 "
        "RETURN o.id AS order_id, o.customer_name AS customer, "
        "o.product AS product, o.total_amount AS amount, o.status AS status"
    )
    query.from_source("orders-source")
    query.auto_start(True)
    lib_builder.with_query(query.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    print("PostgreSQL source started — listening for WAL changes")
    print("Query: orders with total_amount > 500")
    print("Press Ctrl+C to stop...\n")

    # Stream results as they arrive
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            async for result in stream:
                print(f"  High-value order: {result}")
        except (asyncio.CancelledError, KeyboardInterrupt):
            pass

    await lib.stop()
    print("DrasiLib stopped")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutdown requested")
