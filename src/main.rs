mod level;

use std::collections::HashMap;

use bevy::camera::ScalingMode;
use bevy::prelude::*;
use level::{LEVEL_COUNT, LevelDef, ROWS, TILE, Tile};

const VIEW_H: f32 = ROWS as f32 * TILE;
const GRAVITY: f32 = 1900.0;
const GRAVITY_HELD: f32 = 1250.0;
const GRAVITY_CUT: f32 = 3000.0;
const JUMP_V: f32 = 640.0;
const STOMP_V: f32 = 420.0;
const RUN_MAX: f32 = 250.0;
const ACCEL_GROUND: f32 = 1500.0;
const ACCEL_AIR: f32 = 1000.0;
const FRICTION: f32 = 1800.0;
const MAX_FALL: f32 = 900.0;
const COYOTE: f32 = 0.08;
const JUMP_BUFFER: f32 = 0.12;
const PLAYER_HALF: Vec2 = Vec2::new(11.0, 14.0);
const ENEMY_HALF: Vec2 = Vec2::new(13.0, 12.0);
const COIN_HALF: f32 = 9.0;
const ENEMY_SPEED: f32 = 55.0;
const LEVEL_TIME: f32 = 300.0;
const START_LIVES: u32 = 3;

// Touch button layout, in logical pixels from the bottom corners.
const BTN: f32 = 104.0;
const BTN_MARGIN: f32 = 18.0;
const BTN_GAP: f32 = 14.0;
const JUMP_BTN: f32 = 124.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rusty Jump".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Game::new())
        .init_resource::<Controls>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                read_controls,
                phase_input,
                reload_level,
                player_physics,
                enemy_physics,
                interactions,
                dying_update,
                clear_update,
                animate,
                camera_follow,
                update_ui,
            )
                .chain(),
        )
        .run();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Title,
    Playing,
    Dying,
    Clear,
    GameOver,
    Win,
}

#[derive(Resource)]
struct Game {
    phase: Phase,
    level: usize,
    lives: u32,
    coins: u32,
    score: u32,
    time: f32,
    timer: f32,
    reload: bool,
}

impl Game {
    fn new() -> Self {
        Game {
            phase: Phase::Title,
            level: option_env!("RJ_LEVEL").and_then(|v| v.parse().ok()).unwrap_or(0),
            lives: START_LIVES,
            coins: 0,
            score: 0,
            time: LEVEL_TIME,
            timer: 0.0,
            reload: true,
        }
    }
}

#[derive(Resource)]
struct Map {
    def: LevelDef,
    blocks: HashMap<(i32, i32), Entity>,
}

#[derive(Resource, Default)]
struct Controls {
    left: bool,
    right: bool,
    jump_held: bool,
    jump_pressed: bool,
    start_pressed: bool,
    touch_seen: bool,
}

#[derive(Resource)]
struct Palette {
    circle: Handle<Mesh>,
    triangle: Handle<Mesh>,
    coin: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    cloud: Handle<ColorMaterial>,
    hill: Handle<ColorMaterial>,
    bush: Handle<ColorMaterial>,
    flag: Handle<ColorMaterial>,
    pole_ball: Handle<ColorMaterial>,
}

#[derive(Component)]
struct LevelEntity;

#[derive(Component)]
struct Player {
    vel: Vec2,
    grounded: bool,
    coyote: f32,
    buffer: f32,
    facing: f32,
    anim: f32,
}

#[derive(Component)]
struct PlayerBody;

#[derive(Component)]
struct Foot(f32);

#[derive(Component)]
struct Enemy {
    vel: Vec2,
    active: bool,
    squashed: Option<f32>,
    anim: f32,
}

#[derive(Component)]
struct Coin;

#[derive(Component)]
struct Bump {
    t: f32,
    base_y: f32,
}

#[derive(Component)]
struct Popup {
    t: f32,
    life: f32,
    vel: Vec2,
    gravity: f32,
}

#[derive(Component)]
struct FlagCloth;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct Overlay;

#[derive(Component)]
struct OverlayText;

#[derive(Component)]
struct TouchUi;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEW_H,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(0.0, VIEW_H / 2.0, 0.0),
    ));

    let mut mat = |c: Color| materials.add(ColorMaterial::from(c));
    commands.insert_resource(Palette {
        circle: meshes.add(Circle::new(1.0)),
        triangle: meshes.add(Triangle2d::new(
            Vec2::new(0.0, 0.5),
            Vec2::new(0.0, -0.5),
            Vec2::new(-1.0, 0.0),
        )),
        coin: mat(Color::srgb(1.0, 0.82, 0.15)),
        enemy: mat(Color::srgb(0.58, 0.32, 0.16)),
        white: mat(Color::WHITE),
        cloud: mat(Color::srgba(1.0, 1.0, 1.0, 0.92)),
        hill: mat(Color::srgb(0.30, 0.68, 0.28)),
        bush: mat(Color::srgb(0.22, 0.75, 0.25)),
        flag: mat(Color::srgb(0.15, 0.80, 0.30)),
        pole_ball: mat(Color::srgb(0.20, 0.85, 0.35)),
    });

    let font = |size: f32| TextFont::from_font_size(size);

    commands.spawn((
        Text::new(""),
        font(20.0),
        TextColor(Color::WHITE),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(14.0),
            right: Val::Px(14.0),
            ..default()
        },
        HudText,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Overlay,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                font(30.0),
                TextColor(Color::WHITE),
                TextLayout::justify(Justify::Center),
                OverlayText,
            ));
        });

    let button = |label: &str, size: f32, pos: Node| {
        (
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::MAX,
                ..pos
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.22)),
            Visibility::Hidden,
            TouchUi,
            children![(
                Text::new(label),
                font(size * 0.4),
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.85)),
            )],
        )
    };
    commands.spawn(button(
        "<",
        BTN,
        Node {
            left: Val::Px(BTN_MARGIN),
            bottom: Val::Px(BTN_MARGIN),
            ..default()
        },
    ));
    commands.spawn(button(
        ">",
        BTN,
        Node {
            left: Val::Px(BTN_MARGIN + BTN + BTN_GAP),
            bottom: Val::Px(BTN_MARGIN),
            ..default()
        },
    ));
    commands.spawn(button(
        "A",
        JUMP_BTN,
        Node {
            right: Val::Px(BTN_MARGIN),
            bottom: Val::Px(BTN_MARGIN),
            ..default()
        },
    ));
}

fn read_controls(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window>,
    ui_scale: Res<UiScale>,
    mut c: ResMut<Controls>,
) {
    let any = |list: &[KeyCode]| keys.any_pressed(list.iter().copied());
    let jump_keys = [KeyCode::Space, KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::KeyZ, KeyCode::KeyK];
    c.left = any(&[KeyCode::ArrowLeft, KeyCode::KeyA]);
    c.right = any(&[KeyCode::ArrowRight, KeyCode::KeyD]);
    c.jump_held = any(&jump_keys);
    c.jump_pressed = keys.any_just_pressed(jump_keys);
    c.start_pressed = keys.any_just_pressed([KeyCode::Enter, KeyCode::Space])
        || mouse.just_pressed(MouseButton::Left)
        || touches.any_just_pressed();

    let (w, h) = (window.width(), window.height());
    let k = ui_scale.0;
    let zone = |p: Vec2| -> u8 {
        if p.y < h * 0.35 {
            return 0;
        }
        if p.x > w * 0.5 {
            3
        } else if p.x < (BTN_MARGIN + BTN + BTN_GAP / 2.0) * k {
            1
        } else {
            2
        }
    };
    for t in touches.iter() {
        c.touch_seen = true;
        match zone(t.position()) {
            1 => c.left = true,
            2 => c.right = true,
            3 => c.jump_held = true,
            _ => {}
        }
    }
    if touches.iter_just_pressed().any(|t| zone(t.position()) == 3) {
        c.jump_pressed = true;
    }
}

fn phase_input(mut game: ResMut<Game>, c: Res<Controls>) {
    if !c.start_pressed {
        return;
    }
    match game.phase {
        Phase::Title => {
            game.phase = Phase::Playing;
            game.time = LEVEL_TIME;
        }
        Phase::GameOver | Phase::Win => {
            *game = Game::new();
            game.phase = Phase::Playing;
        }
        _ => {}
    }
}

fn reload_level(
    mut commands: Commands,
    mut game: ResMut<Game>,
    palette: Res<Palette>,
    old: Query<Entity, With<LevelEntity>>,
    mut clear: ResMut<ClearColor>,
) {
    if !game.reload {
        return;
    }
    game.reload = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let def = level::build(game.level);
    clear.0 = Color::srgb(def.sky[0], def.sky[1], def.sky[2]);
    game.time = LEVEL_TIME;
    if game.phase != Phase::Title {
        game.phase = Phase::Playing;
    }

    let night = game.level == 2;
    spawn_scenery(&mut commands, &palette, &def, night);

    let mut blocks = HashMap::new();
    for y in 0..ROWS {
        for x in 0..def.width {
            let t = def.get(x, y);
            if t.solid() {
                let above = def.get(x, y + 1);
                let left = def.get(x - 1, y);
                let pipe_left = !matches!(left, Tile::Pipe | Tile::PipeTop);
                let e = spawn_tile(&mut commands, t, x, y, above != t, pipe_left);
                blocks.insert((x, y), e);
            }
        }
    }
    for &(x, y) in &def.coins {
        commands.spawn((
            Mesh2d(palette.circle.clone()),
            MeshMaterial2d(palette.coin.clone()),
            Transform::from_translation(tile_center(x, y).extend(2.0))
                .with_scale(Vec3::new(8.0, 11.0, 1.0)),
            Coin,
            LevelEntity,
        ));
    }
    for &(x, y) in &def.enemies {
        spawn_enemy(&mut commands, &palette, tile_center(x, y) - Vec2::Y * (16.0 - ENEMY_HALF.y));
    }
    spawn_flag(&mut commands, &palette, def.flag_x);
    let mut start_tile = def.start;
    if let Some(x) = option_env!("RJ_START_X").and_then(|v| v.parse().ok()) {
        start_tile.0 = x;
    }
    let start = tile_center(start_tile.0, start_tile.1) - Vec2::Y * (16.0 - PLAYER_HALF.y);
    spawn_player(&mut commands, start);

    commands.insert_resource(Map { def, blocks });
}

fn tile_center(x: i32, y: i32) -> Vec2 {
    Vec2::new((x as f32 + 0.5) * TILE, (y as f32 + 0.5) * TILE)
}

fn rect(color: Color, w: f32, h: f32, x: f32, y: f32, z: f32) -> (Sprite, Transform) {
    (
        Sprite::from_color(color, Vec2::new(w, h)),
        Transform::from_xyz(x, y, z),
    )
}

fn spawn_tile(commands: &mut Commands, t: Tile, x: i32, y: i32, top_exposed: bool, pipe_left: bool) -> Entity {
    let pos = tile_center(x, y).extend(1.0);
    let mut e = commands.spawn((Transform::from_translation(pos), Visibility::default(), LevelEntity));
    let s = TILE;
    match t {
        Tile::Ground => {
            e.with_children(|p| {
                p.spawn(rect(Color::srgb(0.45, 0.25, 0.12), s, s, 0.0, 0.0, 0.0));
                p.spawn(rect(Color::srgb(0.60, 0.36, 0.18), s - 4.0, s - 4.0, 0.0, 0.0, 0.01));
                p.spawn(rect(Color::srgb(0.50, 0.29, 0.14), 6.0, 4.0, -6.0, -5.0, 0.02));
                p.spawn(rect(Color::srgb(0.50, 0.29, 0.14), 4.0, 4.0, 7.0, 4.0, 0.02));
                if top_exposed {
                    p.spawn(rect(Color::srgb(0.25, 0.70, 0.20), s, 8.0, 0.0, 12.0, 0.03));
                    p.spawn(rect(Color::srgb(0.40, 0.85, 0.30), s, 3.0, 0.0, 14.5, 0.04));
                }
            });
        }
        Tile::Brick => {
            let mortar = Color::srgb(0.30, 0.12, 0.05);
            let brick = Color::srgb(0.75, 0.36, 0.16);
            e.with_children(|p| {
                p.spawn(rect(mortar, s, s, 0.0, 0.0, 0.0));
                for (bx, by, bw) in [
                    (-8.0, 8.0, 14.0),
                    (8.0, 8.0, 14.0),
                    (0.0, -8.0, 14.0),
                    (-12.5, -8.0, 5.0),
                    (12.5, -8.0, 5.0),
                ] {
                    p.spawn(rect(brick, bw, 13.0, bx, by, 0.01));
                }
            });
        }
        Tile::Question | Tile::Used => {
            let used = t == Tile::Used;
            let inner = if used {
                Color::srgb(0.55, 0.38, 0.22)
            } else {
                Color::srgb(0.98, 0.72, 0.15)
            };
            e.with_children(|p| {
                p.spawn(rect(Color::srgb(0.35, 0.20, 0.08), s, s, 0.0, 0.0, 0.0));
                p.spawn(rect(inner, s - 4.0, s - 4.0, 0.0, 0.0, 0.01));
                for (cx, cy) in [(-11.0, 11.0), (11.0, 11.0), (-11.0, -11.0), (11.0, -11.0)] {
                    p.spawn(rect(Color::srgb(0.35, 0.20, 0.08), 3.0, 3.0, cx, cy, 0.02));
                }
                if !used {
                    p.spawn((
                        Text2d::new("?"),
                        TextFont::from_font_size(24.0),
                        TextColor(Color::srgb(0.50, 0.25, 0.05)),
                        Transform::from_xyz(0.0, 0.0, 0.03),
                    ));
                }
            });
        }
        Tile::Block => {
            e.with_children(|p| {
                p.spawn(rect(Color::srgb(0.30, 0.18, 0.10), s, s, 0.0, 0.0, 0.0));
                p.spawn(rect(Color::srgb(0.72, 0.48, 0.30), s - 4.0, s - 4.0, 0.0, 0.0, 0.01));
                p.spawn(rect(Color::srgb(0.85, 0.62, 0.42), s - 4.0, 3.0, 0.0, 12.5, 0.02));
                p.spawn(rect(Color::srgb(0.85, 0.62, 0.42), 3.0, s - 4.0, -12.5, 0.0, 0.02));
            });
        }
        Tile::Pipe | Tile::PipeTop if pipe_left => {
            let top = t == Tile::PipeTop;
            let (w, h) = if top { (2.0 * s + 6.0, s) } else { (2.0 * s - 6.0, s + 0.5) };
            e.with_children(|p| {
                p.spawn(rect(Color::srgb(0.05, 0.35, 0.08), w, h, s / 2.0, 0.0, 0.0));
                let inner_h = if top { h - 6.0 } else { h };
                p.spawn(rect(Color::srgb(0.15, 0.68, 0.20), w - 6.0, inner_h, s / 2.0, 0.0, 0.01));
                p.spawn(rect(Color::srgb(0.55, 0.92, 0.45), 6.0, inner_h, s / 2.0 - w / 2.0 + 10.0, 0.0, 0.02));
                p.spawn(rect(Color::srgb(0.08, 0.50, 0.12), 6.0, inner_h, s / 2.0 + w / 2.0 - 12.0, 0.0, 0.02));
            });
        }
        Tile::Pipe | Tile::PipeTop => {}
        Tile::Empty => {}
    }
    e.id()
}

fn spawn_scenery(commands: &mut Commands, palette: &Palette, def: &LevelDef, night: bool) {
    let mut seed: u32 = 12345 + def.width as u32;
    let mut rand = move || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
        (seed >> 16) as f32 / 65536.0
    };
    let blob = |commands: &mut Commands, mat: &Handle<ColorMaterial>, x: f32, y: f32, r: f32, z: f32| {
        commands.spawn((
            Mesh2d(palette.circle.clone()),
            MeshMaterial2d(mat.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::new(r, r, 1.0)),
            LevelEntity,
        ));
    };
    let width = def.width as f32 * TILE;
    let mut x = 40.0;
    while x < width {
        let cy = VIEW_H - 70.0 - rand() * 120.0;
        let cloud = if night { &palette.white } else { &palette.cloud };
        if night {
            for _ in 0..6 {
                let sx = x + rand() * 300.0;
                let sy = VIEW_H - 20.0 - rand() * 220.0;
                blob(commands, &palette.white, sx, sy, 1.0 + rand() * 1.5, -9.0);
            }
        } else {
            blob(commands, cloud, x, cy, 22.0, -9.0);
            blob(commands, cloud, x + 24.0, cy + 8.0, 26.0, -9.0);
            blob(commands, cloud, x + 50.0, cy, 20.0, -9.0);
        }
        x += 220.0 + rand() * 200.0;
    }
    if night {
        blob(commands, &palette.white, 300.0, VIEW_H - 80.0, 30.0, -9.5);
    }
    let mut x = 0.0;
    while x < width {
        let r = 70.0 + rand() * 60.0;
        blob(commands, &palette.hill, x, 2.0 * TILE, r, -8.0);
        let bx = x + 150.0 + rand() * 120.0;
        for i in 0..3 {
            blob(commands, &palette.bush, bx + i as f32 * 20.0, 2.0 * TILE + 4.0, 16.0, -7.0);
        }
        x += 350.0 + rand() * 250.0;
    }
}

fn spawn_player(commands: &mut Commands, pos: Vec2) {
    let red = Color::srgb(0.88, 0.15, 0.12);
    let blue = Color::srgb(0.15, 0.30, 0.85);
    let skin = Color::srgb(1.0, 0.78, 0.60);
    let dark = Color::srgb(0.25, 0.12, 0.05);
    commands
        .spawn((
            Transform::from_translation(pos.extend(4.0)),
            Visibility::default(),
            Player {
                vel: Vec2::ZERO,
                grounded: false,
                coyote: 0.0,
                buffer: 0.0,
                facing: 1.0,
                anim: 0.0,
            },
            LevelEntity,
        ))
        .with_children(|p| {
            p.spawn((Transform::default(), Visibility::default(), PlayerBody))
                .with_children(|b| {
                    b.spawn((rect(dark, 8.0, 5.0, -5.0, -12.5, 0.0), Foot(-5.0)));
                    b.spawn((rect(dark, 8.0, 5.0, 5.0, -12.5, 0.0), Foot(5.0)));
                    b.spawn(rect(blue, 18.0, 11.0, 0.0, -5.5, 0.01));
                    b.spawn(rect(red, 22.0, 8.0, 0.0, 3.0, 0.02));
                    b.spawn(rect(blue, 3.0, 8.0, -5.0, 2.0, 0.03));
                    b.spawn(rect(blue, 3.0, 8.0, 5.0, 2.0, 0.03));
                    b.spawn(rect(skin, 16.0, 9.0, 1.0, 10.5, 0.02));
                    b.spawn(rect(red, 18.0, 5.0, 0.0, 16.5, 0.03));
                    b.spawn(rect(red, 8.0, 3.0, 9.0, 14.5, 0.03));
                    b.spawn(rect(Color::BLACK, 3.0, 4.0, 5.0, 11.5, 0.04));
                    b.spawn(rect(dark, 9.0, 2.5, 6.0, 7.5, 0.04));
                });
        });
}

fn spawn_enemy(commands: &mut Commands, palette: &Palette, pos: Vec2) {
    commands
        .spawn((
            Transform::from_translation(pos.extend(3.0)),
            Visibility::default(),
            Enemy {
                vel: Vec2::new(-ENEMY_SPEED, 0.0),
                active: false,
                squashed: None,
                anim: 0.0,
            },
            LevelEntity,
        ))
        .with_children(|p| {
            let dark = Color::srgb(0.20, 0.10, 0.05);
            p.spawn((rect(dark, 9.0, 5.0, -7.0, -9.5, 0.0), Foot(-7.0)));
            p.spawn((rect(dark, 9.0, 5.0, 7.0, -9.5, 0.0), Foot(7.0)));
            p.spawn((
                Mesh2d(palette.circle.clone()),
                MeshMaterial2d(palette.enemy.clone()),
                Transform::from_xyz(0.0, 1.0, 0.01).with_scale(Vec3::new(14.0, 11.0, 1.0)),
            ));
            p.spawn(rect(Color::srgb(0.95, 0.85, 0.70), 14.0, 5.0, 0.0, -7.0, 0.015));
            for sx in [-4.5, 4.5] {
                p.spawn(rect(Color::WHITE, 5.0, 7.0, sx, 3.0, 0.02));
                p.spawn(rect(Color::BLACK, 2.5, 4.0, sx, 2.0, 0.03));
                p.spawn(rect(Color::BLACK, 6.0, 2.0, sx, 7.5, 0.03));
            }
        });
}

fn spawn_flag(commands: &mut Commands, palette: &Palette, flag_x: i32) {
    let x = (flag_x as f32 + 0.5) * TILE;
    let base = 2.0 * TILE;
    let height = 9.5 * TILE;
    commands.spawn((
        rect(Color::srgb(0.85, 0.85, 0.85), 4.0, height, x, base + height / 2.0, 1.5),
        LevelEntity,
    ));
    commands.spawn((
        Mesh2d(palette.circle.clone()),
        MeshMaterial2d(palette.pole_ball.clone()),
        Transform::from_xyz(x, base + height + 4.0, 1.6).with_scale(Vec3::new(6.0, 6.0, 1.0)),
        LevelEntity,
    ));
    commands.spawn((
        Mesh2d(palette.triangle.clone()),
        MeshMaterial2d(palette.flag.clone()),
        Transform::from_xyz(x - 2.0, base + height - 14.0, 1.4).with_scale(Vec3::new(30.0, 24.0, 1.0)),
        FlagCloth,
        LevelEntity,
    ));
    let cx = x + 5.0 * TILE;
    let wall = Color::srgb(0.70, 0.35, 0.18);
    commands.spawn((rect(Color::srgb(0.30, 0.12, 0.05), 5.0 * TILE, 3.0 * TILE, cx, base + 1.5 * TILE, -1.0), LevelEntity));
    commands.spawn((rect(wall, 5.0 * TILE - 4.0, 3.0 * TILE - 4.0, cx, base + 1.5 * TILE, -0.99), LevelEntity));
    commands.spawn((rect(wall, 3.0 * TILE, 2.0 * TILE, cx, base + 4.0 * TILE, -0.99), LevelEntity));
    for i in 0..5 {
        commands.spawn((rect(wall, 18.0, 12.0, cx - 2.0 * TILE + i as f32 * TILE, base + 3.0 * TILE + 6.0, -0.99), LevelEntity));
    }
    commands.spawn((rect(Color::BLACK, 28.0, 44.0, cx, base + 22.0, -0.98), LevelEntity));
    commands.spawn((rect(Color::BLACK, 14.0, 20.0, cx, base + 4.0 * TILE, -0.98), LevelEntity));
}

fn spawn_popup(commands: &mut Commands, text: &str, pos: Vec2) {
    commands.spawn((
        Text2d::new(text),
        TextFont::from_font_size(16.0),
        TextColor(Color::WHITE),
        Transform::from_translation(pos.extend(6.0)),
        Popup {
            t: 0.0,
            life: 0.8,
            vel: Vec2::new(0.0, 60.0),
            gravity: 0.0,
        },
        LevelEntity,
    ));
}

fn spawn_coin_pop(commands: &mut Commands, palette: &Palette, pos: Vec2) {
    commands.spawn((
        Mesh2d(palette.circle.clone()),
        MeshMaterial2d(palette.coin.clone()),
        Transform::from_translation(pos.extend(0.9)).with_scale(Vec3::new(8.0, 11.0, 1.0)),
        Popup {
            t: 0.0,
            life: 0.55,
            vel: Vec2::new(0.0, 520.0),
            gravity: 1500.0,
        },
        LevelEntity,
    ));
}

struct Collision {
    wall: bool,
    ground: bool,
    ceiling: Option<(i32, i32)>,
}

fn to_tile(v: f32) -> i32 {
    (v / TILE).floor() as i32
}

fn move_body(def: &LevelDef, pos: &mut Vec2, vel: &mut Vec2, half: Vec2, dt: f32) -> Collision {
    let mut out = Collision {
        wall: false,
        ground: false,
        ceiling: None,
    };
    pos.x += vel.x * dt;
    let y0 = to_tile(pos.y - half.y + 0.5);
    let y1 = to_tile(pos.y + half.y - 0.5);
    if vel.x != 0.0 {
        let right = vel.x > 0.0;
        let tx = to_tile(if right { pos.x + half.x } else { pos.x - half.x });
        if (y0..=y1).any(|ty| def.get(tx, ty).solid()) {
            pos.x = if right {
                tx as f32 * TILE - half.x - 0.01
            } else {
                (tx + 1) as f32 * TILE + half.x + 0.01
            };
            vel.x = 0.0;
            out.wall = true;
        }
    }

    pos.y += vel.y * dt;
    let x0 = to_tile(pos.x - half.x + 0.5);
    let x1 = to_tile(pos.x + half.x - 0.5);
    if vel.y <= 0.0 {
        let ty = to_tile(pos.y - half.y);
        if (x0..=x1).any(|tx| def.get(tx, ty).solid()) {
            pos.y = (ty + 1) as f32 * TILE + half.y;
            vel.y = 0.0;
            out.ground = true;
        }
    } else {
        let ty = to_tile(pos.y + half.y);
        let hit = (x0..=x1)
            .filter(|&tx| def.get(tx, ty).solid())
            .min_by_key(|&tx| ((tx as f32 + 0.5) * TILE - pos.x).abs() as i32);
        if let Some(tx) = hit {
            pos.y = ty as f32 * TILE - half.y - 0.01;
            vel.y = 0.0;
            out.ceiling = Some((tx, ty));
        }
    }
    out
}

fn overlap(a: Vec2, ah: Vec2, b: Vec2, bh: Vec2) -> bool {
    (a.x - b.x).abs() < ah.x + bh.x && (a.y - b.y).abs() < ah.y + bh.y
}

fn substeps(dt: f32) -> (u32, f32) {
    let dt = dt.min(1.0 / 20.0);
    let n = (dt / (1.0 / 120.0)).ceil().max(1.0) as u32;
    (n, dt / n as f32)
}

#[allow(clippy::too_many_arguments)]
fn player_physics(
    mut commands: Commands,
    time: Res<Time>,
    c: Res<Controls>,
    mut game: ResMut<Game>,
    map: Option<ResMut<Map>>,
    palette: Res<Palette>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut enemies: Query<(&Transform, &mut Enemy), Without<Player>>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let Some(mut map) = map else { return };
    let Ok((mut tf, mut p)) = player.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let dir = c.right as i32 as f32 - c.left as i32 as f32;
    if dir != 0.0 {
        let accel = if p.grounded { ACCEL_GROUND } else { ACCEL_AIR };
        let turn = if p.vel.x * dir < 0.0 { 2.0 } else { 1.0 };
        p.vel.x = (p.vel.x + dir * accel * turn * dt).clamp(-RUN_MAX, RUN_MAX);
        p.facing = dir;
    } else {
        let f = if p.grounded { FRICTION } else { 300.0 } * dt;
        p.vel.x = if p.vel.x.abs() <= f { 0.0 } else { p.vel.x - f * p.vel.x.signum() };
    }

    if c.jump_pressed {
        p.buffer = JUMP_BUFFER;
    } else {
        p.buffer -= dt;
    }
    if p.grounded {
        p.coyote = COYOTE;
    } else {
        p.coyote -= dt;
    }
    if p.buffer > 0.0 && p.coyote > 0.0 {
        p.vel.y = JUMP_V + p.vel.x.abs() * 0.15;
        p.buffer = 0.0;
        p.coyote = 0.0;
        p.grounded = false;
    }
    let g = if p.vel.y > 0.0 {
        if c.jump_held { GRAVITY_HELD } else { GRAVITY_CUT }
    } else {
        GRAVITY
    };
    p.vel.y = (p.vel.y - g * dt).max(-MAX_FALL);

    let (n, step) = substeps(dt);
    let mut pos = tf.translation.truncate();
    let mut vel = p.vel;
    let mut grounded = false;
    let mut ceiling = None;
    for _ in 0..n {
        let col = move_body(&map.def, &mut pos, &mut vel, PLAYER_HALF, step);
        grounded |= col.ground;
        if col.ceiling.is_some() && ceiling.is_none() {
            ceiling = col.ceiling;
        }
    }
    p.vel = vel;
    p.grounded = grounded;
    p.anim += p.vel.x.abs() * dt;
    tf.translation.x = pos.x;
    tf.translation.y = pos.y;

    if let Some((tx, ty)) = ceiling {
        hit_block(&mut commands, &mut game, &mut map, &palette, tx, ty, &mut enemies);
    }
}

fn hit_block(
    commands: &mut Commands,
    game: &mut Game,
    map: &mut Map,
    palette: &Palette,
    tx: i32,
    ty: i32,
    enemies: &mut Query<(&Transform, &mut Enemy), Without<Player>>,
) {
    let t = map.def.get(tx, ty);
    let center = tile_center(tx, ty);
    match t {
        Tile::Question => {
            map.def.set(tx, ty, Tile::Used);
            if let Some(e) = map.blocks.remove(&(tx, ty)) {
                commands.entity(e).despawn();
            }
            let e = spawn_tile(commands, Tile::Used, tx, ty, false, false);
            commands.entity(e).insert(Bump {
                t: 0.0,
                base_y: center.y,
            });
            map.blocks.insert((tx, ty), e);
            spawn_coin_pop(commands, palette, center + Vec2::Y * TILE);
            game.coins += 1;
            game.score += 200;
            spawn_popup(commands, "200", center + Vec2::Y * TILE * 2.0);
        }
        Tile::Brick => {
            map.def.set(tx, ty, Tile::Empty);
            if let Some(e) = map.blocks.remove(&(tx, ty)) {
                commands.entity(e).despawn();
            }
            for (vx, vy) in [(-120.0, 420.0), (120.0, 420.0), (-90.0, 260.0), (90.0, 260.0)] {
                commands.spawn((
                    rect(Color::srgb(0.75, 0.36, 0.16), 10.0, 10.0, center.x, center.y, 5.0),
                    Popup {
                        t: 0.0,
                        life: 1.2,
                        vel: Vec2::new(vx, vy),
                        gravity: 1500.0,
                    },
                    LevelEntity,
                ));
            }
            game.score += 50;
        }
        _ => return,
    }
    // Knock out enemies standing on the block.
    for (etf, mut enemy) in enemies.iter_mut() {
        let ep = etf.translation.truncate();
        if enemy.squashed.is_none()
            && (ep.x - center.x).abs() < TILE * 0.75
            && (ep.y - ENEMY_HALF.y - (center.y + TILE / 2.0)).abs() < 4.0
        {
            enemy.squashed = Some(0.0);
            game.score += 100;
            spawn_popup(commands, "100", ep + Vec2::Y * 20.0);
        }
    }
}

fn enemy_physics(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    map: Option<Res<Map>>,
    camera: Single<&Transform, (With<Camera2d>, Without<Enemy>)>,
    window: Single<&Window>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy)>,
) {
    if !matches!(game.phase, Phase::Playing | Phase::Clear) {
        return;
    }
    let Some(map) = map else { return };
    let dt = time.delta_secs();
    let half_w = VIEW_H / 2.0 * window.width() / window.height().max(1.0);
    let wake_x = camera.translation.x + half_w + TILE * 2.0;
    let (n, step) = substeps(dt);

    let mut bodies: Vec<(Entity, Vec2, f32)> = vec![];
    for (e, mut tf, mut enemy) in &mut enemies {
        if let Some(t) = enemy.squashed.as_mut() {
            *t += dt;
            if *t > 0.5 {
                commands.entity(e).despawn();
            }
            continue;
        }
        if !enemy.active {
            if tf.translation.x < wake_x {
                enemy.active = true;
            } else {
                continue;
            }
        }
        enemy.vel.y = (enemy.vel.y - GRAVITY * dt).max(-MAX_FALL);
        let mut pos = tf.translation.truncate();
        let mut vel = enemy.vel;
        let dir = vel.x.signum();
        let mut wall = false;
        for _ in 0..n {
            wall |= move_body(&map.def, &mut pos, &mut vel, ENEMY_HALF, step).wall;
        }
        vel.x = if wall { -dir * ENEMY_SPEED } else { dir * ENEMY_SPEED };
        enemy.vel = vel;
        enemy.anim += dt;
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
        if pos.y < -TILE * 2.0 {
            commands.entity(e).despawn();
            continue;
        }
        bodies.push((e, pos, vel.x));
    }
    // Enemies bounce off each other.
    let mut flip = vec![];
    for i in 0..bodies.len() {
        for j in i + 1..bodies.len() {
            let (a, pa, va) = bodies[i];
            let (b, pb, vb) = bodies[j];
            if overlap(pa, ENEMY_HALF, pb, ENEMY_HALF) {
                let a_left = pa.x < pb.x;
                if (a_left && va > 0.0) || (!a_left && va < 0.0) {
                    flip.push(a);
                }
                if (a_left && vb < 0.0) || (!a_left && vb > 0.0) {
                    flip.push(b);
                }
            }
        }
    }
    for e in flip {
        if let Ok((_, _, mut enemy)) = enemies.get_mut(e) {
            enemy.vel.x = -enemy.vel.x;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn interactions(
    mut commands: Commands,
    mut game: ResMut<Game>,
    c: Res<Controls>,
    time: Res<Time>,
    map: Option<Res<Map>>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut enemies: Query<(&Transform, &mut Enemy), Without<Player>>,
    coins: Query<(Entity, &Transform), (With<Coin>, Without<Player>, Without<Enemy>)>,
) {
    if game.phase != Phase::Playing {
        return;
    }
    let Some(map) = map else { return };
    let Ok((mut tf, mut p)) = player.single_mut() else {
        return;
    };
    let pos = tf.translation.truncate();

    game.time -= time.delta_secs();
    if game.time <= 0.0 {
        game.time = 0.0;
        start_dying(&mut game, &mut p);
        return;
    }
    if pos.y < -TILE {
        start_dying(&mut game, &mut p);
        return;
    }

    for (e, ctf) in &coins {
        if overlap(pos, PLAYER_HALF, ctf.translation.truncate(), Vec2::splat(COIN_HALF)) {
            commands.entity(e).despawn();
            game.coins += 1;
            game.score += 100;
            if game.coins % 50 == 0 {
                game.lives += 1;
                spawn_popup(&mut commands, "1UP", pos + Vec2::Y * 24.0);
            }
        }
    }

    let mut stomped = false;
    for (etf, mut enemy) in &mut enemies {
        if enemy.squashed.is_some() {
            continue;
        }
        let ep = etf.translation.truncate();
        if !overlap(pos, PLAYER_HALF, ep, ENEMY_HALF) {
            continue;
        }
        if p.vel.y < 0.0 && pos.y - PLAYER_HALF.y > ep.y - 2.0 || stomped {
            enemy.squashed = Some(0.0);
            stomped = true;
            game.score += 100;
            spawn_popup(&mut commands, "100", ep + Vec2::Y * 20.0);
        } else {
            start_dying(&mut game, &mut p);
            return;
        }
    }
    if stomped {
        p.vel.y = if c.jump_held { JUMP_V * 0.9 } else { STOMP_V };
    }

    let pole_x = (map.def.flag_x as f32 + 0.5) * TILE;
    if pos.x + PLAYER_HALF.x >= pole_x - 2.0 {
        game.phase = Phase::Clear;
        game.timer = 0.0;
        let height = (pos.y - 2.0 * TILE).max(0.0);
        let bonus = ((height / (9.0 * TILE)).min(1.0) * 50.0) as u32 * 100 + 100;
        game.score += bonus;
        spawn_popup(&mut commands, &bonus.to_string(), pos + Vec2::new(30.0, 0.0));
        tf.translation.x = pole_x - PLAYER_HALF.x - 2.0;
        p.vel = Vec2::ZERO;
        p.facing = 1.0;
    }
}

fn start_dying(game: &mut Game, p: &mut Player) {
    game.phase = Phase::Dying;
    game.timer = 0.0;
    p.vel = Vec2::ZERO;
}

fn dying_update(time: Res<Time>, mut game: ResMut<Game>, mut player: Query<(&mut Transform, &mut Player)>) {
    if game.phase != Phase::Dying {
        return;
    }
    let dt = time.delta_secs();
    game.timer += dt;
    if let Ok((mut tf, mut p)) = player.single_mut() {
        if game.timer > 0.45 {
            if p.vel.y == 0.0 && tf.translation.y > -100.0 && game.timer < 0.5 {
                p.vel.y = 650.0;
            }
            p.vel.y -= GRAVITY * 0.8 * dt;
            tf.translation.y += p.vel.y * dt;
            tf.translation.z = 8.0;
        }
    }
    if game.timer > 2.6 {
        game.lives = game.lives.saturating_sub(1);
        if game.lives == 0 {
            game.phase = Phase::GameOver;
        } else {
            game.reload = true;
        }
    }
}

fn clear_update(
    time: Res<Time>,
    mut game: ResMut<Game>,
    map: Option<Res<Map>>,
    mut player: Query<(&mut Transform, &mut Player)>,
    mut flag: Query<&mut Transform, (With<FlagCloth>, Without<Player>)>,
) {
    if game.phase != Phase::Clear {
        return;
    }
    let Some(map) = map else { return };
    let dt = time.delta_secs();
    game.timer += dt;
    let ground_y = 2.0 * TILE;
    if let Ok(mut ftf) = flag.single_mut() {
        ftf.translation.y = (ftf.translation.y - 260.0 * dt).max(ground_y + 16.0);
    }
    let Ok((mut tf, mut p)) = player.single_mut() else {
        return;
    };
    let feet = tf.translation.y - PLAYER_HALF.y;
    if feet > ground_y + 0.5 && game.timer < 2.0 {
        tf.translation.y = (tf.translation.y - 260.0 * dt).max(ground_y + PLAYER_HALF.y);
    } else if game.timer > 1.0 {
        // Walk to the castle.
        p.vel.x = 140.0;
        p.vel.y = (p.vel.y - GRAVITY * dt).max(-MAX_FALL);
        let mut pos = tf.translation.truncate();
        let mut vel = p.vel;
        let col = move_body(&map.def, &mut pos, &mut vel, PLAYER_HALF, dt.min(1.0 / 30.0));
        p.grounded = col.ground;
        p.vel.y = vel.y;
        p.anim += 140.0 * dt;
        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
        let castle_door = (map.def.flag_x as f32 + 5.5) * TILE;
        if tf.translation.x >= castle_door {
            tf.translation.z = -2.0;
        }
    }
    if game.timer > 2.0 && game.time > 0.0 {
        let tick = (game.time.min(120.0 * dt * 3.0)).max(0.0);
        game.time -= tick;
        game.score += (tick * 50.0) as u32;
    }
    if game.timer > 4.5 && game.time <= 0.5 {
        game.level += 1;
        if game.level >= LEVEL_COUNT {
            game.phase = Phase::Win;
        } else {
            game.reload = true;
        }
    }
}

#[allow(clippy::type_complexity)]
fn animate(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Game>,
    player: Query<&Player>,
    mut bodies: Query<&mut Transform, With<PlayerBody>>,
    mut feet: Query<(&mut Transform, &Foot, &ChildOf), Without<PlayerBody>>,
    enemies: Query<(Entity, &Enemy)>,
    mut enemy_tf: Query<&mut Transform, (With<Enemy>, Without<PlayerBody>, Without<Foot>)>,
    mut coins: Query<&mut Transform, (With<Coin>, Without<PlayerBody>, Without<Foot>, Without<Enemy>)>,
    mut bumps: Query<
        (Entity, &mut Transform, &mut Bump),
        (Without<Coin>, Without<PlayerBody>, Without<Foot>, Without<Enemy>),
    >,
    mut popups: Query<
        (Entity, &mut Transform, &mut Popup),
        (Without<Bump>, Without<Coin>, Without<PlayerBody>, Without<Foot>, Without<Enemy>),
    >,
    body_ids: Query<Entity, With<PlayerBody>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    if let Ok(p) = player.single() {
        for mut btf in &mut bodies {
            btf.scale.x = p.facing;
            btf.rotation = if game.phase == Phase::Dying {
                Quat::from_rotation_z(std::f32::consts::PI)
            } else {
                Quat::IDENTITY
            };
        }
        let moving = p.grounded && p.vel.x.abs() > 10.0;
        for (mut ftf, foot, parent) in &mut feet {
            if !body_ids.contains(parent.parent()) {
                continue;
            }
            let swing = if moving { (p.anim * 0.08).sin() * 4.0 } else { 0.0 };
            let side = foot.0.signum();
            ftf.translation.x = foot.0 + swing * side;
            ftf.translation.y = if p.grounded { -12.5 } else { -10.0 };
        }
    }

    for (e, enemy) in &enemies {
        let Ok(mut tf) = enemy_tf.get_mut(e) else { continue };
        if enemy.squashed.is_some() {
            tf.scale = Vec3::new(1.2, 0.35, 1.0);
        } else {
            let wobble = if enemy.active { (enemy.anim * 10.0).sin() * 0.06 } else { 0.0 };
            tf.rotation = Quat::from_rotation_z(wobble);
        }
    }

    let spin = (t * 4.0).cos().abs().max(0.15);
    for mut ctf in &mut coins {
        ctf.scale.x = 8.0 * spin;
    }

    for (e, mut btf, mut bump) in &mut bumps {
        bump.t += dt;
        let k = (bump.t / 0.15).min(1.0);
        btf.translation.y = bump.base_y + (k * std::f32::consts::PI).sin() * 8.0;
        if k >= 1.0 {
            btf.translation.y = bump.base_y;
            commands.entity(e).remove::<Bump>();
        }
    }

    for (e, mut ptf, mut pop) in &mut popups {
        pop.t += dt;
        pop.vel.y -= pop.gravity * dt;
        ptf.translation += (pop.vel * dt).extend(0.0);
        if pop.t > pop.life {
            commands.entity(e).despawn();
        }
    }
}

fn camera_follow(
    game: Res<Game>,
    map: Option<Res<Map>>,
    window: Single<&Window>,
    player: Query<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Some(map) = map else { return };
    let half_w = VIEW_H / 2.0 * window.width() / window.height().max(1.0);
    let level_w = map.def.width as f32 * TILE;
    let target = match (game.phase, player.single()) {
        (Phase::Dying, _) => return,
        (_, Ok(tf)) => tf.translation.x + half_w * 0.15,
        _ => return,
    };
    let x = if level_w <= half_w * 2.0 {
        level_w / 2.0
    } else {
        target.clamp(half_w, level_w - half_w)
    };
    camera.translation.x = x;
    camera.translation.y = VIEW_H / 2.0;
}

#[allow(clippy::type_complexity)]
fn update_ui(
    game: Res<Game>,
    c: Res<Controls>,
    map: Option<Res<Map>>,
    window: Single<&Window>,
    mut ui_scale: ResMut<UiScale>,
    mut hud: Query<&mut Text, (With<HudText>, Without<OverlayText>)>,
    mut overlay_text: Query<&mut Text, (With<OverlayText>, Without<HudText>)>,
    mut overlay: Query<&mut Visibility, (With<Overlay>, Without<TouchUi>)>,
    mut touch_ui: Query<&mut Visibility, (With<TouchUi>, Without<Overlay>)>,
) {
    let scale = (window.width() / 1000.0).min(window.height() / 560.0).clamp(0.55, 1.0);
    if (ui_scale.0 - scale).abs() > 0.01 {
        ui_scale.0 = scale;
    }

    let name = map.as_ref().map(|m| m.def.name).unwrap_or("1-1");
    if let Ok(mut text) = hud.single_mut() {
        let s = format!(
            "SCORE {:06}    COINS x{:02}    WORLD {}    TIME {:03}    LIVES x{}",
            game.score,
            game.coins,
            name,
            game.time.ceil() as u32,
            game.lives
        );
        if text.0 != s {
            text.0 = s;
        }
    }

    let (show, msg) = match game.phase {
        Phase::Title => (
            true,
            "RUSTY JUMP\n\nArrows / A D : move    Space / W / Z : jump\nPhone: hold landscape, < > to move, A to jump\n\nPress SPACE or tap to start".to_string(),
        ),
        Phase::GameOver => (true, format!("GAME OVER\n\nSCORE {}\n\nPress SPACE or tap to retry", game.score)),
        Phase::Win => (
            true,
            format!("YOU WIN!\n\nFINAL SCORE {}\nCOINS {}\n\nPress SPACE or tap to play again", game.score, game.coins),
        ),
        _ => (false, String::new()),
    };
    if let Ok(mut v) = overlay.single_mut() {
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
    if let Ok(mut text) = overlay_text.single_mut()
        && text.0 != msg
    {
        text.0 = msg;
    }
    let want = if c.touch_seen && !show { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut touch_ui {
        if *v != want {
            *v = want;
        }
    }
}
