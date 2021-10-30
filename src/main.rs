use std::io::{self, Write};

use std::time::{Duration, Instant};

use bevy::{prelude::*, window::WindowMode};
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingPlugin, Selection,
};

mod ui;
use ui::*;

mod level;
use level::*;

struct GlassIndex {
    i: usize,
}

struct FirstSelectedGlass {
    i: Option<usize>,
}

/*
fn exit_system(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit);
}
*/

fn main() {
    let mut app = App::build();
    app.insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins);
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
//    app.add_startup_system(setup.system()).run();


    app.insert_resource(WindowDescriptor {
        title: "Water Sort".to_string(),
        mode: WindowMode::Windowed,
        width: 1200.0,
        height: 600.0,
        ..Default::default()
    })
    .add_plugin(PickingPlugin)
    .add_plugin(InteractablePickingPlugin)
    .add_plugin(UIPlugin)
    .insert_resource(FirstSelectedGlass { i: None })
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_startup_system(setup.system())
    .add_system(select_glass.system())
    .insert_resource(Autoplay {
        timer: Timer::from_seconds(0.2, true),
        moves: Vec::new(),
        running: false,
        select_first: true,
    })
    .add_system(autoplay.system())
    .add_system(bevy::input::system::exit_on_esc_system.system())
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
    let mut scale = 10.0 / (level.number_of_glasses() as f32);
    if scale > 1.0 {
        scale = 1.0;
    }
    let box_size = scale * 0.6;
    let x_start = -5.0;
    let y_start = 0.0;
    let zp = 1.0;
    for x in 0..level.number_of_glasses() {
        let xp = x_start + (x as f32) * scale;
        let yp = y_start;
        let wall = 1.0 / 10.0 * scale;
        let color = Color::rgb(1.0, 1.0, 1.0);
        let mut bounding_box = commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box {
                min_x: -wall,
                min_y: -wall,
                min_z: 0.0,
                max_x: box_size + wall,
                max_y: 4.0 * box_size + wall,
                max_z: box_size,
            })),
            transform: Transform::from_xyz(xp, yp, zp),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
            ..Default::default()
        });
        bounding_box.insert_bundle(PickableBundle::default());
        bounding_box.insert(GlassIndex { i: x });
        let parent_id = bounding_box.id();

        // boxes
        for y in 0..4 {
            if let Some(color) = level.get_color(x, y) {
                let yp = (y as f32) * box_size;
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
            4.0 * box_size + wall,
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
            4.0 * box_size + wall,
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
) {
    // set up the camera
    let mut camera = PerspectiveCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(1.0, 2.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands
        .spawn_bundle(camera)
        .insert_bundle(PickingCameraBundle::default());

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..Default::default()
    });

    // create and show first level
    let level = Level::load(0);
    show_level(
        &entities,
        &mut commands,
        &mut meshes,
        &mut materials,
        &level,
    );
    commands.spawn().insert(level);
}

fn move_water(level: &mut Level, from: usize, to: usize) {
    level.move_water(from, to);
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
    let mut level = level_query.single_mut().expect("level missing");
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
        transform.translation.y = if selection.selected() { 0.2 } else { 0.0 }
    }
}

struct Move {
    from: usize,
    to: usize,
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
    if autoplay.running {
        if autoplay.moves.len() == 0 {
            autoplay.running = false;
        } else {
            autoplay
                .timer
                .tick(Duration::from_secs_f32(time.delta_seconds()));
            if !autoplay.timer.just_finished() {
                return;
            }

            let mut m: usize = autoplay.moves[0].from;
            if autoplay.select_first {
                for (_glass, mut selection, _transform) in glasses_query.iter_mut() {
                    if m == 0 as usize {
                        selection.set_selected(true);
                        break;
                    }
                    m -= 1;
                }
                autoplay.select_first = false;
            } else {
                let m = autoplay.moves.remove(0);
                let mut level = level_query.single_mut().expect("level missing");
                move_water(&mut level, m.from, m.to);
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
    }
}

#[test]
fn benchmark() {
    println!("level,time,level size,number of configurations,solution length");
    for i in 0..1000000 {
        let start = Instant::now();
        let mut level = Level::load(i);
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
