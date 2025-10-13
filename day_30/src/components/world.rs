use bevy::prelude::*;

#[derive(Component)]
pub struct GridTile;

#[derive(Component)]
pub struct CenterMarker;

#[derive(Component)]
pub struct CornerMarker;

#[derive(Component)]
pub struct EnvironmentProp {
    pub blocks_movement: bool,
}

#[derive(Component)]
pub struct CorridorTile;

impl EnvironmentProp {
    pub fn blocking() -> Self {
        Self {
            blocks_movement: true,
        }
    }

    pub fn decorative() -> Self {
        Self {
            blocks_movement: false,
        }
    }
}

#[derive(Component, Debug)]
pub struct RoomTile {
    pub tile_type: RoomTileType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomTileType {
    Floor,
    WallNInnerCornerW,
    WallNInnerMid,
    WallNInnerCornerE,
    WallSInnerCapL,
    WallSInnerMid,
    WallSInnerCapR,
    WallSOuterCapL,
    WallSOuterMid,
    WallSOuterCapR,
    WallESide,
    WallWSide,
    DoorClosed,
    DoorOpen,
    FloorOutdoor,
}

#[derive(Debug, Clone)]
pub struct RoomRect {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
}

#[derive(Component, Debug)]
pub struct CompoundRoom {
    pub rectangles: Vec<RoomRect>,
    pub room_type: CompoundRoomType,
}

#[derive(Debug, Clone)]
pub enum CompoundRoomType {
    LShape,
    TShape,
    Cross,
}

#[derive(Component, Debug)]
pub struct Door {
    pub is_open: bool,
}
