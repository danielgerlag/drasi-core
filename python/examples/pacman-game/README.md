# Pac-Man — Drasi Edition (Web UI)

A classic Pac-Man game where **game events are driven by Drasi continuous queries**,
rendered in the browser using HTML5 Canvas with real-time SSE updates.

## Architecture

```
                        ┌──────────────────────────────────────────┐
                        │              Drasi Engine                │
┌─────────────┐         │                                          │
│  Browser    │─── POST ──→ HttpSource webhook (port 8083)         │
│  (Canvas)   │         │     maps {dir:"up"} → Direction node     │
│             │         │         │                                │
│             │         │         ▼                                │
│             │         │   direction-changed query                │
│             │         │         │                                │
│             │         │         ▼                                │
│             │         │   ApplicationReaction → game tick loop   │
│             │         │                                          │
│             │◄══ SSE ═══ SSE Reaction (port 8082)                │
│             │         │   streams collision/pellet events        │
│             │         │                                          │
│             │◄══ SSE ═══ Flask /stream (port 5050)               │
│             │         │   streams game state snapshots           │
└─────────────┘         └──────────────────────────────────────────┘
```

### Data flow

1. **Player input** → Browser sends `POST {dir: "up"}` to the **HttpSource webhook** (port 8083)
2. **Webhook mapping** → The HttpSource maps the request to a `Direction` node insert with `{dx, dy}` properties
3. **Continuous query** → The `direction-changed` query (on the `input` source) detects the change and emits the new direction
4. **Game logic** → An `ApplicationReaction` worker picks up the direction and sets `next_dir` on the game state
5. **Game tick** → A background thread moves Pacman/ghosts, updates IS_AT relationships via the `ApplicationSource`
6. **Collision queries** → Drasi evaluates 4 queries on the `game` source to detect collisions, pellet pickups, etc.
7. **Browser rendering** → The SSE Reaction streams query results to the browser; Flask `/stream` pushes game state snapshots

### Sources

| Source | Type | Purpose |
|--------|------|---------|
| `game` | `ApplicationSource` | Game entities: Pacman, Ghosts, Cells, Pellets, Dots |
| `input` | `HttpSource` (webhook) | Player direction input from the browser |

### Queries

| Query | Source | Detects |
|-------|--------|---------|
| `ghost-catches-pacman` | `game` | Ghost and Pacman on same Cell (normal mode) → lose a life |
| `pacman-eats-pellet` | `game` | Pacman on a Cell with a Pellet → power up |
| `pacman-catches-ghost` | `game` | Pacman and Ghost on same Cell (powered mode) → eat ghost |
| `dots-eaten` | `game` | Dot marked as eaten → score tracking |
| `direction-changed` | `input` | Direction node updated → change Pacman direction |

### Reactions

| Reaction | Type | Purpose |
|----------|------|---------|
| `game-sse` | `SseReaction` | Streams query events to browser (visual effects) |
| `ghost-catch-rx` | `ApplicationReaction` | Server-side: deduct lives |
| `pellet-rx` | `ApplicationReaction` | Server-side: activate power mode |
| `pacman-catch-rx` | `ApplicationReaction` | Server-side: score ghost kills |
| `dots-rx` | `ApplicationReaction` | Server-side: track dot consumption |
| `dir-rx` | `ApplicationReaction` | Server-side: update Pacman direction |

## Graph Model

```
(:Pacman {powered}) ─[:IS_AT]→ (:Cell {x, y}) ←[:IS_AT]─ (:Ghost {alive})
                               (:Cell {x, y}) ←[:ON]──── (:Pellet)
(:Dot {x, y, eaten})
(:Direction {dx, dy})  ← on the "input" source
```

**Nodes:**
- `Cell` — every walkable maze tile (~150 nodes)
- `Pacman` — `{powered}` (power-up state)
- `Ghost` — `{alive}` (alive/dead state, one per ghost: blinky, pinky, inky, clyde)
- `Pellet` — power pellets (4 total)
- `Dot` — `{x, y, eaten}` (score dots)
- `Direction` — `{dx, dy}` (player input, lives on the `input` source)

**Relationships:**
- `[:IS_AT]` — Pacman/Ghost → Cell (current position, delete + re-insert each tick)
- `[:ON]` — Pellet → Cell (pellet location, deleted when eaten)

### Example Cypher Queries

```cypher
-- Ghost catches Pacman: both IS_AT the same Cell, Pacman not powered
MATCH
  (g:Ghost)-[:IS_AT]->(c:Cell)<-[:IS_AT]-(p:Pacman)
WHERE
  p.powered = 'false'
  AND g.alive = 'true'
RETURN
  g.id AS ghost_id,
  c.x AS x,
  c.y AS y

-- Pacman eats pellet: Pacman IS_AT a Cell that has a Pellet ON it
MATCH
  (p:Pacman)-[:IS_AT]->(c:Cell)<-[:ON]-(pel:Pellet)
RETURN
  pel.id AS pellet_id,
  c.x AS x,
  c.y AS y

-- Direction changed (on the input source)
MATCH
  (d:Direction)
RETURN
  d.dx AS dx,
  d.dy AS dy
```

## Webhook Configuration

Player input uses the typed `HttpSourceConfig` API to map keyboard POST requests
to graph changes without any custom Flask endpoint:

```python
HttpSourceConfig(
    "0.0.0.0", 8083,
    webhooks=WebhookConfig(
        [WebhookRoute("/input", [
            WebhookMapping(
                "node",
                ElementTemplate("direction", ["Direction"], {"dx": 0, "dy": -1}),
                when=MappingCondition(field="payload.dir", equals="up"),
                operation="insert",
            ),
            # ... similar for down, left, right
        ])],
        cors=CorsConfig(allow_origins=["*"]),
    ),
)
```

## Prerequisites

- Python 3.11+
- [uv](https://docs.astral.sh/uv/) package manager
- A web browser

## Running

```bash
cd python/
uv run python examples/pacman-game/game_web.py
```

Then open **http://localhost:5050** in your browser.

## Controls

| Key | Action |
|-----|--------|
| ↑ ↓ ← → or W A S D | Move Pacman |
| Space / Enter | Restart after game over |

## Gameplay

- **Dots** (white): +10 points each
- **Power Pellets** (orange, pulsing): +50 points, Pacman turns green and can eat ghosts
- **Eating a Ghost**: +200 points, ghost respawns after a short delay
- **Lose a Life**: When a ghost catches Pacman (not powered up)
- **Win**: Eat all dots and power pellets
- **Game Over**: Lose all 3 lives
