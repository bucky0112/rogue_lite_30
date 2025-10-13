use crate::components::world::{CompoundRoomType, RoomRect};
use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub index: usize,
    pub name: &'static str,
    pub layout: RoomLayout,
    pub enemy_counts: EnemyCounts,
    pub prop_plan: PropPlan,
    pub seed: u64,
}

impl LevelDefinition {
    pub fn enemy_total(&self) -> usize {
        self.enemy_counts.total()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnemyCounts {
    pub slimes: usize,
    pub cyclops: usize,
    pub spiders: usize,
    pub boss_wizards: usize,
}

impl EnemyCounts {
    pub fn total(&self) -> usize {
        self.slimes + self.cyclops + self.spiders + self.boss_wizards
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PropPlan {
    pub trees: usize,
    pub rocks: usize,
    pub crates: usize,
}

impl PropPlan {
    pub fn total(&self) -> usize {
        self.trees + self.rocks + self.crates
    }
}

#[derive(Debug, Clone)]
pub enum RoomLayout {
    Rectangle {
        width: usize,
        height: usize,
    },
    Compound {
        room_type: CompoundRoomType,
        rectangles: Vec<RoomRect>,
    },
}

#[derive(Resource, Debug, Clone)]
pub struct LevelState {
    current_index: usize,
    definitions: Vec<LevelDefinition>,
}

impl Default for LevelState {
    fn default() -> Self {
        Self {
            current_index: 0,
            definitions: vec![
                LevelDefinition {
                    index: 0,
                    name: "Verdant Approach",
                    layout: RoomLayout::Rectangle {
                        width: 14,
                        height: 10,
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 6,
                        cyclops: 0,
                        spiders: 0,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 4,
                        rocks: 3,
                        crates: 2,
                    },
                    seed: 11,
                },
                LevelDefinition {
                    index: 1,
                    name: "Crimson Concourse",
                    layout: RoomLayout::Compound {
                        room_type: CompoundRoomType::LShape,
                        rectangles: vec![
                            RoomRect {
                                x: -6,
                                y: -7,
                                width: 9,
                                height: 13,
                            },
                            RoomRect {
                                x: 0,
                                y: -3,
                                width: 10,
                                height: 9,
                            },
                        ],
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 6,
                        cyclops: 4,
                        spiders: 2,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 3,
                        rocks: 4,
                        crates: 3,
                    },
                    seed: 27,
                },
                LevelDefinition {
                    index: 2,
                    name: "Saffron Crossroads",
                    layout: RoomLayout::Compound {
                        room_type: CompoundRoomType::TShape,
                        rectangles: vec![
                            RoomRect {
                                x: -7,
                                y: 1,
                                width: 14,
                                height: 6,
                            },
                            RoomRect {
                                x: -3,
                                y: -7,
                                width: 6,
                                height: 12,
                            },
                        ],
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 8,
                        cyclops: 6,
                        spiders: 3,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 4,
                        rocks: 5,
                        crates: 4,
                    },
                    seed: 56,
                },
                LevelDefinition {
                    index: 3,
                    name: "Azure Sanctum",
                    layout: RoomLayout::Compound {
                        room_type: CompoundRoomType::Cross,
                        rectangles: vec![
                            RoomRect {
                                x: -8,
                                y: -2,
                                width: 16,
                                height: 6,
                            },
                            RoomRect {
                                x: -3,
                                y: -9,
                                width: 6,
                                height: 16,
                            },
                        ],
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 8,
                        cyclops: 8,
                        spiders: 4,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 5,
                        rocks: 5,
                        crates: 4,
                    },
                    seed: 91,
                },
            ],
        }
    }
}

impl LevelState {
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn definition_count(&self) -> usize {
        self.definitions.len()
    }

    pub fn definition(&self, index: usize) -> &LevelDefinition {
        &self.definitions[index]
    }

    pub fn next_index(&self) -> Option<usize> {
        if self.current_index + 1 < self.definitions.len() {
            Some(self.current_index + 1)
        } else {
            None
        }
    }

    pub fn set_current_index(&mut self, index: usize) {
        self.current_index = index.min(self.definitions.len().saturating_sub(1));
    }
}

#[derive(Resource, Default)]
pub struct LevelBuildContext {
    pub pending_layout: Option<usize>,
    pub pending_finalize: Option<usize>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct PendingLevelRewards {
    pub level_index: usize,
    pub portal_anchor: Option<Vec3>,
    pub tile_size: f32,
    pub target_level: Option<usize>,
    pub rewards_spawned: bool,
    pub rewards_available: bool,
}
