use bevy::prelude::*;
use bevy_mod_picking::{
    HighlightablePickingPlugin, InteractablePickingPlugin, PickableBundle, PickingCameraBundle,
    PickingEvent, PickingPlugin,
};

mod level;
use crate::level::*;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        // PickingPlugin provides core picking systems and must be registered first
        .add_plugin(PickingPlugin)
        // InteractablePickingPlugin adds mouse events and selection
        .add_plugin(InteractablePickingPlugin)
        // HighlightablePickingPlugin adds hover, click, and selection highlighting
        .add_plugin(HighlightablePickingPlugin)
        .add_startup_system(setup.system())
        .add_system(show_level.system())
        .add_system_to_stage(CoreStage::PostUpdate, print_events.system())
        .run();
}

fn print_events(mut events: EventReader<PickingEvent>) {
    for event in events.iter() {
        println!("This event happened! {:?}", event);
    }
}

fn show_level(query: Query<&Level>) {
    for level in query.iter() {
        //        println!("level size: {}", level.number_of_glasses());
    }
}

struct WaterPos {
    x: usize,
    y: usize,
}

/// set up a simple 3D scene
fn setup(
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

    // plane
    /*
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
     */

    let level = Level::load("level2.txt");
    let scale = 10.0 / (level.number_of_glasses() as f32);

    // cubes
    let box_size = scale * 0.6;
    let x_start = -5.0;
    let y_start = 0.0;
    let zp = 1.0;
    for x in 0..level.number_of_glasses() {
        let xp = x_start + (x as f32) * scale;
        let yp = y_start;
        let wall = 1.0 / 10.0 * scale;
        let color = Color::rgb(1.0, 1.0, 1.0);
        let mut add_box = |color: Color, x0: f32, y0: f32, z0: f32, x1: f32, y1: f32, z1: f32| {
            commands.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: x0,
                    min_y: y0,
                    min_z: z0,
                    max_x: x1,
                    max_y: y1,
                    max_z: z1,
                })),
                material: materials.add(color.into()),
                //transform: Transform::from_xyz(x, y, 0.0),
                ..Default::default()
            });
        };

        for y in 0..4 {
            if let Some(color) = level.get_color(x, y) {
                let wp = WaterPos { x: x, y: y };
                let yp = (wp.y as f32) * box_size + y_start;
                add_box(
                    color,
                    xp,
                    yp,
                    zp,
                    xp + box_size,
                    yp + box_size,
                    zp + box_size,
                );
            }
        }
        add_box(
            color,
            xp - wall,
            yp - wall,
            zp,
            xp,
            yp + 4.0 * box_size + wall,
            zp + box_size,
        );
        add_box(
            color,
            xp + box_size,
            yp - wall,
            zp,
            xp + box_size + wall,
            yp + 4.0 * box_size + wall,
            zp + box_size,
        );
        add_box(color, xp, yp - wall, zp, xp + box_size, yp, zp + box_size);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: xp - wall,
                    min_y: yp - wall,
                    min_z: zp,
                    max_x: xp + box_size + wall,
                    max_y: yp + 4.0 * box_size + wall,
                    max_z: zp + box_size,
                })),
                material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
                ..Default::default()
            })
            .insert_bundle(PickableBundle::default());
    }
    commands.spawn().insert(level);

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..Default::default()
    });
}
