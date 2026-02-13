# Pac-Man — Drasi Edition (Web UI)

A classic Pac-Man game where **game events are driven by Drasi continuous queries**,
rendered in the browser using HTML5 Canvas with real-time SSE updates.

Game entities (Pacman, Ghosts, PowerPellets, Dots) are modeled as nodes in an
`ApplicationSource`. Drasi continuous queries detect collisions and state changes,
and the **SSE Reaction** streams results directly to the browser:

| Query | Detects | Effect |
|-------|---------|--------|
| `ghost-catches-pacman` | Ghost at same cell as Pacman (normal mode) | Player loses a life |
| `pacman-eats-pellet` | Pacman steps on a power pellet | Pacman becomes powered up |
| `pacman-catches-ghost` | Pacman at same cell as Ghost (powered mode) | Ghost is eaten |
| `dots-eaten` | Dots consumed by Pacman | Score tracking |

## Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Browser    │────→│ Flask Server     │────→│  Drasi Engine   │
│  (Canvas)   │ POST│ (game logic,     │     │  (queries,      │
│             │     │  ghost AI)       │     │   SSE reaction) │
│             │←────│                  │     │        │        │
│             │ GET │                  │     │        │        │
│             │◄════╪══════════════════╪═════╪════════╝        │
│             │ SSE │                  │     │  port 8082      │
└─────────────┘     └──────────────────┘     └─────────────────┘
```

1. **Flask** (port 5050) serves the HTML5 Canvas game page and REST API
2. **SSE Reaction** (port 8082) streams Drasi query results to the browser
3. Ghost AI and movement run server-side in a background thread
4. Player input is sent via POST, game state is polled via GET
5. Drasi events (collisions, power-ups) arrive as SSE and trigger visual effects

## How the Queries Work

The game uses a proper graph model with relationships for spatial queries:

```
(:Pacman)-[:IS_AT]->(:Cell)<-[:IS_AT]-(:Ghost)
                    (:Cell)<-[:ON]-(:Pellet)
```

**Nodes:**
- `Pacman` — `{powered}` (power-up state)
- `Ghost` — `{alive}` (alive/dead state)
- `Cell` — `{x, y}` (walkable maze cells)
- `Pellet` — power pellets
- `Dot` — `{x, y, eaten}` (score dots)

**Relationships:**
- `[:IS_AT]` — Pacman/Ghost → Cell (current position, moved each tick)
- `[:ON]` — Pellet → Cell (pellet location, deleted when eaten)

```cypher
-- Ghost catches Pacman: both IS_AT the same Cell, Pacman not powered
MATCH (g:Ghost)-[:IS_AT]->(c:Cell)<-[:IS_AT]-(p:Pacman)
WHERE p.powered = 'false' AND g.alive = 'true'
RETURN g.id AS ghost_id, c.x AS x, c.y AS y

-- Pacman eats pellet: Pacman IS_AT a Cell that has a Pellet ON it
MATCH (p:Pacman)-[:IS_AT]->(c:Cell)<-[:ON]-(pel:Pellet)
RETURN pel.id AS pellet_id, c.x AS x, c.y AS y

-- Pacman catches ghost: same as ghost-catches but Pacman IS powered
MATCH (g:Ghost)-[:IS_AT]->(c:Cell)<-[:IS_AT]-(p:Pacman)
WHERE p.powered = 'true' AND g.alive = 'true'
RETURN g.id AS ghost_id, c.x AS x, c.y AS y
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
