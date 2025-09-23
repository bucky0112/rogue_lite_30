# 相機跟隨系統（Camera Follow）

在 Rogue-lite 遊戲中，流暢的相機系統（Camera）是玩家沉浸感的關鍵。

想像一下如果遊戲中的視角突然跳躍或無法跟上角色，玩家很快就會感到困惑。就像上一篇做出來的操作系統一切正常，不過角色在操作上會跑出畫面。

![角色會跑出畫面外](https://github.com/bucky0112/blog-images/blob/main/images/CleanShot%202025-09-18%20at%2017.53.30.gif?raw=true)

所以本篇要實作的是相機跟隨系統，目標是讓角色始終保持在畫面中央，同時提供流暢、自然的視覺體驗。
不過在開始之前，要先把專案進行重構。因為目前的程式碼都寫在 `main.rs` 中，重構可以為後續的做更好的擴充。

## 重構專案

預計會將現有專案拆分為：
- `constants.rs` - 遊戲常數設定
- `components/` - ECS 元件定義（player.rs, movement.rs）  
- `systems/` - 遊戲邏輯系統（movement.rs, health.rs）
- `plugins/` - 插件架構（PlayerPlugin）

因為重構並不是這個章節的重點，所以我只提一下把原本交錯的初始化流程抽進 `plugins/PlayerPlugin`，把玩家相關的元件跟系統聚在同一個插件裡。這樣在 App 建立時只要 `.add_plugin(PlayerPlugin)`，就能一次掛上玩家的資料與行為。

所以現在的專案架構會是這樣：

```
.
├── assets
│   └── characters
│       └── knight_lv1.png
├── Cargo.lock
├── Cargo.toml
└── src
    ├── components
    │   ├── mod.rs
    │   └── player.rs
    ├── constants.rs
    ├── main.rs
    ├── plugins
    │   ├── mod.rs
    │   └── player.rs
    └── systems
        ├── health.rs
        ├── mod.rs
        ├── movement.rs
        └── setup.rs
```

在 `main.rs` 則是只要寫短短的幾行，把重構後的其他模組拉進來就跟原本的功能一模一樣了。

```rs
use bevy::prelude::*;

mod constants;
mod components;
mod systems;
mod plugins;

use plugins::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PlayerPlugin)
        .run();
}
```

## 如何設計相機跟隨系統

相機跟隨看似簡單，但其中有著許多細節：跟隨的速度、流暢度、預判性，以及如何在不同遊戲情境下調整行為。在經典的 2D 動作遊戲如《超級瑪利歐》中，畫面甚至會根據玩家移動方向提前移動，讓玩家能看到前方更多內容。雖然我們今天作的是基礎版本，但這個系統將為後續的地圖探索有著重要的幫助。

我選擇使用 **線性插值（Linear Interpolation, Lerp）** 來實現流暢跟隨。這種方法的核心概念是：每一幀都讓相機朝目標位置移動一小段距離，而不是立即跳到目標位置。

公式為：

```
new_position = current_position + (target_position - current_position) * lerp_factor
```

`lerp_factor` 是介於 0 到 1 之間的值，決定了每幀移動的比例。值越大相機跟得越緊，值越小則越流暢但可能有延遲感。

## 程式碼解析

基於我們剛剛建立的模組化架構，相機跟隨功能也會被拆分到不同模組中。

### 元件設計（components/camera.rs）

相機跟隨的核心是 `CameraFollow` 元件：

```rust
#[derive(Component)]
pub struct CameraFollow {
    pub speed: f32,
}

impl CameraFollow {
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}
```

設計成元件而非全域設定，是因為未來可能需要多個相機，每個都有不同的跟隨行為，所以用模組化來實作是比較好的做法。

### 相機跟隨系統（systems/camera.rs）

在 `camera_follow_system` 中，我使用了 Bevy 的查詢過濾器來避免借用衝突：

```rust
pub fn camera_follow_system(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), (With<CameraFollow>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut camera_transform, camera_follow) in &mut camera_query {
            let target_position = player_transform.translation;
            let current_position = camera_transform.translation;
            
            let lerp_factor = (camera_follow.speed * time.delta_secs()).min(1.0);
            
            camera_transform.translation.x = current_position.x + 
                (target_position.x - current_position.x) * lerp_factor;
            camera_transform.translation.y = current_position.y + 
                (target_position.y - current_position.y) * lerp_factor;
        }
    }
}
```

`Without<CameraFollow>` 和 `Without<Player>` 確保兩個查詢不會同時訪問相同的 Entity，這是 Bevy 借用檢查器的要求。

線性插值的實作中，我特別注意了幾個細節：

1. **時間獨立性**：使用 `time.delta_secs()` 確保不同幀率下行為一致
2. **邊界保護**：`lerp_factor.min(1.0)` 防止插值超調
3. **Z軸保護**：只插值 X 和 Y 座標，保持相機深度不變

### 相機設置（systems/setup.rs）

在 systems 中，幫相機加入了 `CameraFollow` 元件：

```rust
pub fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        CameraFollow::new(CAMERA_FOLLOW_SPEED),
    ));
}
```

### 常數配置（constants.rs）

將相機跟隨速度提取為常數，便於調整和維護：

```rust
pub const CAMERA_FOLLOW_SPEED: f32 = 3.0;
```

## 視覺化地圖系統

為了讓相機跟隨效果更明顯，加入了格子地板系統。這個 20x20 的棋盤式地圖不僅提供了空間參考，還展示了 Bevy 中批量生成 Entity 的模式。

### World component（components/world.rs）

主要是為了地圖元素專用元件：

```rust
#[derive(Component)]
pub struct GridTile;

#[derive(Component)] 
pub struct CenterMarker;

#[derive(Component)]
pub struct CornerMarker;
```

### World System（systems/world.rs）

地圖產生邏輯展現了系統化的 Entity 批次建立：

```rust
pub fn setup_world(mut commands: Commands) {
    let half_grid = GRID_SIZE as f32 / 2.0;
    
    // 產生棋盤格地板
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
            color: Color::srgb(1.0, 0.0, 0.0),
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
                color: Color::srgb(0.0, 1.0, 0.0),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..Default::default()
            },
            Transform::from_translation(Vec3::new(x, y, -0.5)),
        ));
    }
}
```

我還加入了視覺標記：紅色中心點標示原點 (0,0)，四個綠色角落提供邊界參考。這些元素的 Z 軸層次安排（格子在 -1.0，標記在 -0.5，角色在 0.0）確保了正確的渲染順序。

### 插件架構（plugins/）

遵循模組化設計，為相機和世界系統分別建立插件：

```rust
// plugins/camera.rs
pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow_system);
    }
}

// plugins/world.rs  
pub struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_world);
    }
}
```

最終在 main.rs 中組織所有插件：

```rust
use bevy::prelude::*;

mod constants;
mod components;
mod systems;
mod plugins;

use plugins::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            WorldPlugin,
            PlayerPlugin,
            CameraPlugin,
        ))
        .run();
}
```

## 結果展示

先來看如果不加入格子地圖的相機系統看起來會是這樣：

![未加入地圖系統](https://github.com/bucky0112/blog-images/blob/main/images/CleanShot%202025-09-18%20at%2022.24.07.gif?raw=true)

而加入地圖之後看起來則是這樣：

![加入地圖系統後](https://github.com/bucky0112/blog-images/blob/main/images/CleanShot%202025-09-18%20at%2022.23.00.gif?raw=true)

在未加入格子地圖之前，角色移動下相機確實會跟隨，看到角色一直在中間就是證明。但是一般使用者是看不出來現在是什麼情況，只覺得怎麼移動了，但是又回到原點。

而加入地圖在對比下，使用者可以清楚地看到相機會跟著角色移動畫面，而且角色確實是有在移動的。

所以完成後的相機系統展現了以下特性：

- **流程跟隨**：當用 WASD 或方向鍵移動角色時，相機以自然的速度跟隨。

- **視覺參考**：格子讓玩家更清楚對照出角色的實際移動，紅色中心點讓你知道相對於原點的位置，綠色角落標記地圖的邊界。

- **響應靈敏**：相機跟隨速度設定為 3.0，在流暢度和響應性之間取得平衡，停止移動時相機也會平穩地停下。

- **效能穩定**：400個格子 Entity（20x20）加上 5 個標記點，在現代硬體上完全沒有效能問題，幀率保持在 60fps。

測試時特別注意斜向移動的效果，相機能正確跟隨 8 方向的移動，展現出一致的流暢度。

## 小結

- **模組化架構**：將功能拆分到不同模組後，程式碼的可讀性和維護性大幅提升。每個檔案職責單一，修改時不會影響其他功能。

- **ECS 查詢過濾**：`Without<T>` 過濾器是解決借用衝突的優雅方案，比手動管理查詢更安全。

- **時間基礎動畫**：使用 `delta_time` 而非固定步長，讓動畫在不同硬體上保持一致性。

- **插件系統靈活性**：Bevy 的插件系統讓功能組織變得非常清晰，`WorldPlugin`、`PlayerPlugin`、`CameraPlugin` 各司其職，可以獨立開發和測試。

- **視覺化調試**：格子地圖系統不僅解決了「看不出移動效果」的問題，更成為了後續地圖產生的原型。

這個架構展現了 Rust 模組系統的強大，`main.rs` 僅剩 19 行，所有功能都被合理分配。未來如果要加入新功能時，只需要新增對應的 component、systems 和 plugins，不會讓程式碼變得混亂。

> 今天的程式碼分享在 [repo](https://github.com/bucky0112/rogue_lite_30)
