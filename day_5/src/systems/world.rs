use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub fn setup_world(mut commands: Commands) {
    let half_grid = GRID_SIZE as f32 / 2.0;
    
    // 棋盤格地板
    for x in 0..GRID_SIZE {
        for y in 0..GRID_SIZE {
            let world_x = (x as f32 - half_grid + 0.5) * TILE_SIZE;
            let world_y = (y as f32 - half_grid + 0.5) * TILE_SIZE;
            
            let color = if (x + y) % 2 == 0 {
                Color::srgb(0.3, 0.3, 0.3) // 深灰色
            } else {
                Color::srgb(0.4, 0.4, 0.4) // 淺灰色
            };
            
            commands.spawn((
                GridTile,
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..Default::default()
                },
                Transform::from_translation(Vec3::new(world_x, world_y, -1.0)),
            ));
        }
    }
    
    // 紅色中心點標記 (0,0)
    commands.spawn((
        CenterMarker,
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0), // 紅色
            custom_size: Some(Vec2::new(16.0, 16.0)),
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -0.5)),
    ));
    
    // 四個綠色角落標記
    let corner_positions = [
        (-half_grid * TILE_SIZE + TILE_SIZE * 0.5, -half_grid * TILE_SIZE + TILE_SIZE * 0.5),
        (half_grid * TILE_SIZE - TILE_SIZE * 0.5, -half_grid * TILE_SIZE + TILE_SIZE * 0.5),
        (-half_grid * TILE_SIZE + TILE_SIZE * 0.5, half_grid * TILE_SIZE - TILE_SIZE * 0.5),
        (half_grid * TILE_SIZE - TILE_SIZE * 0.5, half_grid * TILE_SIZE - TILE_SIZE * 0.5),
    ];
    
    for (x, y) in corner_positions {
        commands.spawn((
            CornerMarker,
            Sprite {
                color: Color::srgb(0.0, 1.0, 0.0), // 綠色
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..Default::default()
            },
            Transform::from_translation(Vec3::new(x, y, -0.5)),
        ));
    }
}

