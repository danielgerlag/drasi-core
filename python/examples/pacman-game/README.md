# Pac-Man — Drasi Edition

A classic Pac-Man game where **game events are driven by Drasi continuous queries**.

Instead of collision checks in the game loop, game entities (Pacman, Ghosts, Power Pellets, Dots) are modeled as nodes in an `ApplicationSource`. Drasi continuous queries detect events in real time:

| Query | Detects | Effect |
|-------|---------|--------|
| `ghost-catches-pacman` | Ghost at same cell as Pacman (normal mode) | Player loses a life |
| `pacman-eats-pellet` | Pacman steps on a power pellet | Pacman becomes powered up |
| `pacman-catches-ghost` | Pacman at same cell as Ghost (powered mode) | Ghost is eaten |
| `dots-eaten` | Dots consumed by Pacman | Score tracking |

## Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  PyGame UI  │────→│ ApplicationSource│────→│  Drasi Queries  │
│  (game.py)  │     │   (game state)   │     │ (ghost-catches,  │
│             │←────│                  │←────│  pellet-eats,   │
│             │     │                  │     │  pacman-catches) │
└─────────────┘     └──────────────────┘     └─────────────────┘
      ↑                                              │
      │              ┌──────────────────┐             │
      └──────────────│   Reactions      │←────────────┘
                     │ (event streams)  │
                     └──────────────────┘
```

Each game tick:
1. Player input moves Pacman
2. Ghost AI moves ghosts
3. All positions are sent to the `ApplicationSource` as node updates
4. Drasi queries evaluate the new state continuously
5. `ApplicationReaction` streams fire events (collision, power-up, ghost eaten)
6. Background workers push events into the game state
7. Game loop processes events and updates the display

## How the Queries Work

The queries use a denormalized approach — each Ghost node carries both its own position AND Pacman's position/state. This allows single-node `MATCH` patterns that work with Drasi's streaming engine:

```cypher
-- Ghost catches Pacman (same cell, Pacman not powered)
MATCH (g:Ghost)
WHERE g.x = g.pacman_x AND g.y = g.pacman_y
  AND g.pacman_powered = 'false' AND g.alive = 'true'
RETURN g.id AS ghost_id, g.x AS x, g.y AS y

-- Pacman catches Ghost (same cell, Pacman IS powered)  
MATCH (g:Ghost)
WHERE g.x = g.pacman_x AND g.y = g.pacman_y
  AND g.pacman_powered = 'true' AND g.alive = 'true'
RETURN g.id AS ghost_id, g.x AS x, g.y AS y
```

## Prerequisites

- Python 3.11+
- [uv](https://docs.astral.sh/uv/) package manager
- A display (pygame requires a window — cannot run headless)

## Setup

```bash
cd python/
./examples/pacman-game/setup.sh
```

## Running

```bash
cd python/
uv run python examples/pacman-game/game.py
```

## Controls

| Key | Action |
|-----|--------|
| ↑ ↓ ← → | Move Pacman |
| Space | Restart after game over |
| Escape | Quit |

## Gameplay

- **Dots** (small white): +10 points each
- **Power Pellets** (large orange, pulsing): +50 points, Pacman turns green and can eat ghosts
- **Eating a Ghost**: +200 points, ghost respawns after a short delay
- **Lose a Life**: When a ghost catches Pacman (not powered up)
- **Win**: Eat all dots and power pellets
- **Game Over**: Lose all 3 lives

## Teardown

```bash
./examples/pacman-game/teardown.sh
```
