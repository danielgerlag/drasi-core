# Changelog

All notable changes to the `drasi-reaction-state-machine` crate are documented here.

## 0.1.0

### Added

- Initial release of the **state machine** component — a paired Drasi reaction and
  source.
- `StateMachineReaction`: subscribes to input queries and maps their results to
  per-entity state transitions guarded by `previous` states and result `ops`,
  using a Handlebars `key` template to extract the entity key from each row.
- `StateMachineSource`: exposes each entity's current state as a graph node
  (`element_id` = entity key, configurable label, properties `state`,
  `previousState`, `enteredAt`, plus pass-through query-row fields).
- Durable entity state persisted in the configured `StateStoreProvider`; reloaded
  on restart and used to bootstrap late-joining downstream subscribers.
- `StateMachineBuilder` for ergonomic static (in-process) construction of the
  linked reaction + source pair.
- Dynamic plugin descriptors (`state-machine` reaction and source kinds) exported
  behind the `dynamic-plugin` feature.
