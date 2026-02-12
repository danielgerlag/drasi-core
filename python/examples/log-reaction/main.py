"""Log Reaction example: Format query results with Handlebars templates.

Demonstrates using LogReaction to print formatted output to the console
whenever query results change. Supports default templates and per-query
route-specific templates using Handlebars syntax.

Template variables:
  - {{after.<field>}}  : field value after the change (inserts and updates)
  - {{before.<field>}} : field value before the change (updates and deletes)

No external infrastructure required — uses ApplicationSource to push data.
"""
import asyncio

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import PyApplicationSource, PyPropertyMapBuilder
from drasi_reaction_log import PyLogReaction


async def main():
    # Step 1: Create an ApplicationSource to push data programmatically
    source = PyApplicationSource("tasks-source")
    handle = source.get_handle()

    # Step 2: Build the LogReaction with Handlebars templates
    log_builder = PyLogReaction.builder("task-logger")

    # Subscribe to both queries
    log_builder.with_queries(["all-tasks", "critical-tasks"])
    log_builder.with_auto_start(True)

    # Default templates apply to any query without a specific route
    log_builder.with_default_template(
        added="[+] New task: {{after.title}} (assignee: {{after.assignee}}, priority: {{after.priority}})",
        updated="[~] Updated: {{after.title}} — priority changed from {{before.priority}} to {{after.priority}}",
        deleted="[-] Removed: {{before.title}}",
    )

    # Route-specific template for the critical-tasks query
    log_builder.with_route(
        "critical-tasks",
        added="🚨 CRITICAL TASK ADDED: {{after.title}} — assigned to {{after.assignee}}",
        updated="🚨 CRITICAL UPDATE: {{after.title}} — was {{before.priority}}, now {{after.priority}}",
        deleted="🚨 CRITICAL TASK REMOVED: {{before.title}}",
    )

    log_reaction = log_builder.build()

    # Step 3: Build DrasiLib with two queries
    lib_builder = DrasiLibBuilder()
    lib_builder.with_id("log-reaction-example")
    lib_builder.with_source(source.into_source_wrapper())

    # Query 1: All tasks
    q1 = Query.cypher("all-tasks")
    q1.query(
        "MATCH (t:Task) "
        "RETURN t.title AS title, t.assignee AS assignee, t.priority AS priority"
    )
    q1.from_source("tasks-source")
    q1.auto_start(True)
    lib_builder.with_query(q1.build())

    # Query 2: Only critical tasks
    q2 = Query.cypher("critical-tasks")
    q2.query(
        "MATCH (t:Task) WHERE t.priority = 'critical' "
        "RETURN t.title AS title, t.assignee AS assignee, t.priority AS priority"
    )
    q2.from_source("tasks-source")
    q2.auto_start(True)
    lib_builder.with_query(q2.build())

    lib_builder.with_reaction(log_reaction.into_reaction_wrapper())

    lib = await lib_builder.build()
    await lib.start()
    print("Log reaction started — watch for formatted output below\n")

    # Step 4: Insert tasks (triggers "added" templates)
    tasks = [
        ("task-1", "Fix login bug", "Alice", "high"),
        ("task-2", "Write docs", "Bob", "medium"),
        ("task-3", "Deploy v2.0", "Charlie", "critical"),
    ]
    for tid, title, assignee, priority in tasks:
        props = PyPropertyMapBuilder()
        props.with_string("title", title)
        props.with_string("assignee", assignee)
        props.with_string("priority", priority)
        await handle.send_node_insert(tid, ["Task"], props.build())

    await asyncio.sleep(0.5)

    # Step 5: Update a task to critical (triggers "updated" templates)
    props = PyPropertyMapBuilder()
    props.with_string("title", "Fix login bug")
    props.with_string("assignee", "Alice")
    props.with_string("priority", "critical")
    await handle.send_node_update("task-1", ["Task"], props.build())

    await asyncio.sleep(0.5)

    # Step 6: Delete a task (triggers "deleted" templates)
    await handle.send_delete("task-2", ["Task"])

    await asyncio.sleep(0.5)

    await lib.stop()
    print("\nDrasiLib stopped")


if __name__ == "__main__":
    asyncio.run(main())
