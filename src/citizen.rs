use bevy::prelude::*;
use crate::town::{TownCell, ZoneType, BuildingType, TOWN_GRID_SIZE};
use crate::grid::Grid;
use crate::GameState;
use rand::prelude::*;

pub struct CitizenPlugin;

impl Plugin for CitizenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_citizens,
                update_citizens,
                spawn_vehicles,
                update_vehicles,
            ).run_if(in_state(GameState::TownView)),
        );
    }
}

// Citizen component
#[derive(Component)]
pub struct Citizen {
    pub home: IVec2,
    pub workplace: Option<IVec2>,
    pub destination: IVec2,
    pub state: CitizenState,
    pub happiness: f32,
    pub timer: Timer,
}

// Citizen state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CitizenState {
    AtHome,
    GoingToWork,
    AtWork,
    GoingHome,
    Shopping,
}

// Vehicle component
#[derive(Component)]
pub struct Vehicle {
    pub start: IVec2,
    pub destination: IVec2,
    pub path: Vec<IVec2>,
    pub path_index: usize,
    pub speed: f32,
}

// Spawn citizens based on residential zones
fn spawn_citizens(
    mut commands: Commands,
    town_cells: Query<&TownCell>,
    citizens: Query<&Citizen>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize timer if needed
    if timer.duration() == Duration::ZERO {
        *timer = Timer::from_seconds(2.0, TimerMode::Repeating);
    }
    
    // Only spawn citizens periodically
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    
    // Find residential zones
    let residential_zones: Vec<IVec2> = town_cells
        .iter()
        .filter(|cell| cell.zone == ZoneType::Residential)
        .map(|cell| cell.position)
        .collect();
    
    // Find commercial and industrial zones for workplaces
    let workplaces: Vec<IVec2> = town_cells
        .iter()
        .filter(|cell| cell.zone == ZoneType::Commercial || cell.zone == ZoneType::Industrial)
        .map(|cell| cell.position)
        .collect();
    
    // Don't spawn more citizens than we have residential capacity
    let max_citizens = residential_zones.len() * 5; // Each residential zone can house 5 citizens
    if citizens.iter().count() >= max_citizens {
        return;
    }
    
    // Spawn new citizens in residential zones
    let mut rng = rand::thread_rng();
    if !residential_zones.is_empty() {
        // Randomly select a residential zone
        let home = residential_zones[rng.gen_range(0..residential_zones.len())];
        
        // Assign a workplace if available
        let workplace = if !workplaces.is_empty() {
            Some(workplaces[rng.gen_range(0..workplaces.len())])
        } else {
            None
        };
        
        // Spawn the citizen
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.9, 0.9, 0.9),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    (home.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    (home.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    1.0,
                )),
                ..default()
            },
            Citizen {
                home,
                workplace,
                destination: home,
                state: CitizenState::AtHome,
                happiness: 0.5,
                timer: Timer::from_seconds(rng.gen_range(5.0..15.0), TimerMode::Once),
            },
        ));
    }
}

// Update citizen behavior
fn update_citizens(
    mut commands: Commands,
    time: Res<Time>,
    mut citizens: Query<(Entity, &mut Citizen, &mut Transform)>,
    town_cells: Query<&TownCell>,
) {
    let mut rng = rand::thread_rng();
    
    for (entity, mut citizen, mut transform) in citizens.iter_mut() {
        // Update timer
        citizen.timer.tick(time.delta());
        
        // Handle citizen state
        match citizen.state {
            CitizenState::AtHome => {
                if citizen.timer.just_finished() {
                    // Decide what to do next
                    if citizen.workplace.is_some() && rng.gen_bool(0.7) {
                        // Go to work
                        citizen.destination = citizen.workplace.unwrap();
                        citizen.state = CitizenState::GoingToWork;
                    } else {
                        // Go shopping
                        let commercial_zones: Vec<IVec2> = town_cells
                            .iter()
                            .filter(|cell| cell.zone == ZoneType::Commercial)
                            .map(|cell| cell.position)
                            .collect();
                        
                        if !commercial_zones.is_empty() {
                            citizen.destination = commercial_zones[rng.gen_range(0..commercial_zones.len())];
                            citizen.state = CitizenState::Shopping;
                        }
                    }
                    
                    // Set a new timer for the next activity
                    citizen.timer = Timer::from_seconds(rng.gen_range(5.0..10.0), TimerMode::Once);
                }
            }
            CitizenState::GoingToWork => {
                // Move towards workplace
                let workplace_pos = Vec3::new(
                    (citizen.destination.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    (citizen.destination.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    1.0,
                );
                
                let direction = (workplace_pos - transform.translation).normalize();
                transform.translation += direction * 20.0 * time.delta_seconds();
                
                // Check if arrived
                if transform.translation.distance(workplace_pos) < 5.0 {
                    transform.translation = workplace_pos;
                    citizen.state = CitizenState::AtWork;
                    citizen.timer = Timer::from_seconds(rng.gen_range(20.0..40.0), TimerMode::Once);
                }
            }
            CitizenState::AtWork => {
                if citizen.timer.just_finished() {
                    // Go home after work
                    citizen.destination = citizen.home;
                    citizen.state = CitizenState::GoingHome;
                    citizen.timer = Timer::from_seconds(rng.gen_range(5.0..10.0), TimerMode::Once);
                }
            }
            CitizenState::GoingHome => {
                // Move towards home
                let home_pos = Vec3::new(
                    (citizen.home.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    (citizen.home.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    1.0,
                );
                
                let direction = (home_pos - transform.translation).normalize();
                transform.translation += direction * 20.0 * time.delta_seconds();
                
                // Check if arrived
                if transform.translation.distance(home_pos) < 5.0 {
                    transform.translation = home_pos;
                    citizen.state = CitizenState::AtHome;
                    citizen.timer = Timer::from_seconds(rng.gen_range(10.0..30.0), TimerMode::Once);
                }
            }
            CitizenState::Shopping => {
                // Move towards shopping destination
                let shop_pos = Vec3::new(
                    (citizen.destination.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    (citizen.destination.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                    1.0,
                );
                
                let direction = (shop_pos - transform.translation).normalize();
                transform.translation += direction * 20.0 * time.delta_seconds();
                
                // Check if arrived
                if transform.translation.distance(shop_pos) < 5.0 {
                    // Shop for a while, then go home
                    citizen.destination = citizen.home;
                    citizen.state = CitizenState::GoingHome;
                    citizen.timer = Timer::from_seconds(rng.gen_range(5.0..10.0), TimerMode::Once);
                }
            }
        }
    }
}

// Spawn vehicles based on citizen movement
fn spawn_vehicles(
    mut commands: Commands,
    citizens: Query<&Citizen>,
    town_cells: Query<&TownCell>,
    vehicles: Query<&Vehicle>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    // Initialize timer if needed
    if timer.duration() == Duration::ZERO {
        *timer = Timer::from_seconds(3.0, TimerMode::Repeating);
    }
    
    // Only spawn vehicles periodically
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }
    
    // Find citizens who are traveling
    let traveling_citizens: Vec<&Citizen> = citizens
        .iter()
        .filter(|c| c.state == CitizenState::GoingToWork || c.state == CitizenState::GoingHome || c.state == CitizenState::Shopping)
        .collect();
    
    // Don't spawn too many vehicles
    let max_vehicles = 10;
    if vehicles.iter().count() >= max_vehicles || traveling_citizens.is_empty() {
        return;
    }
    
    // Find road cells
    let road_cells: Vec<&TownCell> = town_cells
        .iter()
        .filter(|cell| cell.building == BuildingType::Road)
        .collect();
    
    if road_cells.is_empty() {
        return;
    }
    
    // Randomly select a traveling citizen
    let mut rng = rand::thread_rng();
    let citizen = traveling_citizens[rng.gen_range(0..traveling_citizens.len())];
    
    // Find nearest road to start and destination
    let start_road = find_nearest_road(&road_cells, citizen.home);
    let dest_road = find_nearest_road(&road_cells, citizen.destination);
    
    if let (Some(start), Some(dest)) = (start_road, dest_road) {
        // Find a path along roads
        let is_road = |pos: IVec2| -> bool {
            road_cells.iter().any(|cell| cell.position == pos)
        };
        
        if let Some(path) = Grid::find_path(start.position, dest.position, is_road, TOWN_GRID_SIZE) {
            if !path.is_empty() {
                // Spawn a vehicle
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.8, 0.2, 0.2),
                            custom_size: Some(Vec2::new(6.0, 3.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            (start.position.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                            (start.position.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
                            0.5,
                        )),
                        ..default()
                    },
                    Vehicle {
                        start: start.position,
                        destination: dest.position,
                        path,
                        path_index: 0,
                        speed: rng.gen_range(30.0..50.0),
                    },
                ));
            }
        }
    }
}

// Update vehicle movement
fn update_vehicles(
    mut commands: Commands,
    time: Res<Time>,
    mut vehicles: Query<(Entity, &mut Vehicle, &mut Transform)>,
) {
    for (entity, mut vehicle, mut transform) in vehicles.iter_mut() {
        if vehicle.path_index >= vehicle.path.len() - 1 {
            // Vehicle has reached its destination, despawn it
            commands.entity(entity).despawn();
            continue;
        }
        
        // Get current and next positions in the path
        let current = vehicle.path[vehicle.path_index];
        let next = vehicle.path[vehicle.path_index + 1];
        
        // Convert to world positions
        let current_pos = Vec3::new(
            (current.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
            (current.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
            0.5,
        );
        let next_pos = Vec3::new(
            (next.x as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
            (next.y as f32 - TOWN_GRID_SIZE as f32 / 2.0) * 12.0,
            0.5,
        );
        
        // Calculate direction and move
        let direction = (next_pos - current_pos).normalize();
        transform.translation += direction * vehicle.speed * time.delta_seconds();
        
        // Rotate the vehicle to face the direction of travel
        let angle = direction.y.atan2(direction.x);
        transform.rotation = Quat::from_rotation_z(angle);
        
        // Check if reached the next point in the path
        if transform.translation.distance(next_pos) < 2.0 {
            vehicle.path_index += 1;
        }
    }
}

// Helper function to find the nearest road to a position
fn find_nearest_road<'a>(road_cells: &[&'a TownCell], position: IVec2) -> Option<&'a TownCell> {
    road_cells
        .iter()
        .min_by_key(|cell| Grid::manhattan_distance(cell.position, position))
        .copied()
}
