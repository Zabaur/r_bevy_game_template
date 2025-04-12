use bevy::prelude::*;
use crate::GameState;

pub struct TownPlugin;

/// This plugin handles the town view and simulation
impl Plugin for TownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::TownView), setup_town)
            .add_systems(
                Update,
                (
                    handle_town_interaction,
                    update_town_simulation,
                ).run_if(in_state(GameState::TownView)),
            )
            .add_systems(OnExit(GameState::TownView), cleanup_town);
    }
}

// Town grid size (finer than island grid)
pub const TOWN_GRID_SIZE: usize = 50;

// Zone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    None,
    Residential,
    Commercial,
    Industrial,
}

// Building types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    None,
    Road,
    TownHall,
    PowerPlant,
    WaterTower,
    Police,
    Fire,
    Hospital,
    School,
    Park,
    // Town Hall department modules
    LawAndOrder,  // Pentagon shape
    Education,    // Trapezoid shape
    Transportation, // Chevron shape
    Health,       // Cross shape
    Energy,       // Star shape
    Housing,      // Round shape
    SocialServices, // Hexagon shape
    Upgrade,      // Square shape (can be attached to any department)
}

// Town cell component
#[derive(Component)]
pub struct TownCell {
    pub position: IVec2,
    pub zone: ZoneType,
    pub building: BuildingType,
    pub accessible: bool,
}

// Town resource
#[derive(Resource)]
pub struct Town {
    pub grid: [[TownCell; TOWN_GRID_SIZE]; TOWN_GRID_SIZE],
    pub population: i32,
    pub happiness: f32,
    pub funds: i32,
    pub power: i32,
    pub water: i32,
}

// Citizen component
#[derive(Component)]
pub struct Citizen {
    pub home: IVec2,
    pub workplace: IVec2,
    pub happiness: f32,
}

// Vehicle component
#[derive(Component)]
pub struct Vehicle {
    pub start: IVec2,
    pub destination: IVec2,
    pub progress: f32,
}

// Setup the town view
fn setup_town(mut commands: Commands) {
    // Create a new town if it doesn't exist
    // In a real implementation, we would load the town data based on the selected town
    
    // Add a camera
    commands.spawn(Camera2dBundle::default());
    
    // Create a simple town grid
    for y in 0..TOWN_GRID_SIZE {
        for x in 0..TOWN_GRID_SIZE {
            let position = IVec2::new(x as i32, y as i32);
            
            // Create a town cell
            let cell = TownCell {
                position,
                zone: ZoneType::None,
                building: BuildingType::None,
                accessible: false,
            };
            
            // Spawn a sprite for each cell
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: get_cell_color(&cell),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        (x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                        (y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                        0.0,
                    )),
                    ..default()
                },
                cell,
            ));
        }
    }
    
    // Add UI for tools
    setup_town_ui(&mut commands);
}

// Setup town UI
fn setup_town_ui(commands: &mut Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.7).into(),
            ..default()
        })
        .with_children(|parent| {
            // Road tool
            create_tool_button(parent, "Road", BuildingType::Road);
            
            // Zone tools
            create_zone_button(parent, "R", ZoneType::Residential, Color::rgb(0.0, 0.8, 0.0));
            create_zone_button(parent, "C", ZoneType::Commercial, Color::rgb(0.0, 0.0, 0.8));
            create_zone_button(parent, "I", ZoneType::Industrial, Color::rgb(0.8, 0.8, 0.0));
            
            // Building tools
            create_tool_button(parent, "Town Hall", BuildingType::TownHall);
            create_tool_button(parent, "Power", BuildingType::PowerPlant);
            create_tool_button(parent, "Water", BuildingType::WaterTower);
            
            // Back to island view button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgb(0.8, 0.2, 0.2).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ));
                });
        });
}

// Create a tool button
fn create_tool_button(parent: &mut ChildBuilder, name: &str, building_type: BuildingType) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(0.3, 0.3, 0.3).into(),
                ..default()
            },
            ToolButton { building_type, zone_type: ZoneType::None },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                name,
                TextStyle {
                    font_size: 16.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

// Create a zone button
fn create_zone_button(parent: &mut ChildBuilder, name: &str, zone_type: ZoneType, color: Color) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: color.into(),
                ..default()
            },
            ToolButton { building_type: BuildingType::None, zone_type },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                name,
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });
}

// Tool button component
#[derive(Component)]
struct ToolButton {
    building_type: BuildingType,
    zone_type: ZoneType,
}

// Currently selected tool
#[derive(Resource, Default)]
struct SelectedTool {
    building_type: Option<BuildingType>,
    zone_type: Option<ZoneType>,
}

// Handle town interaction
fn handle_town_interaction(
    mut commands: Commands,
    mut town_cells: Query<(&mut Sprite, &mut TownCell)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    tool_buttons: Query<(&Interaction, &ToolButton), (Changed<Interaction>, With<Button>)>,
    mut selected_tool: Local<SelectedTool>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Handle tool selection
    for (interaction, tool_button) in tool_buttons.iter() {
        if *interaction == Interaction::Pressed {
            if tool_button.building_type != BuildingType::None {
                selected_tool.building_type = Some(tool_button.building_type);
                selected_tool.zone_type = None;
            } else if tool_button.zone_type != ZoneType::None {
                selected_tool.zone_type = Some(tool_button.zone_type);
                selected_tool.building_type = None;
            }
        }
    }
    
    // Handle mouse clicks
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera_q.single();
        
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                // Convert world position to grid position
                let grid_x = ((world_position.x / 12.0) + TOWN_GRID_SIZE as f32 / 2.0).floor() as i32;
                let grid_y = ((world_position.y / 12.0) + TOWN_GRID_SIZE as f32 / 2.0).floor() as i32;
                
                // Check if the position is within the grid
                if grid_x >= 0 && grid_x < TOWN_GRID_SIZE as i32 && grid_y >= 0 && grid_y < TOWN_GRID_SIZE as i32 {
                    // Apply the selected tool to the cell
                    for (mut sprite, mut cell) in town_cells.iter_mut() {
                        if cell.position.x == grid_x && cell.position.y == grid_y {
                            if let Some(building_type) = selected_tool.building_type {
                                cell.building = building_type;
                                cell.zone = ZoneType::None;
                            } else if let Some(zone_type) = selected_tool.zone_type {
                                cell.zone = zone_type;
                                // Only clear the building if it's not a road
                                if cell.building != BuildingType::Road {
                                    cell.building = BuildingType::None;
                                }
                            }
                            
                            // Update the cell color
                            sprite.color = get_cell_color(&cell);
                        }
                    }
                }
            }
        }
    }
    
    // Handle back button
    if mouse_button_input.just_pressed(MouseButton::Right) {
        next_state.set(GameState::IslandView);
    }
}

// Update town simulation
fn update_town_simulation(time: Res<Time>, mut town_cells: Query<(&mut Sprite, &mut TownCell)>) {
    // This would be where we update the simulation
    // For now, we'll just update the colors of cells with zones to simulate development
    
    // Only update every 0.5 seconds
    if (time.elapsed_seconds() * 2.0).floor() % 2.0 != 0.0 {
        return;
    }
    
    for (mut sprite, cell) in town_cells.iter_mut() {
        if cell.zone != ZoneType::None && cell.building == BuildingType::None {
            // Randomly update some cells to simulate development
            if rand::random::<f32>() < 0.01 {
                sprite.color = match cell.zone {
                    ZoneType::Residential => Color::rgb(0.0, 0.7, 0.0),
                    ZoneType::Commercial => Color::rgb(0.0, 0.0, 0.7),
                    ZoneType::Industrial => Color::rgb(0.7, 0.7, 0.0),
                    ZoneType::None => Color::rgb(0.2, 0.2, 0.2),
                };
            }
        }
    }
}

// Clean up the town view
fn cleanup_town(mut commands: Commands, query: Query<Entity, With<TownCell>>, ui: Query<Entity, With<Node>>, camera: Query<Entity, With<Camera2d>>) {
    // Remove all town cells
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Remove UI
    for entity in ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Remove the camera
    for entity in camera.iter() {
        commands.entity(entity).despawn();
    }
}

// Helper function to get the color for a cell based on its zone and building
fn get_cell_color(cell: &TownCell) -> Color {
    match cell.building {
        BuildingType::None => {
            match cell.zone {
                ZoneType::None => Color::rgb(0.2, 0.2, 0.2),
                ZoneType::Residential => Color::rgb(0.0, 0.5, 0.0),
                ZoneType::Commercial => Color::rgb(0.0, 0.0, 0.5),
                ZoneType::Industrial => Color::rgb(0.5, 0.5, 0.0),
            }
        }
        BuildingType::Road => Color::rgb(0.3, 0.3, 0.3),
        BuildingType::TownHall => Color::rgb(0.8, 0.2, 0.2),
        BuildingType::PowerPlant => Color::rgb(0.8, 0.8, 0.0),
        BuildingType::WaterTower => Color::rgb(0.0, 0.5, 0.8),
        BuildingType::Police => Color::rgb(0.0, 0.0, 0.8),
        BuildingType::Fire => Color::rgb(0.8, 0.0, 0.0),
        BuildingType::Hospital => Color::rgb(0.8, 0.0, 0.8),
        BuildingType::School => Color::rgb(0.0, 0.8, 0.8),
        BuildingType::Park => Color::rgb(0.0, 0.8, 0.0),
        BuildingType::LawAndOrder => Color::rgb(0.5, 0.0, 0.5),
        BuildingType::Education => Color::rgb(0.0, 0.5, 0.5),
        BuildingType::Transportation => Color::rgb(0.5, 0.5, 0.0),
        BuildingType::Health => Color::rgb(0.8, 0.0, 0.0),
        BuildingType::Energy => Color::rgb(0.8, 0.8, 0.0),
        BuildingType::Housing => Color::rgb(0.0, 0.0, 0.8),
        BuildingType::SocialServices => Color::rgb(0.0, 0.8, 0.0),
        BuildingType::Upgrade => Color::rgb(0.5, 0.5, 0.5),
    }
}
