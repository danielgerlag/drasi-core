"""Multi-query pipeline: Fan-out from one source to multiple queries and reactions.

Demonstrates connecting a single ApplicationSource to multiple independent
Cypher queries, each with its own ApplicationReaction. This is useful when
you need to detect different patterns in the same data stream.

Pattern:
                       ┌─ Query A (low stock)   ─→ Reaction A
    Source ─────────── ┼─ Query B (high value)   ─→ Reaction B
                       └─ Query C (all products) ─→ Reaction C
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import ApplicationSource, PropertyMapBuilder
from drasi_reaction_application import ApplicationReaction


async def read_reaction(label: str, reaction_handle):
    """Read a single result from a reaction stream with timeout."""
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
            print(f"    {result}")
        except (asyncio.TimeoutError, StopAsyncIteration):
            print(f"    (no results within timeout)")


async def main():
    print("=== Multi-Query Pipeline Example ===\n")

    # Step 1: Create a single shared source
    print("Step 1: Creating shared ApplicationSource...")
    source = ApplicationSource("inventory-source")
    handle = source.get_handle()
    print("  ✓ Source 'inventory-source' created\n")

    # Step 2: Create three reactions for three different queries
    print("Step 2: Creating three ApplicationReactions...")

    # Reaction A: Low stock alerts (quantity < 10)
    builder_a = ApplicationReaction.builder("low-stock-reaction")
    builder_a.with_query("low-stock")
    builder_a.with_auto_start(True)
    reaction_a, handle_a = builder_a.build()
    print("  ✓ Reaction A: low-stock-reaction (qty < 10)")

    # Reaction B: High-value items (price > 100)
    builder_b = ApplicationReaction.builder("high-value-reaction")
    builder_b.with_query("high-value")
    builder_b.with_auto_start(True)
    reaction_b, handle_b = builder_b.build()
    print("  ✓ Reaction B: high-value-reaction (price > 100)")

    # Reaction C: Full inventory listing
    builder_c = ApplicationReaction.builder("all-items-reaction")
    builder_c.with_query("all-items")
    builder_c.with_auto_start(True)
    reaction_c, handle_c = builder_c.build()
    print("  ✓ Reaction C: all-items-reaction (all products)\n")

    # Step 3: Build DrasiLib with all components
    print("Step 3: Building DrasiLib with 1 source, 3 queries, 3 reactions...")
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("multi-query-example")
    lib_builder.with_source(source.into_source_wrapper())

    # Query A: Items with stock below 10
    q_low = Query.cypher("low-stock")
    q_low.query(
        "MATCH (p:Product) WHERE p.quantity < 10 "
        "RETURN p.name AS name, p.quantity AS qty"
    )
    q_low.from_source("inventory-source")
    q_low.auto_start(True)
    lib_builder.with_query(q_low.build())

    # Query B: Items worth more than $100
    q_high = Query.cypher("high-value")
    q_high.query(
        "MATCH (p:Product) WHERE p.price > 100 "
        "RETURN p.name AS name, p.price AS price"
    )
    q_high.from_source("inventory-source")
    q_high.auto_start(True)
    lib_builder.with_query(q_high.build())

    # Query C: All items
    q_all = Query.cypher("all-items")
    q_all.query(
        "MATCH (p:Product) "
        "RETURN p.name AS name, p.price AS price, p.quantity AS qty"
    )
    q_all.from_source("inventory-source")
    q_all.auto_start(True)
    lib_builder.with_query(q_all.build())

    # Register all three reactions
    lib_builder.with_reaction(reaction_a.into_reaction_wrapper())
    lib_builder.with_reaction(reaction_b.into_reaction_wrapper())
    lib_builder.with_reaction(reaction_c.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    print("  ✓ DrasiLib started successfully\n")

    # Step 4: Push inventory data
    print("Step 4: Pushing product data...")
    products = [
        ("widget-a", "Widget A", 150.0, 5),   # Low stock + high value
        ("widget-b", "Widget B", 25.0, 100),   # Normal (neither alert)
        ("widget-c", "Widget C", 200.0, 3),    # Low stock + high value
        ("widget-d", "Widget D", 10.0, 50),    # Normal
        ("widget-e", "Widget E", 75.0, 8),     # Low stock only
    ]

    for pid, name, price, qty in products:
        props = PropertyMapBuilder()
        props.with_string("name", name)
        props.with_float("price", price)
        props.with_integer("quantity", qty)
        await handle.send_node_insert(pid, ["Product"], props.build())
        print(f"  Inserted: {name} (${price:.2f}, qty={qty})")

    # Allow time for changes to propagate
    await asyncio.sleep(0.5)

    # Step 5: Read results from each reaction
    print("\nStep 5: Reading results from each reaction...")

    print("\n  [Low Stock] Items with quantity < 10:")
    print("  Expected: Widget A (qty=5), Widget C (qty=3), Widget E (qty=8)")
    await read_reaction("Low Stock", handle_a)

    print("\n  [High Value] Items with price > $100:")
    print("  Expected: Widget A ($150), Widget C ($200)")
    await read_reaction("High Value", handle_b)

    print("\n  [All Items] Complete inventory:")
    print("  Expected: All 5 products")
    await read_reaction("All Items", handle_c)

    # Step 6: Update a product — triggers different reactions
    print("\nStep 6: Updating Widget B price to $120 (now high-value)...")
    update_props = PropertyMapBuilder()
    update_props.with_string("name", "Widget B")
    update_props.with_float("price", 120.0)
    update_props.with_integer("quantity", 100)
    await handle.send_node_update("widget-b", ["Product"], update_props.build())
    print("  ✓ Widget B updated")

    await asyncio.sleep(0.5)

    print("\n  [High Value] should now include Widget B:")
    await read_reaction("High Value", handle_b)

    print("\n  [All Items] should show Widget B's new price:")
    await read_reaction("All Items", handle_c)

    # Step 7: List active components
    print("\nStep 7: Listing active components...")
    sources = await lib.list_sources()
    queries = await lib.list_queries()
    reactions = await lib.list_reactions()
    print(f"  Active: {len(sources)} source(s), "
          f"{len(queries)} query/queries, {len(reactions)} reaction(s)")

    # Step 8: Clean shutdown
    print("\nStep 8: Stopping DrasiLib...")
    await lib.stop()
    print("  ✓ DrasiLib stopped cleanly")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    asyncio.run(main())
