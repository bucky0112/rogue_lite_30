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
    pub boss_wizards: usize,
}

impl EnemyCounts {
    pub fn total(&self) -> usize {
        self.slimes + self.cyclops + self.boss_wizards
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
                    name: "Emerald Antechamber",
                    layout: RoomLayout::Rectangle {
                        width: 12,
                        height: 9,
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 10,
                        cyclops: 0,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 3,
                        rocks: 2,
                        crates: 1,
                    },
                    seed: 11,
                },
                LevelDefinition {
                    index: 1,
                    name: "Crimson Fork",
                    layout: RoomLayout::Compound {
                        room_type: CompoundRoomType::LShape,
                        rectangles: vec![
                            RoomRect {
                                x: -6,
                                y: -6,
                                width: 8,
                                height: 12,
                            },
                            RoomRect {
                                x: 1,
                                y: -2,
                                width: 9,
                                height: 8,
                            },
                        ],
                    },
                    enemy_counts: EnemyCounts {
                        slimes: 10,
                        cyclops: 5,
                        boss_wizards: 0,
                    },
                    prop_plan: PropPlan {
                        trees: 2,
                        rocks: 3,
                        crates: 2,
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
                        slimes: 12,
                        cyclops: 8,
                        boss_wizards: 1,
                    },
                    prop_plan: PropPlan {
                        trees: 3,
                        rocks: 4,
                        crates: 3,
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
                        slimes: 14,
                        cyclops: 11,
                        boss_wizards: 0,
                    },
                    prop_plan: PropPlan {
                        trees: 4,
                        rocks: 4,
                        crates: 4,
                    },
                    seed: 91,
                },
            ],
        }
    }
}

impl LevelState {
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
