//! sim-view 最小 Demo（开源采纳 O2：bevy_ecs_tilemap + bevy_egui/egui_plot）。
//!
//! 对接口径（docs/architecture/open-source-adoptions.md）：
//! - 只读对接 sim-engine：每步 `run_tick()` 后取 `snapshot()` 渲染，视图层不写权威状态；
//! - 墙钟节拍仅展示节流（2.9 口径：权威时间是 u64 tick，本计时器不进模拟）；
//! - 网格=固定小场景（24×24），Agent 位置取 traffic.vehicles 的 (x,y)（1mm 网格单位）。
//!
//! 运行：cargo run -p sim-view --example demo
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_ecs_tilemap::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use sim_core::CommandPayload;
use sim_engine::{Engine, EngineConfig};

const W: u64 = 24;
const H: u64 = 24;
const TILE: f32 = 16.0;

#[derive(Resource)]
struct Sim(Engine);
#[derive(Resource)]
struct StepTimer(Timer);
#[derive(Resource)]
struct History {
    treasury: Vec<f64>,
    active: Vec<f64>,
}
/// tile 实体表（行优先）——渲染层颜色复位/着色用。
#[derive(Resource)]
struct TileMap {
    tiles: Vec<Entity>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CitySim-rs sim-view demo（非权威）".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TilemapPlugin)
        .add_plugins(EguiPlugin::default())
        .insert_resource(ClearColor(Color::srgb(0.12, 0.12, 0.15)))
        .add_systems(Startup, (setup_engine, setup_tilemap, setup_camera))
        .add_systems(Update, (step_sim, sync_tiles, ui_panel))
        .run();
}

fn build_engine() -> Engine {
    let mut coords = Vec::new();
    for gy in 0..H {
        for gx in 0..W {
            coords.push(((gx * 1000) as i64, (gy * 1000) as i64));
        }
    }
    let cfg = EngineConfig {
        seed: 7,
        coords,
        traffic_w: W as u32,
        traffic_h: H as u32,
        income_per_tick: 120,
        upkeep_per_tick: 50,
    };
    let mut e = Engine::new(cfg);
    for i in 0..12u64 {
        let id = e.spawn_agent();
        let goal = ((i.wrapping_mul(37)) + 5) % (W * H);
        let _ = e.submit_command(id, 1, CommandPayload::SetDestination { node: goal });
    }
    e
}

fn setup_engine(mut commands: Commands) {
    commands.insert_resource(Sim(build_engine()));
    commands.insert_resource(StepTimer(Timer::from_seconds(0.15, TimerMode::Repeating)));
    commands.insert_resource(History {
        treasury: Vec::new(),
        active: Vec::new(),
    });
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_tilemap(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // 单像素白纹理：tile 着色区分底图/Agent（tilemap 要求纹理资源）。
    let mut img = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = bevy::image::ImageSampler::nearest();
    let tex = images.add(img);

    let map_size = TilemapSize {
        x: W as u32,
        y: H as u32,
    };
    let grid_size = TilemapGridSize { x: TILE, y: TILE };
    let tile_size = TilemapTileSize { x: TILE, y: TILE };
    let map_type = TilemapType::default();
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(map_size);
    let mut tiles = Vec::with_capacity((W * H) as usize);
    for y in 0..map_size.y {
        for x in 0..map_size.x {
            let tile_pos = TilePos { x, y };
            let tile = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(0),
                    tilemap_id: TilemapId(tilemap_entity),
                    color: TileColor(Color::srgb(0.22, 0.25, 0.30)),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile);
            tiles.push(tile);
        }
    }
    let transform = get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0);
    commands.entity(tilemap_entity).insert((
        map_size,
        grid_size,
        tile_size,
        map_type,
        tile_storage,
        TilemapTexture::Single(tex),
        transform,
    ));
    commands.insert_resource(TileMap { tiles });
}

/// 权威步进节流：墙钟仅展示（权威时间=u64 tick，本计时器不进模拟）。
fn step_sim(
    time: Res<Time>,
    mut timer: ResMut<StepTimer>,
    mut sim: ResMut<Sim>,
    mut hist: ResMut<History>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        sim.0.run_tick();
        let s = sim.0.snapshot();
        hist.treasury.push(s.economy.treasury as f64);
        hist.active.push(s.traffic.vehicles.len() as f64);
        if hist.treasury.len() > 240 {
            hist.treasury.remove(0);
            hist.active.remove(0);
        }
    }
}

/// 只读渲染：底色复位 → Agent 位置（1mm→格）着色。
fn sync_tiles(sim: Res<Sim>, map: Res<TileMap>, mut tiles_q: Query<&mut TileColor>) {
    let s = sim.0.snapshot();
    for e in map.tiles.iter() {
        if let Ok(mut color) = tiles_q.get_mut(*e) {
            *color = TileColor(Color::srgb(0.22, 0.25, 0.30));
        }
    }
    for v in s.traffic.vehicles.iter() {
        let tx = (v.x as usize / 1000) % W as usize;
        let ty = (v.y as usize / 1000) % H as usize;
        if let Some(e) = map.tiles.get(ty * W as usize + tx) {
            if let Ok(mut color) = tiles_q.get_mut(*e) {
                *color = TileColor(Color::srgb(0.95, 0.55, 0.15));
            }
        }
    }
}

fn ui_panel(mut contexts: EguiContexts, hist: Res<History>, sim: Res<Sim>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::SidePanel::right("stats").show(ctx, |ui| {
        let s = sim.0.snapshot();
        ui.heading("CitySim-rs（非权威视图）");
        ui.label(format!("tick = {}", s.clock_tick));
        ui.label(format!("treasury = {}", s.economy.treasury));
        ui.label(format!("agents on road = {}", s.traffic.vehicles.len()));
        ui.label("注：墙钟仅展示节流；权威时间=u64 tick。");
        let plot = egui_plot::Plot::new("treasury").height(120.0);
        plot.show(ui, |plot_ui| {
            let line: egui_plot::PlotPoints = hist
                .treasury
                .iter()
                .enumerate()
                .map(|(i, v)| [i as f64, *v])
                .collect();
            plot_ui.line(egui_plot::Line::new("treasury", line));
            let line2: egui_plot::PlotPoints = hist
                .active
                .iter()
                .enumerate()
                .map(|(i, v)| [i as f64, *v])
                .collect();
            plot_ui.line(egui_plot::Line::new("agents", line2));
        });
    });
}
