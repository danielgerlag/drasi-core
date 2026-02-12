"""Basic Drasi usage example: ApplicationSource → Cypher Query → ApplicationReaction

Demonstrates the core Drasi pattern:
1. Create an ApplicationSource to push data programmatically
2. Define a Cypher continuous query
3. Attach an ApplicationReaction to consume query results
4. Push Person nodes and observe real-time query result changes
5. Demonstrate insert, update, and delete operations
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import PyApplicationSource, PyPropertyMapBuilder
from drasi_reaction_application import PyApplicationReaction


async def main():
    print("=== Basic Drasi Usage Example ===\n")

    # Step 1: Create an ApplicationSource
    # This source lets you push graph data (nodes/relations) programmatically.
    print("Step 1: Creating ApplicationSource...")
    source = PyApplicationSource("people-source")
    handle = source.get_handle()
    print("  ✓ ApplicationSource 'people-source' created\n")

    # Step 2: Create an ApplicationReaction
    # This reaction collects query results so you can read them in your app.
    print("Step 2: Creating ApplicationReaction...")
    builder = PyApplicationReaction.builder("people-reaction")
    builder.with_query("people-query")
    builder.with_auto_start(True)
    reaction, reaction_handle = builder.build()
    print("  ✓ ApplicationReaction 'people-reaction' created\n")

    # Step 3: Build the DrasiLib instance with a Cypher query
    print("Step 3: Building DrasiLib with Cypher query...")
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("basic-example")
    lib_builder.with_source(source.into_source_wrapper())

    # Define a Cypher query that matches all Person nodes
    query = Query.cypher("people-query")
    query.query("MATCH (p:Person) RETURN p.name AS name, p.age AS age")
    query.from_source("people-source")
    query.auto_start(True)
    lib_builder.with_query(query.build())

    lib_builder.with_reaction(reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    print("  ✓ DrasiLib started successfully\n")

    # Step 4: Push Person nodes
    print("Step 4: Inserting Person nodes...")
    people = [
        ("Alice", 30),
        ("Bob", 25),
        ("Charlie", 35),
        ("Diana", 28),
        ("Eve", 42),
    ]
    for name, age in people:
        props = PyPropertyMapBuilder()
        props.with_string("name", name)
        props.with_integer("age", age)
        await handle.send_node_insert(
            f"person-{name.lower()}", ["Person"], props.build()
        )
        print(f"  Inserted: {name}, age {age}")

    # Allow time for changes to propagate through the query engine
    await asyncio.sleep(0.5)

    # Step 5: Read results from the reaction stream
    print("\nStep 5: Reading results from reaction stream...")
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
            print(f"  Result: {result}")
        except (asyncio.TimeoutError, StopAsyncIteration):
            print("  No results available within timeout")

    # Step 6: Update a person's age
    print("\nStep 6: Updating Bob's age from 25 to 26...")
    update_props = PyPropertyMapBuilder()
    update_props.with_string("name", "Bob")
    update_props.with_integer("age", 26)
    await handle.send_node_update("person-bob", ["Person"], update_props.build())
    print("  ✓ Bob updated")

    await asyncio.sleep(0.5)

    # Read the update result
    print("\n  Reading update result...")
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
            print(f"  Update diff: {result}")
        except (asyncio.TimeoutError, StopAsyncIteration):
            print("  No update results available within timeout")

    # Step 7: Delete a person
    print("\nStep 7: Deleting Charlie...")
    await handle.send_delete("person-charlie", ["Person"])
    print("  ✓ Charlie deleted")

    await asyncio.sleep(0.5)

    # Read the delete result
    print("\n  Reading delete result...")
    stream = await reaction_handle.as_stream()
    if stream is not None:
        try:
            result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
            print(f"  Delete diff: {result}")
        except (asyncio.TimeoutError, StopAsyncIteration):
            print("  No delete results available within timeout")

    # Step 8: Clean shutdown
    print("\nStep 8: Stopping DrasiLib...")
    await lib.stop()
    print("  ✓ DrasiLib stopped cleanly")
    print("\n=== Example Complete ===")


if __name__ == "__main__":
    asyncio.run(main())
