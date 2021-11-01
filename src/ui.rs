use crate::*;
use bevy::ecs::system::EntityCommands;

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}

enum GameButton {
    Restart,
    Solution,
    Size3,
    Size4,
    Size5,
}

struct LevelInfoMarker;

impl GameButton {
    fn name(&self) -> String {
        match self {
            Self::Restart => "Restart".to_string(),
            Self::Solution => "Solution".to_string(),
            Self::Size3 => "3".to_string(),
            Self::Size4 => "4".to_string(),
            Self::Size5 => "5".to_string(),
        }
    }
}

fn add_button(
    button: GameButton,
    menu_bar: &mut EntityCommands,
    asset_server: &ResMut<AssetServer>,
    button_materials: &Res<ButtonMaterials>,
) {
    menu_bar.with_children(|parent| {
        parent
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    // center button
                    margin: Rect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                material: button_materials.normal.clone(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        button.name(),
                        TextStyle {
                            font: asset_server.load("fonts/DejaVuSans.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                });
            })
            .insert(button);
    });
}

fn init_ui(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    button_materials: Res<ButtonMaterials>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    let mut menu_bar = commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(70.0)),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            ..Default::default()
        },
        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
        ..Default::default()
    });
    add_button(
        GameButton::Restart,
        &mut menu_bar,
        &asset_server,
        &button_materials,
    );

    menu_bar.with_children(|parent| {
        parent
            .spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Level".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/DejaVuSans.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.5, 0.5, 1.0),
                        },
                    }],
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(LevelInfoMarker {});
    });

    add_button(
        GameButton::Solution,
        &mut menu_bar,
        &asset_server,
        &button_materials,
    );

    add_button(
        GameButton::Size3,
        &mut menu_bar,
        &asset_server,
        &button_materials,
    );

    add_button(
        GameButton::Size4,
        &mut menu_bar,
        &asset_server,
        &button_materials,
    );

    add_button(
        GameButton::Size5,
        &mut menu_bar,
        &asset_server,
        &button_materials,
    );
}

fn button_system(
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (&Interaction, &mut Handle<ColorMaterial>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut material, _children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *material = button_materials.pressed.clone();
            }
            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            }
            Interaction::None => {
                *material = button_materials.normal.clone();
            }
        }
    }
}

fn set_size(
    size: usize,
    entities: &Query<Entity, With<GlassIndex>>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level: &mut Level,
) {
    level.glass_height = size;
    level.resize(size);
    show_level(
        entities,
        commands,
        meshes,
        materials,
        level,
    );
}

fn button_press_system(
    query: Query<(&Interaction, &GameButton), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level_query: Query<&mut Level>,
    entities: Query<Entity, With<GlassIndex>>,
    mut autoplay: ResMut<Autoplay>,
) {
    let mut level = level_query.single_mut().expect("level missing");
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Clicked {
            match button {
                GameButton::Restart => {
                    if !autoplay.running {
                        level.restart();
                        show_level(
                            &entities,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &level,
                        );
                    }
                }
                GameButton::Solution => {
                    level.restart();
                    show_level(
                        &entities,
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &level,
                    );
                    let (solution, _len) = level.solve();
                    /*
                    println!("{} combinations tested", len);
                    if solution.len() > 0 {
                        println!("solution: {}", solution);
                    } else {
                        println!("no solution found");
                    }
                    */
                    let sb = solution.as_bytes();
                    for i in 0..(sb.len() / 2) {
                        let from = sb[2 * i] - b'a';
                        let to = sb[2 * i + 1] - b'a';
                        let m = Move {
                            from: from as usize,
                            to: to as usize,
                        };
                        autoplay.moves.push(m);
                    }
                    autoplay.running = true;
                }
                GameButton::Size3 => {
                    if !autoplay.running {
                        set_size(3,
                            &entities,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut level,
                        );
                    }
                }
                GameButton::Size4 => {
                    if !autoplay.running {
                        set_size(4,
                            &entities,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut level,
                        );
                    }
                }
                GameButton::Size5 => {
                    if !autoplay.running {
                        set_size(5,
                            &entities,
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &mut level,
                        );
                    }
                }
            };
        }
    }
}

fn level_info(level_query: Query<&Level>, mut query: Query<(&mut Text, &LevelInfoMarker)>) {
    let level = level_query.single().expect("level missing");
    let (mut text, _marker) = query.single_mut().unwrap();
    text.sections[0].value = format!("Level: {}", level.number + 1);
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_startup_system(init_ui.system())
            .add_system(button_system.system())
            .add_system(button_press_system.system())
            .add_system(level_info.system());
    }
}
