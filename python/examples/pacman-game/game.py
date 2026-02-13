"""Pac-Man game powered by Drasi continuous queries.

Game entities (Pacman, Ghosts, PowerPellets, Dots) are nodes in an
ApplicationSource.  Drasi queries detect collisions and state changes:

  ghost-catches-pacman : Ghost at same cell as Pacman while Pacman is normal
  pacman-eats-pellet   : Pacman at same cell as a PowerPellet
  pacman-catches-ghost : Pacman at same cell as a Ghost while Pacman is powered

ApplicationReactions consume those queries and trigger game events (death,
power-up, ghost-eaten) in real time.
"""

import asyncio
import math
import random
import sys
import threading
import time
import uuid

import pygame

from drasi_lib import DrasiLibBuilder, Query
from drasi_source_application import PyApplicationSource, PyPropertyMapBuilder
from drasi_reaction_application import PyApplicationReaction


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

TILE = 24
COLS, ROWS = 21, 21
WIDTH, HEIGHT = COLS * TILE, ROWS * TILE + 60  # extra for HUD
FPS = 10

BLACK = (0, 0, 0)
BLUE = (33, 33, 222)
YELLOW = (255, 255, 0)
WHITE = (255, 255, 255)
RED = (255, 0, 0)
PINK = (255, 184, 255)
CYAN = (0, 255, 255)
ORANGE = (255, 184, 82)
GREEN = (0, 255, 0)
DARK_BLUE = (0, 0, 40)
PELLET_COLOR = (255, 184, 82)

GHOST_COLORS = {
    "blinky": RED,
    "pinky": PINK,
    "inky": CYAN,
    "clyde": ORANGE,
}

# Simple maze layout: 1=wall, 0=path, 2=power pellet
MAZE = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,0,1],
    [1,2,1,1,0,1,1,1,0,0,0,0,0,1,1,1,0,1,1,2,1],
    [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,0,1],
    [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],
    [1,1,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,1,1,0,0,0,1,1,0,1,0,1,1,1,1],
    [0,0,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0],
    [1,1,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,0,0,0,0,0,0,0,0,1,0,1,1,1,1],
    [1,1,1,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,1,1,1],
    [1,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1],
    [1,0,1,1,0,1,1,1,0,0,1,0,0,1,1,1,0,1,1,0,1],
    [1,2,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,2,1],
    [1,1,0,1,0,1,0,1,1,1,1,1,1,1,0,1,0,1,0,1,1],
    [1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],
    [1,0,1,1,1,1,1,1,0,0,1,0,0,1,1,1,1,1,1,0,1],
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
]

GHOST_START = {
    "blinky": (10, 9),
    "pinky":  (9, 10),
    "inky":   (10, 10),
    "clyde":  (11, 10),
}
PACMAN_START = (10, 16)

POWER_DURATION = 50  # ticks


# ---------------------------------------------------------------------------
# Async helper
# ---------------------------------------------------------------------------

_bg_loop: asyncio.AbstractEventLoop | None = None
_bg_thread: threading.Thread | None = None


def _get_bg_loop():
    global _bg_loop, _bg_thread
    if _bg_loop is None or _bg_loop.is_closed():
        _bg_loop = asyncio.new_event_loop()
        _bg_thread = threading.Thread(target=_bg_loop.run_forever, daemon=True)
        _bg_thread.start()
    return _bg_loop


def run_async(coro):
    loop = _get_bg_loop()
    fut = asyncio.run_coroutine_threadsafe(coro, loop)
    return fut.result(timeout=5)


def fire_and_forget(coro):
    loop = _get_bg_loop()
    asyncio.run_coroutine_threadsafe(coro, loop)


# ---------------------------------------------------------------------------
# Drasi wiring
# ---------------------------------------------------------------------------

def build_drasi():
    source = PyApplicationSource("game")
    handle = source.get_handle()

    def make_rx(name, queries):
        b = PyApplicationReaction.builder(name)
        for q in queries:
            b.with_query(q)
        b.with_auto_start(True)
        return b.build()

    ghost_catch_rx, ghost_catch_h = make_rx("ghost-catch-rx", ["ghost-catches-pacman"])
    pellet_rx, pellet_h = make_rx("pellet-rx", ["pacman-eats-pellet"])
    pacman_catch_rx, pacman_catch_h = make_rx("pacman-catch-rx", ["pacman-catches-ghost"])
    dots_rx, dots_h = make_rx("dots-rx", ["dots-eaten"])

    def cypher(qid, text):
        q = Query.cypher(qid)
        q.query(text)
        q.from_source("game")
        q.auto_start(True)
        return q.build()

    # Ghost catches pacman: ghost & pacman at same cell, pacman not powered
    q_ghost_catch = cypher(
        "ghost-catches-pacman",
        "MATCH (g:Ghost) "
        "WHERE g.x = g.pacman_x AND g.y = g.pacman_y "
        "AND g.pacman_powered = 'false' AND g.alive = 'true' "
        "RETURN g.id AS ghost_id, g.x AS x, g.y AS y",
    )

    # Pacman eats power pellet: pacman at same cell as pellet
    q_pellet = cypher(
        "pacman-eats-pellet",
        "MATCH (p:Pacman) "
        "WHERE p.on_pellet = 'true' "
        "RETURN p.id AS pacman_id, p.x AS x, p.y AS y",
    )

    # Pacman catches ghost while powered
    q_pacman_catch = cypher(
        "pacman-catches-ghost",
        "MATCH (g:Ghost) "
        "WHERE g.x = g.pacman_x AND g.y = g.pacman_y "
        "AND g.pacman_powered = 'true' AND g.alive = 'true' "
        "RETURN g.id AS ghost_id, g.x AS x, g.y AS y",
    )

    # Track dots eaten
    q_dots = cypher(
        "dots-eaten",
        "MATCH (d:Dot) WHERE d.eaten = 'true' "
        "RETURN d.id AS dot_id",
    )

    builder = DrasiLibBuilder()
    builder.with_id("pacman")
    builder.with_source(source.into_source_wrapper())

    for qcfg in [q_ghost_catch, q_pellet, q_pacman_catch, q_dots]:
        builder.with_query(qcfg)
    for r in [ghost_catch_rx, pellet_rx, pacman_catch_rx, dots_rx]:
        builder.with_reaction(r.into_reaction_wrapper())

    async def _init():
        lib = await builder.build()
        await lib.start()
        s1 = await ghost_catch_h.as_stream()
        s2 = await pellet_h.as_stream()
        s3 = await pacman_catch_h.as_stream()
        s4 = await dots_h.as_stream()
        return lib, s1, s2, s3, s4

    lib, s_ghost, s_pellet, s_pacman, s_dots = run_async(_init())
    return lib, handle, s_ghost, s_pellet, s_pacman, s_dots


# ---------------------------------------------------------------------------
# Game state
# ---------------------------------------------------------------------------

class GameState:
    def __init__(self, handle):
        self.handle = handle
        self.pacman_pos = list(PACMAN_START)
        self.pacman_dir = (0, 0)
        self.next_dir = (0, 0)
        self.powered = False
        self.power_timer = 0
        self.lives = 3
        self.score = 0
        self.game_over = False
        self.win = False
        self.ghosts = {}
        for name, pos in GHOST_START.items():
            self.ghosts[name] = {
                "pos": list(pos),
                "alive": True,
                "dir": random.choice([(0, -1), (0, 1), (-1, 0), (1, 0)]),
                "respawn_timer": 0,
            }
        # Dots
        self.dots = set()
        self.pellets = set()
        for r in range(ROWS):
            for c in range(COLS):
                if MAZE[r][c] == 0:
                    self.dots.add((c, r))
                elif MAZE[r][c] == 2:
                    self.pellets.add((c, r))
        self.total_dots = len(self.dots) + len(self.pellets)
        # Events from Drasi
        self.events = []
        self.event_lock = threading.Lock()

    def push_event(self, etype, data=None):
        with self.event_lock:
            self.events.append((etype, data or {}))

    def pop_events(self):
        with self.event_lock:
            evts = list(self.events)
            self.events.clear()
        return evts


def can_move(x, y):
    """Check if (x, y) is a valid path cell."""
    # Wrap horizontally for tunnel
    nx = x % COLS
    ny = y % ROWS
    if 0 <= ny < ROWS and 0 <= nx < COLS:
        return MAZE[ny][nx] != 1
    return False


# ---------------------------------------------------------------------------
# Send state to Drasi
# ---------------------------------------------------------------------------

def send_pacman_state(gs: GameState):
    """Update the Pacman node in Drasi with current position and state."""
    p = PyPropertyMapBuilder()
    p.with_string("id", "pacman")
    p.with_integer("x", gs.pacman_pos[0])
    p.with_integer("y", gs.pacman_pos[1])
    p.with_string("powered", str(gs.powered).lower())
    on_pellet = tuple(gs.pacman_pos) in gs.pellets
    p.with_string("on_pellet", str(on_pellet).lower())

    async def _send():
        await gs.handle.send_node_update("pacman", ["Pacman"], p.build())
    fire_and_forget(_send())


def send_ghost_state(gs: GameState, name: str):
    """Update a Ghost node with current position and pacman's position."""
    g = gs.ghosts[name]
    p = PyPropertyMapBuilder()
    p.with_string("id", name)
    p.with_integer("x", g["pos"][0])
    p.with_integer("y", g["pos"][1])
    p.with_string("alive", str(g["alive"]).lower())
    p.with_integer("pacman_x", gs.pacman_pos[0])
    p.with_integer("pacman_y", gs.pacman_pos[1])
    p.with_string("pacman_powered", str(gs.powered).lower())

    async def _send():
        await gs.handle.send_node_update(name, ["Ghost"], p.build())
    fire_and_forget(_send())


def send_dot_eaten(gs: GameState, x, y):
    """Mark a dot as eaten."""
    dot_id = f"dot-{x}-{y}"
    p = PyPropertyMapBuilder()
    p.with_string("id", dot_id)
    p.with_integer("x", x)
    p.with_integer("y", y)
    p.with_string("eaten", "true")

    async def _send():
        await gs.handle.send_node_update(dot_id, ["Dot"], p.build())
    fire_and_forget(_send())


def init_drasi_state(gs: GameState):
    """Insert initial nodes for Pacman, Ghosts, and Dots."""
    async def _init():
        # Pacman
        p = PyPropertyMapBuilder()
        p.with_string("id", "pacman")
        p.with_integer("x", gs.pacman_pos[0])
        p.with_integer("y", gs.pacman_pos[1])
        p.with_string("powered", "false")
        p.with_string("on_pellet", "false")
        await gs.handle.send_node_insert("pacman", ["Pacman"], p.build())

        # Ghosts
        for name, info in gs.ghosts.items():
            g = PyPropertyMapBuilder()
            g.with_string("id", name)
            g.with_integer("x", info["pos"][0])
            g.with_integer("y", info["pos"][1])
            g.with_string("alive", "true")
            g.with_integer("pacman_x", gs.pacman_pos[0])
            g.with_integer("pacman_y", gs.pacman_pos[1])
            g.with_string("pacman_powered", "false")
            await gs.handle.send_node_insert(name, ["Ghost"], g.build())

        # Dots
        for (x, y) in gs.dots | gs.pellets:
            d = PyPropertyMapBuilder()
            dot_id = f"dot-{x}-{y}"
            d.with_string("id", dot_id)
            d.with_integer("x", x)
            d.with_integer("y", y)
            d.with_string("eaten", "false")
            await gs.handle.send_node_insert(dot_id, ["Dot"], d.build())

    run_async(_init())


# ---------------------------------------------------------------------------
# Reaction stream workers
# ---------------------------------------------------------------------------

def start_reaction_workers(gs, s_ghost, s_pellet, s_pacman, s_dots):
    """Background workers that consume Drasi reaction streams."""

    async def _ghost_worker():
        if s_ghost is None:
            return
        async for result in s_ghost:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    data = r.get("data", {})
                    gs.push_event("ghost_catches_pacman", data)

    async def _pellet_worker():
        if s_pellet is None:
            return
        async for result in s_pellet:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    data = r.get("data", {})
                    gs.push_event("pacman_eats_pellet", data)

    async def _pacman_worker():
        if s_pacman is None:
            return
        async for result in s_pacman:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    data = r.get("data", {})
                    gs.push_event("pacman_catches_ghost", data)

    async def _dots_worker():
        if s_dots is None:
            return
        async for result in s_dots:
            for r in result.get("results", []):
                if r.get("type") == "ADD":
                    gs.push_event("dot_eaten", r.get("data", {}))

    fire_and_forget(_ghost_worker())
    fire_and_forget(_pellet_worker())
    fire_and_forget(_pacman_worker())
    fire_and_forget(_dots_worker())


# ---------------------------------------------------------------------------
# Ghost AI
# ---------------------------------------------------------------------------

def move_ghost(gs, name):
    g = gs.ghosts[name]
    if not g["alive"]:
        g["respawn_timer"] -= 1
        if g["respawn_timer"] <= 0:
            g["pos"] = list(GHOST_START[name])
            g["alive"] = True
        return

    px, py = gs.pacman_pos
    gx, gy = g["pos"]
    dx, dy = g["dir"]

    # Possible directions (no reversal)
    reverse = (-dx, -dy)
    choices = []
    for d in [(0, -1), (0, 1), (-1, 0), (1, 0)]:
        if d == reverse:
            continue
        nx, ny = (gx + d[0]) % COLS, (gy + d[1]) % ROWS
        if can_move(nx, ny):
            choices.append(d)

    if not choices:
        # Dead end — reverse
        choices = [reverse] if can_move((gx + reverse[0]) % COLS, (gy + reverse[1]) % ROWS) else [(0, 0)]

    if gs.powered:
        # Run away from pacman
        def away_dist(d):
            nx, ny = gx + d[0], gy + d[1]
            return -((nx - px) ** 2 + (ny - py) ** 2)
        choices.sort(key=away_dist)
    else:
        # Chase pacman (with some randomness)
        if random.random() < 0.3:
            random.shuffle(choices)
        else:
            def chase_dist(d):
                nx, ny = gx + d[0], gy + d[1]
                return (nx - px) ** 2 + (ny - py) ** 2
            choices.sort(key=chase_dist)

    g["dir"] = choices[0]
    nx = (gx + g["dir"][0]) % COLS
    ny = (gy + g["dir"][1]) % ROWS
    g["pos"] = [nx, ny]


# ---------------------------------------------------------------------------
# Drawing
# ---------------------------------------------------------------------------

def draw(screen, gs, font):
    screen.fill(BLACK)

    # Maze
    for r in range(ROWS):
        for c in range(COLS):
            rect = pygame.Rect(c * TILE, r * TILE, TILE, TILE)
            if MAZE[r][c] == 1:
                pygame.draw.rect(screen, BLUE, rect)

    # Dots
    for (x, y) in gs.dots:
        cx = x * TILE + TILE // 2
        cy = y * TILE + TILE // 2
        pygame.draw.circle(screen, WHITE, (cx, cy), 2)

    # Power pellets (animate)
    pulse = abs(math.sin(time.time() * 4))
    for (x, y) in gs.pellets:
        cx = x * TILE + TILE // 2
        cy = y * TILE + TILE // 2
        radius = int(4 + 2 * pulse)
        pygame.draw.circle(screen, PELLET_COLOR, (cx, cy), radius)

    # Ghosts
    for name, g in gs.ghosts.items():
        if not g["alive"]:
            continue
        gx = g["pos"][0] * TILE + TILE // 2
        gy = g["pos"][1] * TILE + TILE // 2
        color = (50, 50, 255) if gs.powered else GHOST_COLORS[name]
        # Body
        pygame.draw.circle(screen, color, (gx, gy - 2), TILE // 2 - 2)
        pygame.draw.rect(screen, color, (gx - TILE // 2 + 2, gy - 2, TILE - 4, TILE // 2))
        # Eyes
        pygame.draw.circle(screen, WHITE, (gx - 3, gy - 4), 3)
        pygame.draw.circle(screen, WHITE, (gx + 3, gy - 4), 3)
        pygame.draw.circle(screen, BLACK, (gx - 3, gy - 4), 1)
        pygame.draw.circle(screen, BLACK, (gx + 3, gy - 4), 1)

    # Pacman
    px = gs.pacman_pos[0] * TILE + TILE // 2
    py = gs.pacman_pos[1] * TILE + TILE // 2
    color = GREEN if gs.powered else YELLOW

    # Animated mouth
    mouth_angle = abs(math.sin(time.time() * 8)) * 45
    dx, dy = gs.pacman_dir if gs.pacman_dir != (0, 0) else (1, 0)
    start_angle = math.atan2(-dy, dx)
    start_deg = math.degrees(start_angle)
    pygame.draw.arc(screen, color,
                    (px - TILE // 2 + 2, py - TILE // 2 + 2, TILE - 4, TILE - 4),
                    math.radians(start_deg + mouth_angle),
                    math.radians(start_deg + 360 - mouth_angle), TILE // 2 - 2)
    # Filled circle with mouth cut
    pygame.draw.circle(screen, color, (px, py), TILE // 2 - 2)
    # Cut mouth
    mouth_len = TILE
    end1 = (px + int(mouth_len * math.cos(math.radians(start_deg + mouth_angle))),
            py - int(mouth_len * math.sin(math.radians(start_deg + mouth_angle))))
    end2 = (px + int(mouth_len * math.cos(math.radians(start_deg - mouth_angle))),
            py - int(mouth_len * math.sin(math.radians(start_deg - mouth_angle))))
    pygame.draw.polygon(screen, BLACK, [(px, py), end1, end2])

    # HUD
    hud_y = ROWS * TILE + 5
    score_surf = font.render(f"Score: {gs.score}", True, WHITE)
    lives_surf = font.render(f"Lives: {gs.lives}", True, WHITE)
    screen.blit(score_surf, (10, hud_y))
    screen.blit(lives_surf, (WIDTH - 120, hud_y))

    if gs.powered:
        power_surf = font.render(f"POWER: {gs.power_timer}", True, GREEN)
        screen.blit(power_surf, (WIDTH // 2 - 40, hud_y))

    if gs.game_over:
        msg = "YOU WIN!" if gs.win else "GAME OVER"
        color = GREEN if gs.win else RED
        go_surf = font.render(msg, True, color)
        rect = go_surf.get_rect(center=(WIDTH // 2, HEIGHT // 2))
        pygame.draw.rect(screen, BLACK, rect.inflate(20, 10))
        screen.blit(go_surf, rect)
        restart_surf = font.render("Press SPACE to restart", True, WHITE)
        screen.blit(restart_surf, restart_surf.get_rect(center=(WIDTH // 2, HEIGHT // 2 + 30)))

    pygame.display.flip()


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

def main():
    pygame.init()
    screen = pygame.display.set_mode((WIDTH, HEIGHT))
    pygame.display.set_caption("Pac-Man — Drasi Edition")
    clock = pygame.time.Clock()
    font = pygame.font.SysFont("monospace", 20, bold=True)

    print("Initializing Drasi...")
    lib, handle, s_ghost, s_pellet, s_pacman, s_dots = build_drasi()

    def new_game():
        gs = GameState(handle)
        init_drasi_state(gs)
        start_reaction_workers(gs, s_ghost, s_pellet, s_pacman, s_dots)
        time.sleep(0.3)  # let Drasi process initial state
        return gs

    gs = new_game()
    print("Game started!")

    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            elif event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                elif event.key == pygame.K_SPACE and gs.game_over:
                    gs = new_game()
                elif event.key == pygame.K_UP:
                    gs.next_dir = (0, -1)
                elif event.key == pygame.K_DOWN:
                    gs.next_dir = (0, 1)
                elif event.key == pygame.K_LEFT:
                    gs.next_dir = (-1, 0)
                elif event.key == pygame.K_RIGHT:
                    gs.next_dir = (1, 0)

        if not gs.game_over:
            # ── Move Pacman ─────────────────────────────────────────────
            nx = (gs.pacman_pos[0] + gs.next_dir[0]) % COLS
            ny = (gs.pacman_pos[1] + gs.next_dir[1]) % ROWS
            if can_move(nx, ny):
                gs.pacman_dir = gs.next_dir

            nx = (gs.pacman_pos[0] + gs.pacman_dir[0]) % COLS
            ny = (gs.pacman_pos[1] + gs.pacman_dir[1]) % ROWS
            if can_move(nx, ny):
                gs.pacman_pos = [nx, ny]

            # Eat dots locally (for immediate feedback)
            pos_tuple = tuple(gs.pacman_pos)
            if pos_tuple in gs.dots:
                gs.dots.discard(pos_tuple)
                gs.score += 10
                send_dot_eaten(gs, pos_tuple[0], pos_tuple[1])

            # Power timer
            if gs.powered:
                gs.power_timer -= 1
                if gs.power_timer <= 0:
                    gs.powered = False

            # ── Move Ghosts ─────────────────────────────────────────────
            for name in gs.ghosts:
                move_ghost(gs, name)

            # ── Send state to Drasi ─────────────────────────────────────
            send_pacman_state(gs)
            for name in gs.ghosts:
                send_ghost_state(gs, name)

            # ── Process Drasi events ────────────────────────────────────
            for etype, data in gs.pop_events():
                if etype == "ghost_catches_pacman" and not gs.powered:
                    gs.lives -= 1
                    if gs.lives <= 0:
                        gs.game_over = True
                    else:
                        # Reset positions
                        gs.pacman_pos = list(PACMAN_START)
                        gs.pacman_dir = (0, 0)
                        gs.next_dir = (0, 0)
                        for name, pos in GHOST_START.items():
                            gs.ghosts[name]["pos"] = list(pos)
                            gs.ghosts[name]["alive"] = True

                elif etype == "pacman_eats_pellet":
                    pos_tuple = tuple(gs.pacman_pos)
                    if pos_tuple in gs.pellets:
                        gs.pellets.discard(pos_tuple)
                        gs.score += 50
                        gs.powered = True
                        gs.power_timer = POWER_DURATION

                elif etype == "pacman_catches_ghost" and gs.powered:
                    ghost_id = data.get("ghost_id", "")
                    if ghost_id in gs.ghosts and gs.ghosts[ghost_id]["alive"]:
                        gs.ghosts[ghost_id]["alive"] = False
                        gs.ghosts[ghost_id]["respawn_timer"] = 20
                        gs.score += 200

            # Win condition
            if not gs.dots and not gs.pellets:
                gs.game_over = True
                gs.win = True

        draw(screen, gs, font)
        clock.tick(FPS)

    pygame.quit()
    sys.exit()


if __name__ == "__main__":
    main()
