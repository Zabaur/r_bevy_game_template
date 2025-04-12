use bevy::prelude::*;
use crate::GameState;

pub struct IslandPlugin;

/// This plugin handles the island map view and functionality
impl Plugin for IslandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::IslandView), setup_island)
            .add_systems(
                Update,
                (
                    handle_island_interaction,
                ).run_if(in_state(GameState::IslandView)),
            )
            .add_systems(OnExit(GameState::IslandView), cleanup_island);
    }
}

// Island grid size
pub const ISLAND_GRID_SIZE: usize = 20;

// Island cell types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IslandCellType {
    Water,
    Land,
    Forest,
    Mountain,
    Town,
}

// Island cell component
#[derive(Component)]
pub struct IslandCell {
    pub position: IVec2,
    pub cell_type: IslandCellType,
    pub owned: bool,
}

// Island resource
#[derive(Resource)]
pub struct Island {
    pub grid: [[IslandCellType; ISLAND_GRID_SIZE]; ISLAND_GRID_SIZE],
    pub owned_cells: Vec<IVec2>,
    pub towns: Vec<IVec2>,
}

impl Default for Island {
    fn default() -> Self {
        // Create a default island with water around the edges and some land in the middle
        let mut grid = [[IslandCellType::Water; ISLAND_GRID_SIZE]; ISLAND_GRID_SIZE];
        
        // Create some land in the middle
        for x in 5..15 {
            for y in 5..15 {
                grid[y][x] = IslandCellType::Land;
                
                // Add some variety
                if (x + y) % 7 == 0 {
                    grid[y][x] = IslandCellType::Forest;
                }
                if (x * y) % 13 == 0 {
                    grid[y][x] = IslandCellType::Mountain;
                }
            }
        }
        
        Island {
            grid,
            owned_cells: Vec::new(),
            towns: Vec::new(),
        }
    }
}

// Setup the island view
fn setup_island(mut commands: Commands, mut island: Option<ResMut<Island>>) {
    // If the island doesn't exist yet, create it
    if island.is_none() {
        commands.insert_resource(Island::default());
    }
    
    // Create the island grid visualization
    for y in 0..ISLAND_GRID_SIZE {
        for x in 0..ISLAND_GRID_SIZE {
            let position = IVec2::new(x as i32, y as i32);
            let cell_type = island
                .as_ref()
                .map(|i| i.grid[y][x])
                .unwrap_or(IslandCellType::Water);
            
            let owned = island
                .as_ref()
                .map(|i| i.owned_cells.contains(&position))
                .unwrap_or(false);
            
            // Spawn a sprite for each cell
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: get_cell_color(cell_type, owned),
                        custom_size: Some(Vec2::new(30.0, 30.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(
                        (x as f32 - ISLAND_GRID_SIZE as f32 / 2.0) * 32.0,
                        (y as f32 - ISLAND_GRID_SIZE as f32 / 2.0) * 32.0,
                        0.0,
                    )),
                    ..default()
                },
                IslandCell {
                    position,
                    cell_type,
                    owned,
                },
            ));
        }
    }
    
    // Add a camera
    commands.spawn(Camera2dBundle::default());
}

// Handle island interaction (clicking on cells, etc.)
fn handle_island_interaction(
    mut commands: Commands,
    mut island: ResMut<Island>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut cells: Query<(&mut Sprite, &IslandCell)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Handle mouse clicks
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera_q.single();
        
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                // Convert world position to grid position
                let grid_x = ((world_position.x / 32.0) + ISLAND_GRID_SIZE as f32 / 2.0).floor() as i32;
                let grid_y = ((world_position.y / 32.0) + ISLAND_GRID_SIZE as f32 / 2.0).floor() as i32;
                
                // Check if the position is within the grid
                if grid_x >= 0 && grid_x < ISLAND_GRID_SIZE as i32 && grid_y >= 0 && grid_y < ISLAND_GRID_SIZE as i32 {
                    let position = IVec2::new(grid_x, grid_y);
                    let cell_type = island.grid[grid_y as usize][grid_x as usize];
                    
                    // Handle cell interaction based on cell type
                    match cell_type {
                        IslandCellType::Land | IslandCellType::Forest => {
                            // If it's land and not owned, purchase it
                            if !island.owned_cells.contains(&position) {
                                island.owned_cells.push(position);
                                
                                // Update the cell color
                                for (mut sprite, cell) in cells.iter_mut() {
                                    if cell.position == position {
                                        sprite.color = get_cell_color(cell_type, true);
                                    }
                                }
                            } else if !island.towns.contains(&position) {
                                // If it's owned land without a town, found a new town
                                island.towns.push(position);
                                island.grid[grid_y as usize][grid_x as usize] = IslandCellType::Town;
                                
                                // Update the cell color
                                for (mut sprite, cell) in cells.iter_mut() {
                                    if cell.position == position {
                                        sprite.color = get_cell_color(IslandCellType::Town, true);
                                    }
                                }
                                
                                // TODO: Store the selected town and transition to town view
                                next_state.set(GameState::TownView);
                            }
                        }
                        IslandCellType::Town => {
                            // If it's a town, enter town view
                            // TODO: Store the selected town
                            next_state.set(GameState::TownView);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// Clean up the island view
fn cleanup_island(mut commands: Commands, query: Query<Entity, With<IslandCell>>, camera: Query<Entity, With<Camera2d>>) {
    // Remove all island cells
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Remove the camera
    for entity in camera.iter() {
        commands.entity(entity).despawn();
    }
}

// Helper function to get the color for a cell based on its type and ownership
fn get_cell_color(cell_type: IslandCellType, owned: bool) -> Color {
    match cell_type {
        IslandCellType::Water => Color::rgb(0.0, 0.3, 0.8),
        IslandCellType::Land => {
            if owned {
                Color::rgb(0.2, 0.8, 0.2)
            } else {
                Color::rgb(0.5, 0.5, 0.2)
            }
        }
        IslandCellType::Forest => {
            if owned {
                Color::rgb(0.0, 0.6, 0.0)
            } else {
                Color::rgb(0.0, 0.4, 0.0)
            }
        }
        IslandCellType::Mountain => Color::rgb(0.5, 0.3, 0.2),
        IslandCellType::Town => Color::rgb(0.8, 0.2, 0.2),
    }
}
