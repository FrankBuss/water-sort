/*use bevy::prelude::*;
use bevy_mod_picking::{
    DebugCursorPickingPlugin, DebugEventsPickingPlugin, DefaultPickingPlugins, PickableBundle,
    PickingCameraBundle,
};
*/



use std::fs::File;
use std::io::prelude::*;
use std::io::{self, Write};
use std::process;

use std::time::{Duration, Instant};

use bevy::utils::tracing::Subscriber;
use bevy::{prelude::*, window::WindowMode};
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingPlugin, Selection,
};
mod ui;
use ui::*;


/*
mod game_ui;
use game_ui::*;
*/

mod level;
use level::*;

#[derive(Component)]
struct GlassIndex {
    i: usize,
}

struct FirstSelectedGlass {
    i: Option<usize>,
}

#[derive(Default)]
struct FPSCounter(Timer, u32);

/*
fn exit_system(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}
*/

fn save_levels(glass_height: usize) {
    let filename = Level::create_levels_filename(glass_height);
    let mut file = File::create(filename).unwrap();
    for level_number in 0..10 {
        let level = Level::create(level_number, glass_height);
        level.save_to_file(&mut file);
    }
}

fn main() {
    /*
    App::new()
        .insert_resource(WindowDescriptor {
            vsync: false, // Disabled for this demo to reduce input latency
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins) // <- Adds Picking, Interaction, and Highlighting plugins.
        .add_plugin(DebugCursorPickingPlugin) // <- Adds the green debug cursor.
        .add_plugin(DebugEventsPickingPlugin) // <- Adds debug event logging.
        .add_startup_system(setup)
        .run();*/

        App::new()
        .insert_resource(Msaa { samples: 4 })
            .add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            title: "Water Sort".to_string(),
            mode: WindowMode::Windowed,
            width: 1200.0,
            height: 600.0,
            vsync: false,
            ..Default::default()
        })
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(UIPlugin)
        //.add_plugin(GameUIPlugin)
        .insert_resource(FirstSelectedGlass { i: None })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_startup_system(setup)
        .add_system(select_glass)
        .insert_resource(Autoplay {
            timer: Timer::from_seconds(0.2, true),
            moves: Vec::new(),
            running: false,
            select_first: true,
        })
        .add_system(autoplay)
        .add_system(bevy::input::system::exit_on_esc_system)
        .insert_resource(FPSCounter(Timer::from_seconds(1.0, true), 0))
        .add_system(fps_counter)
        .run();
    
}

fn add_box(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent_id: Entity,
    color: Color,
    x0: f32,
    y0: f32,
    z0: f32,
    x1: f32,
    y1: f32,
    z1: f32,
) {
    let id = commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: x0,
                min_y: y0,
                min_z: z0,
                max_x: x1,
                max_y: y1,
                max_z: z1,
            })),
            material: materials.add(color.into()),
            ..Default::default()
        })
        .id();
    commands.entity(parent_id).push_children(&[id]);
}

fn show_level(
    entities: &Query<Entity, With<GlassIndex>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level: &Level,
) {
    // remove old graphics
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // create new graphics
    let scale = (10.0 / (level.number_of_glasses() as f32)).min(0.5);
    let box_size = scale * 0.6;
    let x_start = -5.0;
    let y_start = 0.0;
    let z_pos = 1.0;
    let float_glass_height = level.glass_height as f32;
    for x in 0..level.number_of_glasses() {
        let x_pos = x_start + (x as f32) * scale;
        let y_pos = y_start;
        let wall = 1.0 / 10.0 * scale;
        let box_gap = wall / 3.0;
        let color = Color::rgb(1.0, 1.0, 1.0);
        let mut bounding_box = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -wall,
                min_y: -wall,
                min_z: 0.0,
                max_x: box_size + wall,
                max_y: (float_glass_height + box_gap) * box_size + wall,
                max_z: box_size,
            })),
            transform: Transform::from_xyz(x_pos, y_pos, z_pos),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
            ..Default::default()
        });
        bounding_box.insert_bundle(PickableBundle::default());
        bounding_box.insert(GlassIndex { i: x });
        let parent_id = bounding_box.id();

        // boxes
        for y in 0..level.glass_height {
            if let Some(color) = level.get_color(x, y) {
                let yp = (y as f32) * (box_size + box_gap);
                add_box(
                    commands,
                    meshes,
                    materials,
                    parent_id,
                    color,
                    0.0,
                    yp,
                    0.0,
                    box_size,
                    yp + box_size,
                    box_size,
                );
            }
        }

        // frame
        add_box(
            commands,
            meshes,
            materials,
            parent_id,
            color,
            -wall,
            -wall,
            0.0,
            0.0,
            float_glass_height * (box_size + box_gap) + wall,
            box_size,
        );
        add_box(
            commands,
            meshes,
            materials,
            parent_id,
            color,
            box_size,
            -wall,
            0.0,
            box_size + wall,
            float_glass_height * (box_size + box_gap) + wall,
            box_size,
        );
        add_box(
            commands, meshes, materials, parent_id, color, 0.0, -wall, 0.0, box_size, 0.0, box_size,
        );
    }
}

fn setup(
    entities: Query<Entity, With<GlassIndex>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    // set up the camera
    let mut camera = PerspectiveCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(1.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands
        .spawn_bundle(camera)
        .insert_bundle(PickingCameraBundle::default());

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            intensity: 800.0, // Roughly a 60W non-halogen incandescent bulb
            range: 20.0,
            radius: 0.0,
            shadows_enabled: false,
            shadow_depth_bias: PointLight::DEFAULT_SHADOW_DEPTH_BIAS,
            shadow_normal_bias: PointLight::DEFAULT_SHADOW_NORMAL_BIAS,
        },
        ..Default::default()
    });

    // create and show first level
    let level = Level::load(0, 4);
    show_level(
        &entities,
        &mut commands,
        &mut meshes,
        &mut materials,
        &level,
    );
    commands.spawn().insert(level);

    let music = asset_server.load("sounds/jelsonic-another-brilliant-age.mp3");
    audio.play(music);
}

fn move_water(level: &mut Level, from: usize, to: usize) {
    level.move_water(from, to, true);
    //level.move_reverse(from, to, 2);
    if level.test_win() {
        level.load_next();
    }
}

fn select_glass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut glasses_query: Query<(&GlassIndex, &mut Selection, &mut Transform)>,
    mut level_query: Query<&mut Level>,
    mut first: ResMut<FirstSelectedGlass>,
    entities: Query<Entity, With<GlassIndex>>,
    autoplay: ResMut<Autoplay>,
) {
    let mut level = level_query.single_mut();
    for (glass, mut selection, mut transform) in glasses_query.iter_mut() {
        if !autoplay.running {
            if selection.selected() {
                if let Some(from) = first.i {
                    let to = glass.i;
                    if to != from {
                        first.i = None;
                        move_water(&mut level, from, to);
                        show_level(
                            &entities,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &level,
                        );
                        selection.set_selected(false);
                    }
                } else {
                    first.i = Some(glass.i);
                }
            }
        }
        transform.translation.y = selection.selected().then(|| 0.2).unwrap_or(0.0);
    }
}

struct Autoplay {
    timer: Timer,
    moves: Vec<Move>,
    running: bool,
    select_first: bool,
}

fn autoplay(
    mut autoplay: ResMut<Autoplay>,
    time: Res<Time>,
    entities: Query<Entity, With<GlassIndex>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level_query: Query<&mut Level>,
    mut glasses_query: Query<(&GlassIndex, &mut Selection, &Transform)>,
) {
    if !autoplay.running {
        return;
    }
    if autoplay.moves.is_empty() {
        autoplay.running = false;
        return;
    }
    autoplay
        .timer
        .tick(Duration::from_secs_f32(time.delta_seconds()));
    if !autoplay.timer.just_finished() {
        return;
    }

    let mut from = autoplay.moves[0].from;
    if autoplay.select_first {
        for (_glass, mut selection, _transform) in glasses_query.iter_mut() {
            if from == 0 {
                selection.set_selected(true);
                break;
            }
            from -= 1;
        }
        autoplay.select_first = false;
    } else {
        let move_pair = autoplay.moves.remove(0);
        let mut level = level_query.single_mut();
        move_water(&mut level, move_pair.from, move_pair.to);
        show_level(
            &entities,
            &mut commands,
            &mut meshes,
            &mut materials,
            &level,
        );
        autoplay.select_first = true;
    }
}

fn fps_counter(time: Res<Time>, mut timer: ResMut<FPSCounter>) {
    timer.0.tick(time.delta());
    timer.1 += 1;
    if timer.0.finished() {
        println!("fps: {}", timer.1);
        timer.1 = 0;
    }
}

/*
#[test]
fn benchmark() {
    let mut spaces = spaces;
    println!("level,time,level size,number of configurations,solution length");
    let glass_height = 8;
    for i in 0..1000000 {
        let start = Instant::now();
        let mut level = Level::load(i, glass_height);
        let elapsed = start.elapsed().as_secs_f32();
        let (keys, configurations) = level.solve();
        println!(
            "{},{},{},{},{}",
            i + 1,
            elapsed,
            level.number_of_glasses(),
            configurations,
            keys.len()
        );
        io::stdout().flush().unwrap();
    }
}
*/



