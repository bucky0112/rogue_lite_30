use bevy::prelude::*;

#[derive(Component)]
pub struct GridTile;

#[derive(Component)]
pub struct CenterMarker;

#[derive(Component)]
pub struct CornerMarker;

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
