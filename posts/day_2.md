# 打好 ECS 基礎（Entity  Component  System）

## 功能介紹

在開始開發遊戲之前，我們需要先打好基礎。在 Bevy 遊戲引擎裡，最核心的概念就是 ECS（Entity Component System）。這是一種現代遊戲引擎常用的設計模式，能讓遊戲邏輯保持彈性和可維護性。

它將遊戲物件拆解成三個基本概念：
- 實體（Entity）
- 元件（Component）
- 系統（System）

與傳統的物件導向程式設計不同，ECS 採用組合優於繼承的概念，讓我們能夠更靈活地組織遊戲邏輯。透過建立基本的玩家實體、移動元件和輸入系統。

在今天的進度，我們將試著做出一個可控制的藍色方塊，這將成為我們 Rogue-lite 遊戲的基礎。

## 技術解析

ECS 架構的精髓在於資料與邏輯的分離。在 Bevy 中：

- Entity：只是一個 ID，沒有邏輯，本身就像一個容器。
- Component：純資料結構，存放與實體相關的狀態，例如位置、速度、血量。
- System：一個函式，會批次處理符合條件的實體，負責遊戲邏輯。

這樣的分工能讓「資料」和「行為」完全分離，比起物件導向程式設計（OOP）的繼承方式，更容易組合、替換和擴展。

我們先定義三個核心 Component：

```rust
use bevy::prelude::*;

#[derive(Component)]
struct Player;  // 標記元件，用來標示玩家實體

#[derive(Component)]
struct Health {
    current: i32,  // 目前血量
    max: i32,      // 最大血量
}

#[derive(Component)]
struct Velocity {
    x: f32,  // X 軸移動速度
    y: f32,  // Y 軸移動速度
}
```

`Player` 是一個標記元件，沒有給預設狀態，只是用來標記「這個實體是玩家」。

而 `Health` 和 `Velocity` 則是資料元件，分別代表生命值與移動速度的設定。

## 設定 System

在遊戲一開始，需要建立一個畫面，或者可以想像成建立一個遊戲中的世界，我們要建立一個 2D 的世界：

```rust
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Sprite {
            color: Color::srgb(0.0, 0.5, 1.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        Health {
            current: 100,
            max: 100,
        },
        Velocity { x: 0.0, y: 0.0 },
    ));
    info!("玩家已誕生！");
}
```

來看看我們做了什麼：

簡單來說，`setup` 建立了攝影機，`spawn_player` 則是生成一個玩家角色，並附加了 `Sprite`、`Transform`、`Health` 和 `Velocity` 等元件。

`Sprite` 代表產生一個藍色方塊，`Transform` 則是控制方塊的座標，`Health` 是角色的血量，最後 `Velocity` 存放角色移動的速度。

這樣一個完整的「玩家實體」就出現了。

接著來做移動系統的實作，它展現了 ECS Query 的威力：

```rust
fn movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,  // 鍵盤輸入資源
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,  // 查詢語法
    time: Res<Time>,  // 時間資源
) {
    for (mut transform, mut velocity) in &mut query {
        velocity.x = 0.0;
        velocity.y = 0.0;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.y = 300.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.y = -300.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            velocity.x = -300.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            velocity.x = 300.0;
        }

        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}
```

這個查詢語法 `Query<(&mut Transform, &mut Velocity), With<Player>>` 的意思是：「找出所有同時擁有 `Transform`、`Velocity` 元件，並且標記為 Player 的實體」。

這個 movement_system 屬於 Update 系統，代表每一幀都會跑一次，根據鍵盤輸入更新玩家的 `Velocity`，再改變 `Transform`。

ECS 的查詢系統會自動幫我們篩選出符合條件的實體，讓邏輯只會應用在玩家身上。換句話說，movement_system 做的事就是：根據鍵盤輸入更新玩家的速度，然後再更新玩家的座標位置。

最後再加上一個血量檢查系統，雖然目前玩家不會扣血，但這個系統可以在未來的戰鬥機制中直接沿用，負責處理死亡判定或觸發動畫。

```
fn health_system(query: Query<&Health, With<Player>>) {
    for health in &query {
        if health.current <= 0 {
            info!("玩家死亡！");
        }
    }
}
```

主要是檢查玩家的 `Health`，如果 `current <= 0` 就出現「玩家死亡！」。

## 系統執行時機

系統的執行時機很重要，所以這裡再次說明一下：

- `spawn_player` 被放在 `Startup` 階段，只會在遊戲啟動時執行一次，負責產生角色實體。

- `movement_system` 和 `health_system` 則被放在 `Update` 階段，每一幀都會執行，用來處理即時的遊戲邏輯。

這種分類方式避免了不必要的效能消耗，也讓程式的職責分工更清楚。

假如把 `spawn_player` 放在 `Update`，就會導致每幀都產生一個新玩家，畫面上瞬間會出現無限個方塊，效能也會立刻崩壞。這就是為什麼要區分 Startup 與 Update 系統。

## 結果展示

經過實作之後，我們成功建立了一個基本的 ECS 系統。整個 App 的架構如下：

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)  // 加載預設功能
        .add_systems(Startup, (setup, spawn_player))  // 初始化系統
        .add_systems(Update, (movement_system, health_system))  // 每幀更新系統
        .run();
}
```

執行 `cargo run` 後，螢幕上會出現一個藍色的 50x50 方塊。你可以用 WASD 或方向鍵控制它左右移動、上下加速，移動速度設定為每秒 300 像素。雖然目前只是個方塊，但它已經是遊戲角色的雛形。

![demo](https://raw.githubusercontent.com/bucky0112/blog-images/main/images/img_20250915_214248.gif![CleanShot 2025-09-15 at 21.32.31](https://raw.githubusercontent.com/bucky0112/blog-images/main/images/img_20250915_214248.gif))

明天，我們會試著幫這個藍色方塊換上真實的美術資源，讓角色不只是「能動的方塊」，讓角色**活過來**！

> 今天的程式碼分享在 [repo](https://github.com/bucky0112/rogue_lite_30)

