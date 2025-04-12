use bevy::prelude::*;
use crate::town::{Town, TownCell, ZoneType, BuildingType};
use crate::GameState;

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_population,
                update_economy,
                update_resources,
                update_happiness,
            ).run_if(in_state(GameState::TownView)),
        );
    }
}

// Simulation parameters
const BASE_POPULATION_GROWTH: f32 = 0.01;
const BASE_HAPPINESS_DECAY: f32 = 0.001;
const BASE_RESOURCE_CONSUMPTION: i32 = 1;

// Population simulation
#[derive(Resource)]
pub struct Population {
    pub total: i32,
    pub employed: i32,
    pub growth_rate: f32,
}

impl Default for Population {
    fn default() -> Self {
        Population {
            total: 0,
            employed: 0,
            growth_rate: BASE_POPULATION_GROWTH,
        }
    }
}

// Economy simulation
#[derive(Resource)]
pub struct Economy {
    pub funds: i32,
    pub income: i32,
    pub expenses: i32,
    pub tax_rate: f32,
}

impl Default for Economy {
    fn default() -> Self {
        Economy {
            funds: 10000,
            income: 0,
            expenses: 0,
            tax_rate: 0.1,
        }
    }
}

// Resources simulation
#[derive(Resource)]
pub struct Resources {
    pub power: ResourceInfo,
    pub water: ResourceInfo,
    pub goods: ResourceInfo,
    pub services: ResourceInfo,
}

#[derive(Default)]
pub struct ResourceInfo {
    pub production: i32,
    pub consumption: i32,
    pub storage: i32,
    pub max_storage: i32,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            power: ResourceInfo {
                max_storage: 1000,
                ..Default::default()
            },
            water: ResourceInfo {
                max_storage: 1000,
                ..Default::default()
            },
            goods: ResourceInfo {
                max_storage: 500,
                ..Default::default()
            },
            services: ResourceInfo {
                max_storage: 500,
                ..Default::default()
            },
        }
    }
}

// Demand simulation
#[derive(Resource)]
pub struct Demand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
}

impl Default for Demand {
    fn default() -> Self {
        Demand {
            residential: 0.5,
            commercial: 0.3,
            industrial: 0.2,
        }
    }
}

// Update population
fn update_population(
    time: Res<Time>,
    mut population: Option<ResMut<Population>>,
    town_cells: Query<&TownCell>,
) {
    // Initialize population if it doesn't exist
    let mut population = match population {
        Some(pop) => pop,
        None => return,
    };
    
    // Count residential, commercial, and industrial zones
    let mut residential_count = 0;
    let mut commercial_count = 0;
    let mut industrial_count = 0;
    
    for cell in town_cells.iter() {
        match cell.zone {
            ZoneType::Residential => residential_count += 1,
            ZoneType::Commercial => commercial_count += 1,
            ZoneType::Industrial => industrial_count += 1,
            _ => {}
        }
    }
    
    // Calculate population growth based on available residential zones and happiness
    let growth_factor = (residential_count as f32 * 0.1).min(10.0);
    let growth = population.growth_rate * growth_factor * time.delta_seconds();
    
    // Update population
    population.total += (growth * population.total as f32).round() as i32;
    
    // Calculate employment based on commercial and industrial zones
    let max_employment = (commercial_count + industrial_count) * 5; // Each zone can employ 5 citizens
    population.employed = population.total.min(max_employment);
}

// Update economy
fn update_economy(
    time: Res<Time>,
    mut economy: Option<ResMut<Economy>>,
    population: Option<Res<Population>>,
) {
    // Initialize economy if it doesn't exist
    let mut economy = match economy {
        Some(eco) => eco,
        None => return,
    };
    
    let population = match population {
        Some(pop) => pop,
        None => return,
    };
    
    // Calculate income based on population and employment
    let base_income = population.total as f32 * 1.0; // 1 fund per citizen
    let employment_bonus = population.employed as f32 * 2.0; // 2 additional funds per employed citizen
    
    economy.income = (base_income + employment_bonus) as i32;
    
    // Calculate expenses (maintenance, services, etc.)
    economy.expenses = (population.total as f32 * 0.5) as i32; // 0.5 funds per citizen
    
    // Update funds
    let net_income = (economy.income as f32 * economy.tax_rate) as i32 - economy.expenses;
    economy.funds += net_income;
}

// Update resources
fn update_resources(
    time: Res<Time>,
    mut resources: Option<ResMut<Resources>>,
    town_cells: Query<&TownCell>,
    population: Option<Res<Population>>,
) {
    // Initialize resources if they don't exist
    let mut resources = match resources {
        Some(res) => res,
        None => return,
    };
    
    let population = match population {
        Some(pop) => pop,
        None => return,
    };
    
    // Reset production and consumption
    resources.power.production = 0;
    resources.power.consumption = 0;
    resources.water.production = 0;
    resources.water.consumption = 0;
    resources.goods.production = 0;
    resources.goods.consumption = 0;
    resources.services.production = 0;
    resources.services.consumption = 0;
    
    // Calculate production based on buildings
    for cell in town_cells.iter() {
        match cell.building {
            BuildingType::PowerPlant => resources.power.production += 100,
            BuildingType::WaterTower => resources.water.production += 100,
            _ => {}
        }
        
        // Industrial zones produce goods
        if cell.zone == ZoneType::Industrial {
            resources.goods.production += 5;
        }
        
        // Commercial zones produce services
        if cell.zone == ZoneType::Commercial {
            resources.services.production += 5;
        }
    }
    
    // Calculate consumption based on population and buildings
    let population_consumption = (population.total as f32 * 0.1) as i32;
    resources.power.consumption = population_consumption;
    resources.water.consumption = population_consumption;
    resources.goods.consumption = (population.total as f32 * 0.05) as i32;
    resources.services.consumption = (population.total as f32 * 0.05) as i32;
    
    // Update storage
    resources.power.storage += (resources.power.production - resources.power.consumption).max(-resources.power.storage);
    resources.water.storage += (resources.water.production - resources.water.consumption).max(-resources.water.storage);
    resources.goods.storage += (resources.goods.production - resources.goods.consumption).max(-resources.goods.storage);
    resources.services.storage += (resources.services.production - resources.services.consumption).max(-resources.services.storage);
    
    // Cap storage at max
    resources.power.storage = resources.power.storage.min(resources.power.max_storage);
    resources.water.storage = resources.water.storage.min(resources.water.max_storage);
    resources.goods.storage = resources.goods.storage.min(resources.goods.max_storage);
    resources.services.storage = resources.services.storage.min(resources.services.max_storage);
}

// Update happiness
fn update_happiness(
    time: Res<Time>,
    mut town: Option<ResMut<Town>>,
    resources: Option<Res<Resources>>,
    population: Option<Res<Population>>,
    economy: Option<Res<Economy>>,
) {
    // Initialize town if it doesn't exist
    let mut town = match town {
        Some(town) => town,
        None => return,
    };
    
    let resources = match resources {
        Some(res) => res,
        None => return,
    };
    
    let population = match population {
        Some(pop) => pop,
        None => return,
    };
    
    let economy = match economy {
        Some(eco) => eco,
        None => return,
    };
    
    // Calculate happiness factors
    let resource_factor = if resources.power.storage > 0 && resources.water.storage > 0 {
        1.0
    } else {
        0.5
    };
    
    let employment_factor = if population.total > 0 {
        population.employed as f32 / population.total as f32
    } else {
        1.0
    };
    
    let tax_factor = 1.0 - economy.tax_rate;
    
    // Calculate overall happiness
    let target_happiness = resource_factor * employment_factor * tax_factor;
    
    // Gradually adjust happiness towards target
    let adjustment_rate = 0.1 * time.delta_seconds();
    town.happiness += (target_happiness - town.happiness) * adjustment_rate;
    
    // Ensure happiness stays in range [0, 1]
    town.happiness = town.happiness.clamp(0.0, 1.0);
}
