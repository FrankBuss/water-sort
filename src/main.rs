use bevy::prelude::*;
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingPlugin, Selection,
};

mod level;
use crate::level::*;

struct GlassIndex {
    i: usize,
}

struct FirstSelectedGlass {
    i: Option<usize>,
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
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level: &Level,
) {
    let scale = 10.0 / (level.number_of_glasses() as f32);
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

    let level = Level::load("level2.txt");
    show_level(&mut commands, &mut meshes, &mut materials, &level);
    commands.spawn().insert(level);

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(3.0, 8.0, 5.0),
        ..Default::default()
    });
}

fn select_glass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut glasses_query: Query<(&GlassIndex, &mut Selection, &mut Transform)>,
    mut level_query: Query<&mut Level>,
    mut first: ResMut<FirstSelectedGlass>,
    entities: Query<Entity, With<GlassIndex>>,
) {
    let mut level = level_query.single_mut().expect("level missing");
    for (glass, mut selection, mut transform) in glasses_query.iter_mut() {
        if selection.selected() {
            if let Some(from) = first.i {
                let to = glass.i;
                if to != from {
                    first.i = None;
                    level.move_water(from, to);
                    for entity in entities.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    show_level(&mut commands, &mut meshes, &mut materials, &level);
                    selection.set_selected(false);
                }
            } else {
                first.i = Some(glass.i);
            }
        }
        transform.translation.y = if selection.selected() { 0.2 } else { 0.0 }
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Water Move".to_string(),
            width: 800.0,
            height: 400.0,
            ..Default::default()
        })
        .insert_resource(FirstSelectedGlass { i: None })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_startup_system(setup.system())
        .add_system(select_glass.system())
        .run();
}
