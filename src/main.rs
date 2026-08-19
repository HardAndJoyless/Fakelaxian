// Fakelaxian terminal: multi-color sprites, interpolated rendering, persistent high score.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    f32::consts::PI,
    io::{self, stdout, Write},
    path::PathBuf,
    sync::LazyLock,
    time::{Duration, Instant},
};

const FIXED_TIMESTEP: Duration = Duration::from_millis(33); // ~30 Hz game logic
const MAX_FRAME_TIME: Duration = Duration::from_millis(250);
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 Hz render cap
const EXTRA_LIFE_INTERVAL: u32 = 5_000;
const STICKY_INPUT: f32 = 0.15; // seconds to auto-release a held key when no event arrives
const MIN_COLS: u16 = 40;
const MIN_ROWS: u16 = 15;

// --- THEME & PALETTE SYSTEM ---
#[derive(Clone, Copy)]
struct Palette {
    name: &'static str,
    bg: Color,
    fg: Color,
    yellow: Color,
    gold: Color,
    orange: Color,
    red: Color,
    dark_red: Color,
    magenta: Color,
    purple: Color,
    cyan: Color,
    blue: Color,
    green: Color,
    gray: Color,
    dark_gray: Color,
}

fn hex_rgb(hex: u32) -> Color {
    Color::Rgb {
        r: ((hex >> 16) & 0xFF) as u8,
        g: ((hex >> 8) & 0xFF) as u8,
        b: (hex & 0xFF) as u8,
    }
}

static PALETTES: LazyLock<Vec<Palette>> = LazyLock::new(|| {
    vec![
        Palette {
            name: "Fakelaxian (Classic)",
            bg: Color::Rgb { r: 0, g: 0, b: 0 },
            fg: Color::Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            yellow: Color::Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
            gold: Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            },
            orange: Color::Rgb {
                r: 255,
                g: 140,
                b: 0,
            },
            red: Color::Rgb { r: 255, g: 0, b: 0 },
            dark_red: Color::Rgb { r: 139, g: 0, b: 0 },
            magenta: Color::Rgb {
                r: 255,
                g: 0,
                b: 255,
            },
            purple: Color::Rgb {
                r: 128,
                g: 0,
                b: 128,
            },
            cyan: Color::Rgb {
                r: 0,
                g: 255,
                b: 255,
            },
            blue: Color::Rgb {
                r: 20,
                g: 50,
                b: 255,
            },
            green: Color::Rgb { r: 0, g: 255, b: 0 },
            gray: Color::Rgb {
                r: 169,
                g: 169,
                b: 169,
            },
            dark_gray: Color::Rgb {
                r: 64,
                g: 64,
                b: 64,
            },
        },
        Palette {
            name: "Catppuccin Mocha",
            bg: hex_rgb(0x1e1e2e),
            fg: hex_rgb(0xcdd6f4),
            yellow: hex_rgb(0xf9e2af),
            gold: hex_rgb(0xf5e0dc),
            orange: hex_rgb(0xfab387),
            red: hex_rgb(0xf38ba8),
            dark_red: hex_rgb(0xeba0ac),
            magenta: hex_rgb(0xf5c2e7),
            purple: hex_rgb(0xcba6f7),
            cyan: hex_rgb(0x89dceb),
            blue: hex_rgb(0x89b4fa),
            green: hex_rgb(0xa6e3a1),
            gray: hex_rgb(0x9399b2),
            dark_gray: hex_rgb(0x585b70),
        },
        Palette {
            name: "Nord",
            bg: hex_rgb(0x2e3440),
            fg: hex_rgb(0xd8dee9),
            yellow: hex_rgb(0xebcb8b),
            gold: hex_rgb(0xd08770),
            orange: hex_rgb(0xd08770),
            red: hex_rgb(0xbf616a),
            dark_red: hex_rgb(0x8f4650),
            magenta: hex_rgb(0xb48ead),
            purple: hex_rgb(0xb48ead),
            cyan: hex_rgb(0x88c0d0),
            blue: hex_rgb(0x5e81ac),
            green: hex_rgb(0xa3be8c),
            gray: hex_rgb(0x4c566a),
            dark_gray: hex_rgb(0x3b4252),
        },
        Palette {
            name: "Dracula",
            bg: hex_rgb(0x282a36),
            fg: hex_rgb(0xf8f8f2),
            yellow: hex_rgb(0xf1fa8c),
            gold: hex_rgb(0xffb86c),
            orange: hex_rgb(0xffb86c),
            red: hex_rgb(0xff5555),
            dark_red: hex_rgb(0xbd2c40),
            magenta: hex_rgb(0xff79c6),
            purple: hex_rgb(0xbd93f9),
            cyan: hex_rgb(0x8be9fd),
            blue: hex_rgb(0x6272a4),
            green: hex_rgb(0x50fa7b),
            gray: hex_rgb(0x6272a4),
            dark_gray: hex_rgb(0x44475a),
        },
        Palette {
            name: "Gruvbox Dark",
            bg: hex_rgb(0x282828),
            fg: hex_rgb(0xebdbb2),
            yellow: hex_rgb(0xfabd2f),
            gold: hex_rgb(0xd79921),
            orange: hex_rgb(0xfe8019),
            red: hex_rgb(0xfb4934),
            dark_red: hex_rgb(0xcc241d),
            magenta: hex_rgb(0xd3869b),
            purple: hex_rgb(0xb16286),
            cyan: hex_rgb(0x8ec07c),
            blue: hex_rgb(0x83a598),
            green: hex_rgb(0xb8bb26),
            gray: hex_rgb(0x928374),
            dark_gray: hex_rgb(0x504945),
        },
        Palette {
            name: "Tokyo Night",
            bg: hex_rgb(0x1a1b26),
            fg: hex_rgb(0xc0caf5),
            yellow: hex_rgb(0xe0af68),
            gold: hex_rgb(0xff9e64),
            orange: hex_rgb(0xff9e64),
            red: hex_rgb(0xf7768e),
            dark_red: hex_rgb(0xdb4b4b),
            magenta: hex_rgb(0xbb9af7),
            purple: hex_rgb(0x9d7cd8),
            cyan: hex_rgb(0x7dcfff),
            blue: hex_rgb(0x7aa2f7),
            green: hex_rgb(0x9ece6a),
            gray: hex_rgb(0x565f89),
            dark_gray: hex_rgb(0x292e42),
        },
        Palette {
            name: "System Default",
            bg: Color::Reset,
            fg: Color::Reset,
            yellow: Color::Yellow,
            gold: Color::DarkYellow,
            orange: Color::DarkYellow,
            red: Color::Red,
            dark_red: Color::DarkRed,
            magenta: Color::Magenta,
            purple: Color::DarkMagenta,
            cyan: Color::Cyan,
            blue: Color::Blue,
            green: Color::Green,
            gray: Color::Grey,
            dark_gray: Color::DarkGrey,
        },
    ]
});

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    ch: char,
    fg: Color,
}

#[derive(Clone, Copy, Debug)]
struct Position {
    x: f32,
    y: f32,
}

struct Player {
    pos: Position,
    prev_pos: Position,
    vx: f32,
    lives: i32,
    score: u32,
    level: u32,
    shoot_timer: f32,
    invulnerable_timer: f32,
    respawn_timer: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum EnemyKind {
    Drone,
    Hornet,
    Emissary,
    Boss,
}

struct Enemy {
    pos: Position,
    prev_pos: Position,
    kind: EnemyKind,
    diving: bool,
    dive_phase: f32,
    start_x: f32,
    start_y: f32,
}
struct Bullet {
    pos: Position,
    prev_pos: Position,
    is_player: bool,
}
struct Particle {
    pos: Position,
    prev_pos: Position,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    ch: char,
    color: Color,
}
struct Star {
    pos: Position,
    prev_pos: Position,
    phase: f32,
    speed: f32,
}

struct Game {
    term_width: u16,
    term_height: u16,
    logic_w: i32,
    logic_h: i32,
    offset_x: u16,
    offset_y: u16,

    current_palette_idx: usize,
    theme_popup_timer: f32,

    player: Player,
    enemies: Vec<Enemy>,
    bullets: Vec<Bullet>,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    fleet_dir: f32,
    anim_timer: f32,
    game_over: bool,
    paused: bool,
    in_menu: bool,
    game_over_timer: f32,
    level_banner_timer: f32,
    pending_spawn: bool,
    extra_life_popup_timer: f32,
    next_extra_life: u32,
    high_score: u32,
    new_high: bool,

    rng: rand::rngs::ThreadRng,
    curr: Vec<Cell>,
    prev: Vec<Cell>,
    force_redraw: bool,

    // UI change detection (avoids reprinting the status line every frame)
    last_score: u32,
    last_lives: i32,
    last_level: u32,
    last_high: u32,
    last_paused: bool,

    beep: bool,
}

// --- small rendering / math helpers ---
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn put(curr: &mut [Cell], w: usize, h: usize, x: i32, y: i32, ch: char, fg: Color) {
    if x >= 0 && (x as usize) < w && y >= 0 && (y as usize) < h {
        curr[y as usize * w + x as usize] = Cell { ch, fg };
    }
}

fn draw_centered(curr: &mut [Cell], w: usize, h: usize, y: i32, text: &str, fg: Color) {
    let len = text.chars().count() as i32;
    let x0 = w as i32 / 2 - len / 2;
    for (i, ch) in text.chars().enumerate() {
        put(curr, w, h, x0 + i as i32, y, ch, fg);
    }
}

fn draw_right(curr: &mut [Cell], w: usize, h: usize, y: i32, text: &str, fg: Color) {
    let len = text.chars().count() as i32;
    let x0 = w as i32 - 1 - len;
    for (i, ch) in text.chars().enumerate() {
        put(curr, w, h, x0 + i as i32, y, ch, fg);
    }
}

// --- terminal restore guard (also used by the panic hook) ---
fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(stdout(), Show, LeaveAlternateScreen);
}

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

// --- persistence ---
fn settings_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn load_high_score() -> u32 {
    settings_home()
        .and_then(|h| std::fs::read_to_string(h.join(".fakelaxian_highscore")).ok())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn save_high_score(v: u32) {
    if let Some(h) = settings_home() {
        let _ = std::fs::write(h.join(".fakelaxian_highscore"), v.to_string());
    }
}

fn load_palette() -> usize {
    settings_home()
        .and_then(|h| std::fs::read_to_string(h.join(".fakelaxian_palette")).ok())
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

fn save_palette(idx: usize) {
    if let Some(h) = settings_home() {
        let _ = std::fs::write(h.join(".fakelaxian_palette"), idx.to_string());
    }
}

impl Game {
    fn calc_logical_size(tw: u16, th: u16) -> (i32, i32) {
        let max_rows = th.saturating_sub(3).max(15) as i32;
        let max_cols = tw.max(30) as i32;

        let mut h = max_rows;
        let mut w = (h as f32 * 1.5).round() as i32;

        if w > max_cols {
            w = max_cols;
            h = (w as f32 / 1.5).round() as i32;
        }
        (w, h)
    }

    fn new(term_w: u16, term_h: u16, high_score: u32, palette_idx: usize, in_menu: bool) -> Self {
        let current_palette_idx = palette_idx % PALETTES.len();
        let p = PALETTES[current_palette_idx];

        let (logic_w, logic_h) = Self::calc_logical_size(term_w, term_h);

        let mut rng = rand::thread_rng();
        let stars = (0..50)
            .map(|_| {
                let pos = Position {
                    x: rng.gen_range(1..logic_w - 1) as f32,
                    y: rng.gen_range(1..logic_h - 1) as f32,
                };
                Star {
                    pos,
                    prev_pos: pos,
                    phase: rng.gen_range(0.0..PI * 2.0),
                    speed: rng.gen_range(0.5..2.0),
                }
            })
            .collect();

        let w = logic_w as usize;
        let h = logic_h as usize;
        let empty = vec![Cell { ch: ' ', fg: p.fg }; w * h];

        let player_start = Position {
            x: (logic_w / 2) as f32,
            y: (logic_h - 3) as f32,
        };

        let mut game = Game {
            term_width: term_w,
            term_height: term_h,
            logic_w,
            logic_h,
            offset_x: 0,
            offset_y: 0,
            current_palette_idx,
            theme_popup_timer: 0.0,
            player: Player {
                pos: player_start,
                prev_pos: player_start,
                vx: 0.0,
                lives: 3,
                score: 0,
                level: 1,
                shoot_timer: 0.0,
                invulnerable_timer: 0.0,
                respawn_timer: 0.0,
            },
            enemies: Vec::new(),
            bullets: Vec::new(),
            particles: Vec::new(),
            stars,
            fleet_dir: 1.0,
            anim_timer: 0.0,
            game_over: false,
            paused: false,
            in_menu,
            game_over_timer: 0.0,
            level_banner_timer: 0.0,
            pending_spawn: false,
            extra_life_popup_timer: 0.0,
            next_extra_life: EXTRA_LIFE_INTERVAL,
            high_score,
            new_high: false,
            rng,
            curr: empty.clone(),
            prev: empty,
            force_redraw: true,
            last_score: 0,
            last_lives: 3,
            last_level: 1,
            last_high: high_score,
            last_paused: false,
            beep: false,
        };

        game.recalculate_offsets();
        game.spawn_enemies();
        game
    }

    fn palette(&self) -> Palette {
        PALETTES[self.current_palette_idx]
    }

    fn next_palette(&mut self) {
        self.current_palette_idx = (self.current_palette_idx + 1) % PALETTES.len();
        self.theme_popup_timer = 3.0;
        self.force_redraw = true;
        save_palette(self.current_palette_idx);
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.force_redraw = true;
    }

    fn start_game(&mut self) {
        self.in_menu = false;
        self.force_redraw = true;
    }

    fn persist_high_score(&self) {
        save_high_score(self.high_score);
    }

    fn too_small(&self) -> bool {
        self.term_width < MIN_COLS || self.term_height < MIN_ROWS
    }

    fn recalculate_offsets(&mut self) {
        self.offset_x = self.term_width.saturating_sub(self.logic_w as u16) / 2;
        self.offset_y = self.term_height.saturating_sub(self.logic_h as u16 + 3) / 2;
        self.force_redraw = true;
    }

    fn handle_resize(&mut self, w: u16, h: u16) {
        self.term_width = w;
        self.term_height = h;
        let (new_w, new_h) = Self::calc_logical_size(w, h);

        let old_w = self.logic_w as f32;
        let old_h = self.logic_h as f32;

        self.logic_w = new_w;
        self.logic_h = new_h;

        self.player.pos.x = (self.player.pos.x / old_w) * new_w as f32;
        self.player.pos.y = new_h as f32 - 3.0;
        self.player.prev_pos = self.player.pos;

        for e in &mut self.enemies {
            e.pos.x = (e.pos.x / old_w) * new_w as f32;
            e.start_x = (e.start_x / old_w) * new_w as f32;
            if !e.diving {
                e.pos.y = (e.pos.y / old_h) * new_h as f32;
                e.start_y = e.pos.y;
            }
            e.prev_pos = e.pos;
        }

        for b in &mut self.bullets {
            b.pos.x = (b.pos.x / old_w) * new_w as f32;
            b.pos.y = (b.pos.y / old_h) * new_h as f32;
            b.prev_pos = b.pos;
        }

        for p in &mut self.particles {
            p.pos.x = (p.pos.x / old_w) * new_w as f32;
            p.pos.y = (p.pos.y / old_h) * new_h as f32;
            p.prev_pos = p.pos;
        }

        for s in &mut self.stars {
            s.pos.x = (s.pos.x / old_w) * new_w as f32;
            s.pos.y = (s.pos.y / old_h) * new_h as f32;
            s.prev_pos = s.pos;
        }

        let pal = self.palette();
        let w = new_w as usize;
        let h = new_h as usize;
        self.curr = vec![
            Cell {
                ch: ' ',
                fg: pal.fg
            };
            w * h
        ];
        self.prev = self.curr.clone();

        self.recalculate_offsets();
    }

    fn spawn_enemies(&mut self) {
        self.enemies.clear();
        let cols = if self.logic_w < 40 { 9 } else { 11 };
        let rows = 5;
        let spacing_x = (self.logic_w as f32 / (cols as f32 + 2.0)).min(4.0);
        let spacing_y = (self.logic_h as f32 / 12.0).clamp(2.0, 3.0);

        let start_x = (self.logic_w as f32 - ((cols - 1) as f32 * spacing_x)) / 2.0;
        let start_y = 3.0;

        for r in 0..rows {
            for c in 0..cols {
                let kind = match r {
                    0 => EnemyKind::Boss,
                    1 => EnemyKind::Emissary,
                    2 => EnemyKind::Hornet,
                    _ => EnemyKind::Drone,
                };
                if r == 0 {
                    let mid = cols / 2;
                    if c < mid - 1 || c > mid + 1 {
                        continue;
                    }
                }

                let px = start_x + (c as f32 * spacing_x);
                let py = start_y + (r as f32 * spacing_y);
                let pos = Position { x: px, y: py };
                self.enemies.push(Enemy {
                    pos,
                    prev_pos: pos,
                    start_x: px,
                    start_y: py,
                    kind,
                    diving: false,
                    dive_phase: 0.0,
                });
            }
        }
    }

    fn update(&mut self, move_dir: f32, shoot: bool, brake: bool) {
        if self.paused {
            return;
        }
        let dt = FIXED_TIMESTEP.as_secs_f32();
        self.anim_timer += dt;
        if self.theme_popup_timer > 0.0 {
            self.theme_popup_timer -= dt;
        }
        if self.extra_life_popup_timer > 0.0 {
            self.extra_life_popup_timer -= dt;
        }
        if self.level_banner_timer > 0.0 {
            self.level_banner_timer -= dt;
        }
        if self.game_over_timer > 0.0 {
            self.game_over_timer -= dt;
        }

        // snapshot previous positions for interpolation
        self.player.prev_pos = self.player.pos;
        for e in &mut self.enemies {
            e.prev_pos = e.pos;
        }
        for b in &mut self.bullets {
            b.prev_pos = b.pos;
        }
        for p in &mut self.particles {
            p.prev_pos = p.pos;
        }
        for s in &mut self.stars {
            s.prev_pos = s.pos;
        }

        // particles + stars keep animating even on the game-over screen
        for p in &mut self.particles {
            p.pos.x += p.vx * dt;
            p.pos.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        for star in &mut self.stars {
            star.pos.y += star.speed * dt;
            if star.pos.y > self.logic_h as f32 {
                star.pos.y = 0.0;
                star.pos.x = self.rng.gen_range(1..self.logic_w - 1) as f32;
                star.prev_pos = star.pos; // avoid interpolating across the wrap
            }
        }

        if self.in_menu {
            return; // attract mode: only stars/particles animate on the title screen
        }

        if self.game_over {
            return;
        }

        // respawn handling
        if self.player.respawn_timer > 0.0 {
            self.player.respawn_timer -= dt;
            if self.player.respawn_timer <= 0.0 {
                let pos = Position {
                    x: (self.logic_w / 2) as f32,
                    y: (self.logic_h - 3) as f32,
                };
                self.player.pos = pos;
                self.player.prev_pos = pos;
                self.player.vx = 0.0;
                self.player.invulnerable_timer = 2.0;
            }
        }
        if self.player.invulnerable_timer > 0.0 {
            self.player.invulnerable_timer -= dt;
        }

        // player movement
        if self.player.respawn_timer <= 0.0 {
            if brake {
                self.player.vx = 0.0;
            } else if move_dir != 0.0 {
                self.player.vx += move_dir * 450.0 * dt;
            } else {
                self.player.vx *= 0.15;
            }
            self.player.vx = self.player.vx.clamp(-55.0, 55.0);
            self.player.pos.x =
                (self.player.pos.x + self.player.vx * dt).clamp(3.0, self.logic_w as f32 - 4.0);
        }

        // shooting (bullet cap + fire-rate limit)
        self.player.shoot_timer = (self.player.shoot_timer - dt).max(0.0);
        if shoot
            && self.player.shoot_timer <= 0.0
            && self.player.respawn_timer <= 0.0
            && self.bullets.iter().filter(|b| b.is_player).count() < 2
        {
            let pos = Position {
                x: self.player.pos.x,
                y: self.player.pos.y - 1.5,
            };
            self.bullets.push(Bullet {
                pos,
                prev_pos: pos,
                is_player: true,
            });
            self.player.shoot_timer = 0.12;
        }

        // fleet movement
        let speed = (10.0 + (self.player.level as f32 * 2.0)).min(30.0);
        let mut hit_edge = false;
        for enemy in &mut self.enemies {
            if !enemy.diving {
                enemy.pos.x += self.fleet_dir * speed * dt;
                enemy.start_x = enemy.pos.x;
                if enemy.pos.x <= 2.0 || enemy.pos.x >= self.logic_w as f32 - 3.0 {
                    hit_edge = true;
                }
            }
        }

        if hit_edge {
            self.fleet_dir *= -1.0;
            for enemy in &mut self.enemies {
                if !enemy.diving {
                    enemy.pos.y += 1.0;
                    enemy.start_y = enemy.pos.y;
                }
            }
        }

        // enemy dives + shooting
        let dive_chance = 0.001 * self.player.level as f64;
        for enemy in &mut self.enemies {
            if !enemy.diving && self.rng.gen_bool(dive_chance) {
                enemy.diving = true;
                enemy.dive_phase = 0.0;
            }
            if enemy.diving {
                enemy.dive_phase += dt * 2.0;
                enemy.pos.y += 15.0 * dt;
                enemy.pos.x += (enemy.dive_phase * PI).sin()
                    * 5.0
                    * dt
                    * if enemy.start_x < self.logic_w as f32 / 2.0 {
                        1.0
                    } else {
                        -1.0
                    };
                if self.rng.gen_bool(0.02) {
                    let pos = enemy.pos;
                    self.bullets.push(Bullet {
                        pos,
                        prev_pos: pos,
                        is_player: false,
                    });
                }
                if enemy.pos.y > self.logic_h as f32 {
                    enemy.diving = false;
                    enemy.pos.y = 1.0;
                    enemy.pos.x = enemy.start_x;
                    enemy.prev_pos = enemy.pos; // avoid interpolating across the wrap
                }
            }
        }

        // bullets
        for bullet in &mut self.bullets {
            if bullet.is_player {
                bullet.pos.y -= 50.0 * dt;
            } else {
                bullet.pos.y += 20.0 * dt;
            }
        }
        self.bullets
            .retain(|b| b.pos.y > 0.0 && b.pos.y < self.logic_h as f32);

        self.handle_collisions();

        // level progression (with a short banner before the next wave spawns)
        if self.enemies.is_empty() && !self.game_over && !self.pending_spawn {
            self.pending_spawn = true;
            self.player.level += 1;
            self.level_banner_timer = 2.0;
            self.beep = true;
        }
        if self.pending_spawn && self.level_banner_timer <= 0.0 {
            self.pending_spawn = false;
            self.spawn_enemies();
        }
    }

    fn spawn_explosion(&mut self, x: f32, y: f32, color: Color) {
        let chars = ['*', '+', '·', 'x'];
        for _ in 0..15 {
            let angle = self.rng.gen_range(0.0..PI * 2.0);
            let speed = self.rng.gen_range(5.0..18.0);
            let life = self.rng.gen_range(0.2..0.7);
            let ci = self.rng.gen_range(0..chars.len());
            self.particles.push(Particle {
                pos: Position { x, y },
                prev_pos: Position { x, y },
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life,
                max_life: 0.7,
                ch: chars[ci],
                color,
            });
        }
    }

    fn handle_collisions(&mut self) {
        let enemy_count = self.enemies.len();
        let bullet_count = self.bullets.len();
        let mut enemies_to_remove = vec![false; enemy_count];
        let mut bullets_to_remove = vec![false; bullet_count];
        let mut explosions: Vec<(f32, f32, Color)> = Vec::new();
        let mut score_to_add = 0u32;
        let mut player_hit = false;
        let is_vulnerable =
            self.player.invulnerable_timer <= 0.0 && self.player.respawn_timer <= 0.0;
        let pal = self.palette();

        for (b_idx, bullet) in self.bullets.iter().enumerate() {
            if bullet.is_player {
                for (e_idx, enemy) in self.enemies.iter().enumerate() {
                    if enemies_to_remove[e_idx] {
                        continue;
                    }
                    let dx = (bullet.pos.x - enemy.pos.x).abs();
                    let dy = (bullet.pos.y - enemy.pos.y).abs();
                    let hitbox_w = if enemy.kind == EnemyKind::Boss {
                        2.5
                    } else {
                        1.5
                    };
                    if dx < hitbox_w && dy < 1.0 {
                        enemies_to_remove[e_idx] = true;
                        bullets_to_remove[b_idx] = true;
                        score_to_add += match enemy.kind {
                            EnemyKind::Drone => 50,
                            EnemyKind::Hornet => 100,
                            EnemyKind::Emissary => 150,
                            EnemyKind::Boss => 300,
                        };
                        let color = match enemy.kind {
                            EnemyKind::Drone => pal.cyan,
                            EnemyKind::Hornet => pal.red,
                            _ => pal.purple,
                        };
                        explosions.push((enemy.pos.x, enemy.pos.y, color));
                        self.beep = true;
                        break;
                    }
                }
            } else if is_vulnerable {
                let dx = (bullet.pos.x - self.player.pos.x).abs();
                let dy = (bullet.pos.y - self.player.pos.y).abs();
                if dx < 1.5 && dy < 1.5 {
                    bullets_to_remove[b_idx] = true;
                    player_hit = true;
                }
            }
        }

        if is_vulnerable {
            for (e_idx, enemy) in self.enemies.iter().enumerate() {
                if enemies_to_remove[e_idx] {
                    continue;
                }
                let dx = (enemy.pos.x - self.player.pos.x).abs();
                let dy = (enemy.pos.y - self.player.pos.y).abs();
                if dx < 2.0 && dy < 2.0 {
                    player_hit = true;
                    enemies_to_remove[e_idx] = true;
                    explosions.push((enemy.pos.x, enemy.pos.y, pal.red));
                    break;
                }
            }
        }

        self.player.score += score_to_add;

        if self.player.score > self.high_score {
            self.high_score = self.player.score;
            self.new_high = true;
        }

        // extra life at fixed score thresholds
        if self.player.score >= self.next_extra_life {
            self.player.lives += 1;
            self.next_extra_life += EXTRA_LIFE_INTERVAL;
            self.extra_life_popup_timer = 2.0;
            self.beep = true;
        }

        if player_hit {
            self.player.lives -= 1;
            for _ in 0..3 {
                explosions.push((self.player.pos.x, self.player.pos.y, pal.orange));
            }
            self.beep = true;
            if self.player.lives <= 0 {
                self.game_over = true;
                self.game_over_timer = 1.2;
                self.player.vx = 0.0;
                self.persist_high_score();
            } else {
                self.player.invulnerable_timer = 0.0;
                self.player.respawn_timer = 1.5;
                self.player.vx = 0.0;
            }
        }

        for (x, y, color) in explosions {
            self.spawn_explosion(x, y, color);
        }

        let mut i = self.bullets.len();
        while i > 0 {
            i -= 1;
            if bullets_to_remove[i] {
                self.bullets.remove(i);
            }
        }

        let mut i = self.enemies.len();
        while i > 0 {
            i -= 1;
            if enemies_to_remove[i] {
                self.enemies.remove(i);
            }
        }
    }

    fn render_frame(&mut self, alpha: f32) {
        let pal = self.palette();
        let w = self.logic_w as usize;
        let h = self.logic_h as usize;
        let anim = self.anim_timer;
        let game_over = self.game_over;
        let in_menu = self.in_menu;

        let curr = &mut self.curr;
        curr.fill(Cell {
            ch: ' ',
            fg: pal.fg,
        });

        // border
        for x in 0..w as i32 {
            put(curr, w, h, x, 0, '─', pal.blue);
            put(curr, w, h, x, h as i32 - 1, '─', pal.blue);
        }
        for y in 0..h as i32 {
            put(curr, w, h, 0, y, '│', pal.blue);
            put(curr, w, h, w as i32 - 1, y, '│', pal.blue);
        }
        put(curr, w, h, 0, 0, '┌', pal.blue);
        put(curr, w, h, w as i32 - 1, 0, '┐', pal.blue);
        put(curr, w, h, 0, h as i32 - 1, '└', pal.blue);
        put(curr, w, h, w as i32 - 1, h as i32 - 1, '┘', pal.blue);

        // stars
        let star_colors = [pal.cyan, pal.yellow, pal.magenta, pal.green, pal.red];
        for star in &self.stars {
            let flicker = (star.phase + anim * 5.0).sin();
            if flicker > 0.0 {
                let color = if flicker > 0.8 {
                    star_colors[(star.pos.x as usize) % 5]
                } else {
                    pal.dark_gray
                };
                let ch = if flicker > 0.9 { '+' } else { '.' };
                let ix = lerp(star.prev_pos.x, star.pos.x, alpha) as i32;
                let iy = lerp(star.prev_pos.y, star.pos.y, alpha) as i32;
                put(curr, w, h, ix, iy, ch, color);
            }
        }

        // particles
        for p in &self.particles {
            let fade = p.life / p.max_life;
            let color = if fade > 0.6 {
                pal.fg
            } else if fade > 0.3 {
                p.color
            } else {
                pal.dark_red
            };
            let ix = lerp(p.prev_pos.x, p.pos.x, alpha) as i32;
            let iy = lerp(p.prev_pos.y, p.pos.y, alpha) as i32;
            put(curr, w, h, ix, iy, p.ch, color);
        }

        let frame_is_even = ((anim * 4.0) as usize).is_multiple_of(2);

        if !game_over {
            for enemy in &self.enemies {
                let px = lerp(enemy.prev_pos.x, enemy.pos.x, alpha) as i32;
                let py = lerp(enemy.prev_pos.y, enemy.pos.y, alpha) as i32;
                match enemy.kind {
                    EnemyKind::Drone => {
                        let (w_l, w_r) = if frame_is_even {
                            ('<', '>')
                        } else {
                            ('/', '\\')
                        };
                        put(curr, w, h, px - 1, py, w_l, pal.blue);
                        put(curr, w, h, px, py, '█', pal.cyan);
                        put(curr, w, h, px + 1, py, w_r, pal.blue);
                    }
                    EnemyKind::Hornet => {
                        let (w_l, w_r) = if frame_is_even {
                            ('>', '<')
                        } else {
                            ('\\', '/')
                        };
                        let c_col = if enemy.diving { pal.magenta } else { pal.red };
                        let w_col = if enemy.diving { pal.cyan } else { pal.orange };
                        put(curr, w, h, px - 1, py, w_l, w_col);
                        put(curr, w, h, px, py, '█', c_col);
                        put(curr, w, h, px + 1, py, w_r, w_col);
                    }
                    EnemyKind::Emissary => {
                        let (w_l, w_r) = if frame_is_even {
                            ('[', ']')
                        } else {
                            ('{', '}')
                        };
                        put(curr, w, h, px - 1, py, w_l, pal.purple);
                        put(curr, w, h, px, py, '█', pal.magenta);
                        put(curr, w, h, px + 1, py, w_r, pal.purple);
                    }
                    EnemyKind::Boss => {
                        let is_dive = enemy.diving;
                        let main_c = if is_dive { pal.cyan } else { pal.blue };
                        let accent = if is_dive { pal.fg } else { pal.gold };
                        let core_c = if is_dive { pal.magenta } else { pal.red };

                        put(curr, w, h, px - 1, py - 1, '_', accent);
                        put(curr, w, h, px, py - 1, '^', accent);
                        put(curr, w, h, px + 1, py - 1, '_', accent);

                        let (edge_l, edge_r) = if frame_is_even {
                            ('<', '>')
                        } else {
                            ('[', ']')
                        };
                        put(curr, w, h, px - 2, py, edge_l, main_c);
                        put(curr, w, h, px - 1, py, '█', main_c);
                        put(curr, w, h, px, py, '█', core_c);
                        put(curr, w, h, px + 1, py, '█', main_c);
                        put(curr, w, h, px + 2, py, edge_r, main_c);
                    }
                }
            }
        }

        // bullets
        for b in &self.bullets {
            let px = lerp(b.prev_pos.x, b.pos.x, alpha) as i32;
            let py = lerp(b.prev_pos.y, b.pos.y, alpha) as i32;
            if b.is_player {
                put(curr, w, h, px, py, '|', pal.yellow);
            } else {
                put(curr, w, h, px, py, '*', pal.magenta);
            }
        }

        // player ship
        if !game_over && !in_menu && self.player.respawn_timer <= 0.0 {
            let px = lerp(self.player.prev_pos.x, self.player.pos.x, alpha) as i32;
            let py = lerp(self.player.prev_pos.y, self.player.pos.y, alpha) as i32;
            if self.player.invulnerable_timer <= 0.0 || ((anim * 10.0) as usize).is_multiple_of(2) {
                let jet_tick = ((anim * 20.0) as usize).is_multiple_of(2);
                let jet = if jet_tick { '\'' } else { 'v' };
                let jet_color = if jet_tick { pal.red } else { pal.orange };

                put(curr, w, h, px, py - 1, '^', pal.yellow);
                put(curr, w, h, px - 1, py, '<', pal.blue);
                put(curr, w, h, px, py, '█', pal.red);
                put(curr, w, h, px + 1, py, '>', pal.blue);
                put(curr, w, h, px - 1, py + 1, '|', pal.fg);
                put(curr, w, h, px, py + 1, jet, jet_color);
                put(curr, w, h, px + 1, py + 1, '|', pal.fg);
            }
        }

        // overlays (drawn into the buffer so the diff system cleans them up)
        let cy = h as i32 / 2;
        if in_menu {
            draw_centered(curr, w, h, cy - 4, "FAKELAXIAN", pal.gold);
            draw_centered(
                curr,
                w,
                h,
                cy - 2,
                "a retro arcade terminal shooter",
                pal.gray,
            );
            draw_centered(
                curr,
                w,
                h,
                cy,
                &format!("HIGH SCORE: {:05}", self.high_score),
                pal.green,
            );
            if ((anim * 2.0) as usize).is_multiple_of(2) {
                draw_centered(curr, w, h, cy + 2, "PRESS SPACE TO START", pal.cyan);
            }
            draw_centered(curr, w, h, cy + 4, "C: theme   Q: quit", pal.dark_gray);
        }
        if game_over && self.game_over_timer <= 0.0 {
            draw_centered(curr, w, h, cy - 1, "GAME OVER", pal.red);
            if self.new_high {
                draw_centered(curr, w, h, cy + 1, "NEW HIGH SCORE!", pal.gold);
            }
            draw_centered(
                curr,
                w,
                h,
                cy + 2,
                "Press R to restart or Q to quit",
                pal.fg,
            );
        }
        if self.paused {
            draw_centered(curr, w, h, cy, "PAUSED", pal.yellow);
        }
        if self.level_banner_timer > 0.0 {
            draw_centered(
                curr,
                w,
                h,
                3,
                &format!("LEVEL {}", self.player.level),
                pal.cyan,
            );
        }
        if self.extra_life_popup_timer > 0.0 {
            draw_centered(curr, w, h, 2, "1UP!", pal.green);
        }
        if self.theme_popup_timer > 0.0 {
            draw_right(curr, w, h, 1, &format!(" {} ", pal.name), pal.fg);
        }
    }

    fn draw(&mut self, alpha: f32) -> io::Result<()> {
        let pal = self.palette();
        let mut stdout = stdout();

        if self.too_small() {
            execute!(stdout, Clear(ClearType::All))?;
            let msg1 = "Terminal too small";
            let msg2 = "Resize to at least 40x15";
            let cx = (self.term_width as usize) / 2;
            let cy = (self.term_height as usize) / 2;
            queue!(
                stdout,
                MoveTo(
                    cx.saturating_sub(msg1.len() / 2) as u16,
                    cy.saturating_sub(1) as u16
                ),
                SetForegroundColor(pal.red),
                Print(msg1)
            )?;
            queue!(
                stdout,
                MoveTo(cx.saturating_sub(msg2.len() / 2) as u16, cy as u16),
                SetForegroundColor(pal.fg),
                Print(msg2)
            )?;
            return stdout.flush();
        }

        self.render_frame(alpha);

        let force = self.force_redraw;
        let w = self.logic_w as usize;
        let h = self.logic_h as usize;
        let max_safe_y = self.term_height.saturating_sub(1);

        // field background, then diff + run-length encoded emission
        if force {
            // fill the whole screen (including margins) with the theme background
            queue!(stdout, SetBackgroundColor(pal.bg), Clear(ClearType::All))?;
        } else {
            queue!(stdout, SetBackgroundColor(pal.bg))?;
        }
        let mut run = String::new();
        let mut last_fg: Option<Color> = None;
        for y in 0..h {
            let mut x = 0;
            while x < w {
                let idx = y * w + x;
                let cur = self.curr[idx];
                let prev = self.prev[idx];
                if !force && cur == prev {
                    x += 1;
                    continue;
                }

                queue!(
                    stdout,
                    MoveTo(
                        (self.offset_x as usize + x) as u16,
                        (self.offset_y as usize + y) as u16
                    )
                )?;
                if last_fg != Some(cur.fg) {
                    queue!(stdout, SetForegroundColor(cur.fg))?;
                    last_fg = Some(cur.fg);
                }
                run.clear();
                while x < w {
                    let idx2 = y * w + x;
                    let c2 = self.curr[idx2];
                    let p2 = self.prev[idx2];
                    if c2.fg != cur.fg {
                        break;
                    }
                    if !force && c2 == p2 {
                        break;
                    }
                    run.push(c2.ch);
                    x += 1;
                }
                queue!(stdout, Print(run.as_str()))?;
            }
        }
        std::mem::swap(&mut self.curr, &mut self.prev);
        self.force_redraw = false;

        // status line (only redrawn when something changed)
        let ui_changed = force
            || self.last_score != self.player.score
            || self.last_lives != self.player.lives
            || self.last_level != self.player.level
            || self.last_high != self.high_score
            || self.last_paused != self.paused;

        let ui_y = self.offset_y + self.logic_h as u16;
        if ui_y < max_safe_y && ui_changed {
            queue!(
                stdout,
                MoveTo(0, ui_y),
                SetBackgroundColor(pal.bg),
                Clear(ClearType::CurrentLine)
            )?;
            queue!(
                stdout,
                MoveTo(self.offset_x, ui_y),
                SetForegroundColor(pal.green),
                Print(format!("SCORE {:05}", self.player.score))
            )?;
            queue!(
                stdout,
                MoveTo(self.offset_x + 12, ui_y),
                SetForegroundColor(pal.cyan),
                Print(format!("LVL {}", self.player.level))
            )?;
            queue!(
                stdout,
                MoveTo(self.offset_x + 19, ui_y),
                SetForegroundColor(pal.red),
                Print(format!(
                    "LIVES {}",
                    "^".repeat(self.player.lives.max(0) as usize)
                ))
            )?;
            queue!(
                stdout,
                MoveTo(self.offset_x + 31, ui_y),
                SetForegroundColor(pal.gold),
                Print(format!("HI {:05}", self.high_score))
            )?;

            self.last_score = self.player.score;
            self.last_lives = self.player.lives;
            self.last_level = self.player.level;
            self.last_high = self.high_score;
            self.last_paused = self.paused;
        }

        // static help line (drawn once per force)
        let ui_y2 = ui_y + 1;
        if ui_y2 < max_safe_y && force {
            queue!(
                stdout,
                MoveTo(0, ui_y2),
                SetBackgroundColor(pal.bg),
                Clear(ClearType::CurrentLine)
            )?;
            if self.term_width > 75 {
                queue!(stdout, MoveTo(self.offset_x, ui_y2), SetForegroundColor(pal.gray),
                       Print("[Arrows/A/D]: Move  [Space]: Shoot  [S]: Brake  [C]: Theme  [P]: Pause  [R]: Restart  [Q]: Quit"))?;
            } else {
                queue!(
                    stdout,
                    MoveTo(self.offset_x, ui_y2),
                    SetForegroundColor(pal.gray),
                    Print("Move:A/D Shoot:SPC Brake:S Theme:C Pause:P Quit:Q")
                )?;
            }
        }

        if self.beep {
            stdout.write_all(b"\x07")?;
            self.beep = false;
        }

        stdout.flush()
    }
}

fn main() -> io::Result<()> {
    // Restore the terminal even on panic (with panic=abort the hook still runs).
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        prev_hook(info);
    }));

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;
    terminal::enable_raw_mode()?;
    let _guard = TerminalGuard;

    let (mut term_w, mut term_h) = terminal::size().unwrap_or((80, 24));
    let high_score = load_high_score();
    let palette_idx = load_palette();
    let mut game = Game::new(term_w, term_h, high_score, palette_idx, true);

    let mut quit = false;
    let mut last_time = Instant::now();
    let mut accumulated = Duration::ZERO;
    let mut move_dir: f32 = 0.0;
    let mut brake = false;
    let mut shoot_held = false;
    let mut move_sticky: f32 = 0.0;
    let mut shoot_sticky: f32 = 0.0;

    while !quit {
        let now = Instant::now();
        let mut frame_time = now.duration_since(last_time);
        if frame_time > MAX_FRAME_TIME {
            frame_time = MAX_FRAME_TIME;
        }
        last_time = now;
        let dt_real = frame_time.as_secs_f32();
        accumulated += frame_time;

        // handle terminal resize
        let (cw, ch) = terminal::size().unwrap_or((term_w, term_h));
        if cw != term_w || ch != term_h {
            term_w = cw;
            term_h = ch;
            game.handle_resize(cw, ch);
        }

        // drain input
        let mut move_event = false;
        let mut shoot_event = false;
        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                // Ctrl+C quits (and should never cycle the theme)
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C')) {
                        quit = true;
                        break;
                    }
                    continue;
                }

                if key.kind == KeyEventKind::Release {
                    match key.code {
                        KeyCode::Left
                        | KeyCode::Char('a')
                        | KeyCode::Char('A')
                        | KeyCode::Right
                        | KeyCode::Char('d')
                        | KeyCode::Char('D') => move_dir = 0.0,
                        KeyCode::Char(' ') => shoot_held = false,
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => brake = false,
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                        move_dir = -1.0;
                        move_event = true;
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                        move_dir = 1.0;
                        move_event = true;
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => brake = true,
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if game.in_menu {
                            game.start_game();
                            move_dir = 0.0;
                            brake = false;
                            shoot_held = false;
                            move_sticky = 0.0;
                            shoot_sticky = 0.0;
                        } else {
                            shoot_held = true;
                            shoot_event = true;
                        }
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') if !game.in_menu => {
                        game.toggle_pause();
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => game.next_palette(),
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        let hs = game.high_score;
                        let pal_idx = game.current_palette_idx;
                        game.persist_high_score();
                        game = Game::new(term_w, term_h, hs, pal_idx, false);
                        move_dir = 0.0;
                        brake = false;
                        shoot_held = false;
                        move_sticky = 0.0;
                        shoot_sticky = 0.0;
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => quit = true,
                    _ => {}
                }
            }
        }
        if quit {
            break;
        }

        // sticky fallback: auto-release held keys when no event arrives
        if move_event {
            move_sticky = STICKY_INPUT;
        } else {
            move_sticky -= dt_real;
            if move_sticky <= 0.0 {
                move_dir = 0.0;
            }
        }
        if shoot_event {
            shoot_sticky = STICKY_INPUT;
        } else {
            shoot_sticky -= dt_real;
            if shoot_sticky <= 0.0 {
                shoot_held = false;
            }
        }

        // fixed-timestep simulation
        if !game.too_small() {
            while accumulated >= FIXED_TIMESTEP {
                accumulated -= FIXED_TIMESTEP;
                game.update(move_dir, shoot_held, brake);
            }
        } else {
            accumulated = Duration::ZERO;
        }
        brake = false;

        // render with interpolation between ticks
        let alpha = accumulated.as_secs_f32() / FIXED_TIMESTEP.as_secs_f32();
        game.draw(alpha)?;

        // pace to the next frame, sleeping on input in the meantime
        let next = last_time + FRAME_DURATION;
        let now2 = Instant::now();
        if now2 < next {
            event::poll(next - now2)?;
        }
    }

    game.persist_high_score();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_rgb_parses_components() {
        assert_eq!(hex_rgb(0xff0000), Color::Rgb { r: 255, g: 0, b: 0 });
        assert_eq!(hex_rgb(0x00ff00), Color::Rgb { r: 0, g: 255, b: 0 });
        assert_eq!(hex_rgb(0x0000ff), Color::Rgb { r: 0, g: 0, b: 255 });
        assert_eq!(
            hex_rgb(0x1e2e3e),
            Color::Rgb {
                r: 30,
                g: 46,
                b: 62
            }
        );
    }

    #[test]
    fn logical_size_respects_cell_ratio() {
        let (w, h) = Game::calc_logical_size(80, 24);
        assert!(w > 0 && h > 0);
        let ratio = w as f32 / h as f32;
        assert!((ratio - 1.5).abs() < 0.2);
    }

    #[test]
    fn logical_size_clamps_small_terminal() {
        let (w, h) = Game::calc_logical_size(10, 5);
        assert!(w >= 15);
        assert!(h >= 15);
    }

    #[test]
    fn palettes_are_available() {
        assert!(PALETTES.len() >= 6);
    }

    #[test]
    fn put_clips_out_of_bounds() {
        let mut buf = vec![
            Cell {
                ch: ' ',
                fg: Color::Reset
            };
            4
        ];
        put(&mut buf, 2, 2, -1, 0, 'X', Color::Red);
        put(&mut buf, 2, 2, 5, 0, 'X', Color::Red);
        put(&mut buf, 2, 2, 0, 0, 'X', Color::Red);
        assert_eq!(buf[0].ch, 'X');
        assert_eq!(buf[1].ch, ' ');
    }
}
