# Drasi State Machine Reaction

A unique Drasi component that is **both a reaction and a source**. It maps live
continuous-query results to per-entity **state transitions** and exposes the
realtime state of every entity as a Drasi **source** that downstream queries can
subscribe to.

## How it works

```text
   input queries          state machine                 companion source
 ┌───────────────┐      ┌────────────────┐           ┌─────────────────────┐
 │ draft-orders  │─────▶│  reaction      │  push      │  source             │
 │ paid-orders   │─────▶│  (transitions) │──────────▶ │  (one node/entity)  │──▶ downstream queries
 │ shipped-orders│─────▶│  + persistence │  state     │  + bootstrap        │
 └───────────────┘      └───────┬────────┘           └─────────▲───────────┘
                                │ state store (durable) ───────┘
```

* The **reaction** subscribes to the input queries referenced by its states.
  Each *enter condition* declares:
  * `query` — the input query whose results drive the transition,
  * `ops` — the result operations that trigger it (`added` / `updated` / `deleted`),
  * `previous` — the allowed prior states (`[]` = initial entry, `["*"]` = any),
  * `key` — a Handlebars template (e.g. `{{orderId}}`) that extracts the entity key.

  When a result matches and the entity is in an allowed prior state, the entity
  transitions. The new state is persisted and pushed to the companion source.

* The **source** is registered under a configurable `sourceId`. It emits one node
  per entity:
  * `element_id` = the entity key,
  * label = `entityLabel` (default `EntityState`),
  * properties = the key field, `state`, `previousState`, `enteredAt`, plus the
    pass-through fields from the triggering query result row.

  Late-joining and downstream subscribers are bootstrapped from the durably
  persisted state, so the source survives restarts.

## IDs

The Drasi component graph requires globally-unique component ids, so the reaction
and the source use **different** ids. The reaction's `sourceId` configuration
names the companion source; downstream queries subscribe to that `sourceId`.

## Usage (static / embedded)

```rust
use drasi_reaction_state_machine::{StateMachineBuilder, config::{StateDef, EnterCondition, Op}};

let (reaction, source) = StateMachineBuilder::new("order-state")
    .with_source_id("order-state-source")
    .with_entity_label("OrderState")
    .with_key_field("orderId")
    .with_state(StateDef {
        id: "NEW".to_string(),
        enter: vec![EnterCondition {
            query: "draft-orders".to_string(),
            previous: vec![],
            key: "{{orderId}}".to_string(),
            ops: vec![Op::Added],
        }],
    })
    // ... more states ...
    .build_pair()?;

// Both must be added to the same DrasiLib instance:
//   drasi.add_reaction(reaction).await?;
//   drasi.add_source(source).await?;
# Ok::<(), anyhow::Error>(())
```

A **durable** `StateStoreProvider` (e.g. `drasi-state-store-redb`) must be
configured on the `DrasiLib` instance — the reaction reports `is_durable() = true`.

## Configuration (declarative / YAML)

The crate exports both a `state-machine` **reaction** descriptor and a
`state-machine` **source** descriptor (behind the `dynamic-plugin` feature).
Declare the source under the id referenced by the reaction's `sourceId`:

```yaml
sources:
  - kind: state-machine
    id: order-state-source

reactions:
  - kind: state-machine
    id: order-state
    properties:
      sourceId: order-state-source
      entityLabel: OrderState
      keyField: orderId
      states:
        - id: NEW
          enter:
            - query: draft-orders
              previous: []
              key: "{{orderId}}"
              ops: [added]
        - id: CONFIRMED
          enter:
            - query: confirmed-orders
              previous: [NEW]
              key: "{{orderId}}"
              ops: [added]
        # ... PAID, PICKED, SHIPPED, DELIVERED ...
```

## Example

See [`examples/lib/orders-state-machine`](../../../examples/lib/orders-state-machine)
for a complete, self-driving order-lifecycle demo using a Postgres source,
stored-procedure reactions to advance orders, and the dashboard reaction to
visualize the live state.
