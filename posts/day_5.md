# 拿起勇者之劍攻擊

手上的專案目前已經有可移動的玩家、世界背景和相機跟隨。這篇要在現有架構上擴充，建立一套完整的揮劍攻擊：角色轉身時武器會換邊、按空白鍵有節奏感的動畫、而且維持模組化的程式結構，之後好繼續堆疊命中判定或敵人 AI。

## 想達成的體驗

在開工前先確定想要的效果：
- 玩家調整朝向時，劍自動站到對應的方位並切換左右貼圖。
- 揮劍動畫不能被亂按打斷，節奏要流暢。
- 新邏輯沿用現有的 ECS 模組化做法，方便獨立維護。

整體流程可以拆成三塊：準備資產與元件 → 建立父子實體 → 三段攻擊系統。底下每個步驟都對應到專案中的實際檔案。

## 1. 準備武器資產與座標基準

把左右手版本的劍放進 `assets/weapons/`：
- `assets/weapons/sword.png`
- `assets/weapons/sword_left.png`

採用兩張貼圖是為了避免 runtime 翻轉造成像素模糊，也讓美術能微調左右細節。既有專案中玩家位置是世界座標 `(0, 0, 0)`，Z=0 表示角色層。稍後劍會變成玩家的子實體，因此 `(x, y)` 會是相對於玩家中心的偏移量，Z=1 讓它渲染在角色之上。

## 2. 擴充攻擊相關元件

開啟 `src/components/player.rs`，加入負責攻擊資料的元件：

```rust
#[derive(Component)]
pub struct PlayerFacing {
    pub direction: Vec2,
}

impl PlayerFacing {
    pub fn new() -> Self {
        Self {
            direction: Vec2::new(1.0, 0.0),
        }
    }
}

#[derive(Component)]
pub struct Weapon;

#[derive(Component)]
pub struct WeaponSprites {
    pub right_sprite: Handle<Image>,
    pub left_sprite: Handle<Image>,
}

#[derive(Component)]
pub struct WeaponOffset {
    pub base_angle: f32,
    pub position: Vec2,
}

#[derive(Component)]
pub struct WeaponSwing {
    pub timer: Timer,
    pub from_angle: f32,
    pub to_angle: f32,
}
```

- `PlayerFacing`：記錄目前朝向，後續系統靠它判斷劍要放在哪裡。
- `Weapon`：標記實體身份，查詢用。
- `WeaponSprites`：同時保存左、右兩張圖，切換時不必重新載入資產。
- `WeaponOffset`：控制劍在玩家附近的定位與基準角度。
- `WeaponSwing`：負責動畫的計時器與起迄角度。

把狀態拆開能維持系統的單一職責，未來要調整不同武器的速度或貼圖也只改對應元件即可。

## 3. 生成玩家與武器的父子實體

在 `src/systems/setup.rs` 裡，將武器建立為玩家的子實體：

```rust
let player_entity = commands.spawn((
    Player,
    Sprite::from_image(asset_server.load("characters/knight_lv1.png")),
    Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(PLAYER_SCALE)),
    Health::new(PLAYER_INITIAL_HEALTH),
    Velocity::zero(),
    PlayerFacing::new(),
)).id();

let weapon_entity = commands.spawn((
    Weapon,
    Sprite::from_image(asset_server.load("weapons/sword.png")),
    Transform::from_translation(Vec3::new(8.0, 2.0, 1.0)).with_scale(Vec3::splat(WEAPON_SCALE)),
    WeaponSprites {
        right_sprite: asset_server.load("weapons/sword.png"),
        left_sprite: asset_server.load("weapons/sword_left.png"),
    },
    WeaponOffset {
        base_angle: 0.0,
        position: Vec2::new(8.0, 2.0),
    },
    WeaponSwing {
        timer: Timer::from_seconds(0.5, TimerMode::Once),
        from_angle: 0.0,
        to_angle: 0.0,
    },
)).id();

commands.entity(player_entity).add_children(&[weapon_entity]);
```

兩個重點：
- `Vec3::new(8.0, 2.0, 1.0)` 代表劍相對於玩家中心向右偏 8、向上偏 2。Z=1 確保會疊在角色前方。
- `Timer::from_seconds(0.5, TimerMode::Once)` 指揮揮擊動畫跑 0.5 秒才算完成，`TimerMode::Once` 讓系統能判斷是否允許下一次輸入。

## 4. 更新移動系統，讓面向同步

在既有的 `movement_system` 中加入面向更新：

```rust
if velocity.x != 0.0 || velocity.y != 0.0 {
    facing.direction = Vec2::new(velocity.x, velocity.y).normalize();
}
```

輸入依舊透過速度控制位移，但現在會把方向向量標準化後寫入 `PlayerFacing`。其他系統只需要讀這個元件就能得知最新面向，避免重複解析輸入。

## 5. 三個攻擊系統分工

新增 `src/systems/attack.rs`，把攻擊流程拆解為三個步驟：

### 5.1 觸發揮劍：`attack_input_system`
- 監聽 `KeyCode::Space`。
- 若 `WeaponSwing.timer` 已經結束，就重置計時器並設定這次揮擊的角度範圍（-45° → +45°）。
- 沒走完前不會重新觸發，避免動畫跳針。

### 5.2 調整武器位置與貼圖：`update_weapon_offset_system`
- 讀取玩家面向，把方向角度切成八個象限（上下左右 + 四個斜向）。
- 各象限對應一組手持位置與 `is_left_side` 判斷，再決定使用左右哪張貼圖。
- 更新 `WeaponOffset` 的位置與 `base_angle`，並將 `Transform` 的平移與旋轉寫入。

這一步可以微調 `calculate_weapon_position_and_rotation` 裡的常數，達成符合視覺預期的握劍姿勢。

### 5.3 播放揮擊動畫：`update_weapon_swing_animation_system`
- 透過 `Timer::tick` 取得動畫進度。
- 使用 `lerp_angle` 對 `from_angle` 與 `to_angle` 做線性插值，再加上 `base_angle` 得到最終旋轉。
- 計時結束時把角度重設為 0，確保劍回到待命姿勢。

若想要非線性的動作，例如快速揮出、慢慢收回，可以把 `lerp` 換成 easing 函式或曲線表。

## 6. 使用 AttackPlugin 掛進 App

為了維持模組化，新增 `src/plugins/attack.rs`：

```rust
pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            attack_input_system,
            update_weapon_offset_system,
            update_weapon_swing_animation_system,
        ));
    }
}
```

最後在 `main.rs` 裡把插件加入：

```rust
.add_plugins((WorldPlugin, PlayerPlugin, CameraPlugin, AttackPlugin))
```

這樣攻擊系統就串接完成，後續也能輕鬆開關或進行單元測試。

## 測試與調整清單

- 在遊戲中朝不同方向走動，確認劍會依面向換邊且位置合理。
- 快速連按空白鍵，確保計時器沒完成前不會重新播放。
- 微調 `WeaponOffset` 中各象限的座標，找出最自然的握劍姿勢。
- 修改 `Timer::from_seconds` 的秒數，感受揮擊節奏差異。

## 未來延伸想法

1. 在 `WeaponSwing` 裡加上傷害判定的時間窗配合碰撞判斷。
2. 攻擊時發送事件，觸發音效或粒子特效系統。
3. 為敵人建立類似的 `Facing` 與 `Weapon` 元件，統一管理戰鬥行為。

完成以上步驟後，你的專案就具備了穩固的近戰攻擊核心。面向追蹤、貼圖切換、動畫控制皆已模組化，未來要增加新武器或連段都能從這套基礎繼續擴充。
