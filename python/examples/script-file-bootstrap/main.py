"""ScriptFile Bootstrap example: Load initial data from JSONL files.

Demonstrates using ScriptFileBootstrapProvider to pre-populate the query
index with data from JSONL files before streaming begins. This is useful
for testing and development when you want deterministic initial state.

Flow:
1. Create a temporary JSONL file with initial Person data
2. Use ScriptFileBootstrapProvider to load it at startup
3. After start, bootstrapped data is already in the query result set
4. Push additional changes and read results
"""
import asyncio
import json
import os
import tempfile

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import ApplicationSource, PropertyMapBuilder
from drasi_reaction_application import ApplicationReaction
from drasi_bootstrap_scriptfile import ScriptFileBootstrapProvider


def create_bootstrap_file(path: str):
    """Create a JSONL bootstrap file with initial Person data.

    The ScriptFile format uses JSON Lines with the following record types:
    - Header (required first): metadata about the bootstrap
    - Node: a graph node with id, labels, and properties
    - Relation: a graph relation with id, labels, properties, start/end
    - Finish (optional): marks the end of bootstrap data
    """
    records = [
        {
            "kind": "Header",
            "start_time": "2024-01-01T00:00:00Z",
            "description": "Initial person data for bootstrap example",
        },
        {
            "kind": "Node",
            "id": "person-alice",
            "labels": ["Person"],
            "properties": {"name": "Alice", "age": 30, "department": "Engineering"},
        },
        {
            "kind": "Node",
            "id": "person-bob",
            "labels": ["Person"],
            "properties": {"name": "Bob", "age": 25, "department": "Marketing"},
        },
        {
            "kind": "Node",
            "id": "person-charlie",
            "labels": ["Person"],
            "properties": {"name": "Charlie", "age": 35, "department": "Engineering"},
        },
        {
            "kind": "Node",
            "id": "person-diana",
            "labels": ["Person"],
            "properties": {"name": "Diana", "age": 28, "department": "Sales"},
        },
        {
            "kind": "Node",
            "id": "person-eve",
            "labels": ["Person"],
            "properties": {"name": "Eve", "age": 42, "department": "Engineering"},
        },
        {"kind": "Finish", "description": "Bootstrap complete"},
    ]
    with open(path, "w") as f:
        for record in records:
            f.write(json.dumps(record) + "\n")


async def main():
    print("=== ScriptFile Bootstrap Example ===\n")

    # Step 1: Create a temporary bootstrap data file
    print("Step 1: Creating bootstrap data file...")
    tmp_dir = tempfile.mkdtemp()
    bootstrap_path = os.path.join(tmp_dir, "people.jsonl")
    create_bootstrap_file(bootstrap_path)
    print(f"  ✓ Created bootstrap file with 5 Person nodes: {bootstrap_path}\n")

    try:
        # Step 2: Create the ScriptFileBootstrapProvider
        print("Step 2: Creating ScriptFileBootstrapProvider...")
        bootstrap_builder = ScriptFileBootstrapProvider.builder()
        bootstrap_builder.with_file(bootstrap_path)
        bootstrap_provider = bootstrap_builder.build()
        print("  ✓ Bootstrap provider configured\n")

        # Step 3: Create source and reaction
        print("Step 3: Creating source and reaction...")
        source = ApplicationSource("people-source")
        handle = source.get_handle()

        reaction_builder = ApplicationReaction.builder("people-reaction")
        reaction_builder.with_query("engineers-query")
        reaction_builder.with_auto_start(True)
        reaction, reaction_handle = reaction_builder.build()
        print("  ✓ ApplicationSource and ApplicationReaction created\n")

        # Step 4: Build DrasiLib with bootstrap enabled
        print("Step 4: Building DrasiLib with bootstrap...")
        lib_builder = DrasiLibBuilder()
        lib_builder.with_id("bootstrap-example")
        lib_builder.with_source(source.into_source_wrapper())

        # Query selects engineers (department = 'Engineering')
        query = Query.cypher("engineers-query")
        query.query(
            "MATCH (p:Person) WHERE p.department = 'Engineering' "
            "RETURN p.name AS name, p.age AS age, p.department AS dept"
        )
        query.from_source("people-source")
        query.auto_start(True)
        query.enable_bootstrap(True)
        lib_builder.with_query(query.build())

        lib_builder.with_reaction(reaction.into_reaction_wrapper())

        lib = await lib_builder.build()
        await lib.start()
        print("  ✓ DrasiLib started with bootstrapped data")
        print("    (Alice, Charlie, Eve should already be in the result set)\n")

        # Step 5: Read bootstrapped results
        print("Step 5: Reading bootstrapped results...")
        await asyncio.sleep(0.5)
        stream = await reaction_handle.as_stream()
        if stream is not None:
            try:
                result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
                print(f"  Bootstrap result: {result}")
            except (asyncio.TimeoutError, StopAsyncIteration):
                print("  No bootstrap results available within timeout")

        # Step 6: Push an additional engineer
        print("\nStep 6: Adding a new engineer (Frank, age 31)...")
        props = PropertyMapBuilder()
        props.with_string("name", "Frank")
        props.with_integer("age", 31)
        props.with_string("department", "Engineering")
        await handle.send_node_insert("person-frank", ["Person"], props.build())
        print("  ✓ Frank inserted")

        await asyncio.sleep(0.5)

        # Read the new result
        print("\n  Reading result after insert...")
        stream = await reaction_handle.as_stream()
        if stream is not None:
            try:
                result = await asyncio.wait_for(stream.__anext__(), timeout=2.0)
                print(f"  Insert diff: {result}")
            except (asyncio.TimeoutError, StopAsyncIteration):
                print("  No results available within timeout")

        # Step 7: Push a non-engineer (should not appear in results)
        print("\nStep 7: Adding a non-engineer (Grace, Sales)...")
        props2 = PropertyMapBuilder()
        props2.with_string("name", "Grace")
        props2.with_integer("age", 29)
        props2.with_string("department", "Sales")
        await handle.send_node_insert("person-grace", ["Person"], props2.build())
        print("  ✓ Grace inserted (should NOT appear in engineer query results)")

        await asyncio.sleep(0.5)

        # Step 8: Clean shutdown
        print("\nStep 8: Stopping DrasiLib...")
        await lib.stop()
        print("  ✓ DrasiLib stopped cleanly")

    finally:
        # Cleanup temp files
        if os.path.exists(bootstrap_path):
            os.remove(bootstrap_path)
        if os.path.exists(tmp_dir):
            os.rmdir(tmp_dir)
        print("  ✓ Temporary files cleaned up")

    print("\n=== Example Complete ===")


if __name__ == "__main__":
    asyncio.run(main())
