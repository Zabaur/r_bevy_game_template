use bevy::prelude::*;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        // No systems needed for now, this is just a utility module
    }
}

// Grid cell marker trait
pub trait GridCell {
    fn position(&self) -> IVec2;
}

// Grid utility functions
pub struct Grid;

impl Grid {
    // Check if two positions are adjacent
    pub fn are_adjacent(pos1: IVec2, pos2: IVec2) -> bool {
        let diff = (pos1 - pos2).abs();
        (diff.x + diff.y == 1) || (diff.x == 1 && diff.y == 1)
    }
    
    // Get all adjacent positions
    pub fn get_adjacent_positions(pos: IVec2) -> [IVec2; 8] {
        [
            IVec2::new(pos.x - 1, pos.y - 1),
            IVec2::new(pos.x, pos.y - 1),
            IVec2::new(pos.x + 1, pos.y - 1),
            IVec2::new(pos.x - 1, pos.y),
            IVec2::new(pos.x + 1, pos.y),
            IVec2::new(pos.x - 1, pos.y + 1),
            IVec2::new(pos.x, pos.y + 1),
            IVec2::new(pos.x + 1, pos.y + 1),
        ]
    }
    
    // Get orthogonally adjacent positions (no diagonals)
    pub fn get_orthogonal_positions(pos: IVec2) -> [IVec2; 4] {
        [
            IVec2::new(pos.x, pos.y - 1),
            IVec2::new(pos.x - 1, pos.y),
            IVec2::new(pos.x + 1, pos.y),
            IVec2::new(pos.x, pos.y + 1),
        ]
    }
    
    // Check if a position is within bounds
    pub fn is_in_bounds(pos: IVec2, size: usize) -> bool {
        pos.x >= 0 && pos.x < size as i32 && pos.y >= 0 && pos.y < size as i32
    }
    
    // Calculate Manhattan distance between two positions
    pub fn manhattan_distance(pos1: IVec2, pos2: IVec2) -> i32 {
        (pos1.x - pos2.x).abs() + (pos1.y - pos2.y).abs()
    }
    
    // Find a path between two positions using A* algorithm
    pub fn find_path<T: GridCell>(
        start: IVec2,
        goal: IVec2,
        is_accessible: impl Fn(IVec2) -> bool,
        size: usize,
    ) -> Option<Vec<IVec2>> {
        use std::collections::{BinaryHeap, HashMap};
        use std::cmp::Ordering;
        
        // A* node
        #[derive(Copy, Clone, Eq, PartialEq)]
        struct Node {
            position: IVec2,
            f_score: i32,
        }
        
        impl Ord for Node {
            fn cmp(&self, other: &Self) -> Ordering {
                // Reverse ordering for min-heap
                other.f_score.cmp(&self.f_score)
            }
        }
        
        impl PartialOrd for Node {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        
        g_score.insert(start, 0);
        open_set.push(Node {
            position: start,
            f_score: Grid::manhattan_distance(start, goal),
        });
        
        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // Reconstruct path
                let mut path = vec![goal];
                let mut current = goal;
                while let Some(&prev) = came_from.get(&current) {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                return Some(path);
            }
            
            let current_g = *g_score.get(&current.position).unwrap_or(&i32::MAX);
            
            for neighbor in Grid::get_orthogonal_positions(current.position) {
                if !Grid::is_in_bounds(neighbor, size) || !is_accessible(neighbor) {
                    continue;
                }
                
                let tentative_g = current_g + 1;
                if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                    came_from.insert(neighbor, current.position);
                    g_score.insert(neighbor, tentative_g);
                    let f_score = tentative_g + Grid::manhattan_distance(neighbor, goal);
                    open_set.push(Node {
                        position: neighbor,
                        f_score,
                    });
                }
            }
        }
        
        None // No path found
    }
}
