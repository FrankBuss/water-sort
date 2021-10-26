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
}

impl GameButton {
    fn name(&self) -> String {
        match self {
            Self::Restart => "Restart".to_string(),
            Self::Solution => "Solution".to_string(),
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
    add_button(
        GameButton::Solution,
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
                    let solution = level.solve();
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
            };
        }
    }
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<ButtonMaterials>()
            .add_startup_system(init_ui.system())
            .add_system(button_system.system())
            .add_system(button_press_system.system());
    }
}