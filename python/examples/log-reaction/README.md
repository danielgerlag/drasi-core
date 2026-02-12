# Log Reaction Example

Format continuous query results with Handlebars templates using the LogReaction.

## What This Example Does

1. Creates an in-memory `ApplicationSource` (no external infrastructure needed)
2. Defines two Cypher queries: one for all tasks, one for critical tasks only
3. Configures a `LogReaction` with **default templates** and **route-specific templates**
4. Pushes task nodes through the source and observes formatted log output

## Prerequisites

- **Python 3.10+** with the `drasi` packages installed (see the top-level `python/` README)

## Quick Start

```bash
python main.py
```

## Handlebars Template Syntax

LogReaction uses [Handlebars](https://handlebarsjs.com/) templates to format output.
Each template receives `before` and `after` objects representing the row state before
and after the change.

### Available Variables

| Variable | Available On | Description |
|----------|-------------|-------------|
| `{{after.<field>}}` | insert, update | Field value after the change |
| `{{before.<field>}}` | update, delete | Field value before the change |

### Template Types

| Template | Triggered When |
|----------|---------------|
| `added` | A new row matches the query |
| `updated` | An existing matching row is modified |
| `deleted` | A matching row is removed |

### Default vs Route-Specific Templates

**Default templates** apply to all queries subscribed to the reaction:

```python
log_builder.with_default_template(
    added="[+] New: {{after.title}}",
    updated="[~] Changed: {{after.title}}",
    deleted="[-] Removed: {{before.title}}",
)
```

**Route-specific templates** override defaults for a particular query:

```python
log_builder.with_route(
    "critical-tasks",
    added="🚨 CRITICAL: {{after.title}} — assigned to {{after.assignee}}",
    updated="🚨 UPDATE: {{after.title}}",
    deleted="🚨 REMOVED: {{before.title}}",
)
```

## Expected Output

```
Log reaction started — watch for formatted output below

[+] New task: Fix login bug (assignee: Alice, priority: high)
[+] New task: Write docs (assignee: Bob, priority: medium)
[+] New task: Deploy v2.0 (assignee: Charlie, priority: critical)
🚨 CRITICAL TASK ADDED: Deploy v2.0 — assigned to Charlie

[~] Updated: Fix login bug — priority changed from high to critical
🚨 CRITICAL UPDATE: Fix login bug — was high, now critical

[-] Removed: Write docs

DrasiLib stopped
```

## Files

| File | Description |
|------|-------------|
| `main.py` | Complete example with default and route-specific templates |
| `README.md` | This file |
