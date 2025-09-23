# 利用遊戲素材資源創造出勇者


這篇文章要為我們的 Rogue-lite 遊戲加入真正的角色形象，跟之前創造的藍色方塊說再見。

在 Rogue-lite 遊戲中，玩家角色是整個遊戲體驗的核心，需要有清晰辨識的視覺形象。雖然藍色方塊能夠展示 ECS 系統和移動機制，但真正的角色能大幅提升遊戲的沉浸感，就像你不會想玩一個不知道是什麼的東西。

網路上有很多遊戲開發的資源，有收費也有免費的選擇，我選擇使用的是 [Kenney](https://kenney.nl/) 的圖庫。

![Kenney 官網圖](https://ithelp.ithome.com.tw/upload/images/20250917/201202930D4HrgRu0t.png)

Kenney 是一個很受歡迎的遊戲素材網站，對遊戲開發者（特別是 Indie 或練習階段的人）非常友好。提供大量免費遊戲素材，大部分素材可以免費使用，用於商業作品也沒問題。授權通常很開放（例如 Creative Commons、CC0 或相近類型），但還是要注意每個素材的授權條款並確認細節。

以我們想做的目標是像素遊戲來說，可以前往 Assets 的頁面，在這裡選擇 Pixel 的素材。

![選擇 Pixel 步驟](https://ithelp.ithome.com.tw/upload/images/20250917/20120293k2NaYxKN76.png)

選擇有很多，大家可以選擇自己喜歡的素材，我選擇的是這個 [Tiny Dungeon](https://kenney.nl/assets/tiny-dungeon)。他的授權是寫 Creative Commons CC0，代表可以自由使用，但可能還是有些需要注意的地方，例如不要直接把原素材整包拿來販售，這是尊重原作者，也是讓創作者們有一個好的循環。

如果你覺得這些資源對你很有幫助的話，也可以在下載時抖內對方。

![下載時可以考慮抖內對方的創作](https://ithelp.ithome.com.tw/upload/images/20250917/20120293fD1ovzCQ8r.png)

![Tiny Dungeon 素材包](https://ithelp.ithome.com.tw/upload/images/20250917/20120293qu1cgzerJ0.png)

我選擇的是 Tiny Dungeon 這個素材包，主要是裡面有一個看起來是騎士的角色。因為這個階段重點是建立正確的資源載入和顯示流程，為後續的方向系統和動畫打下基礎。而這個角色具有經典 RPG 勇者的視覺魅力，清晰的像素風格也符合我們的遊戲定位。

## 設計思路

目前實作的是一個簡單但清晰的角色系統，重點有三個：

1. **資源載入**：使用 Bevy 的 `AssetServer` 載入角色圖片。
2. **像素藝術處理**：設定 ImagePlugin::default_nearest() 保持像素銳利。
3. **尺寸縮放**：將 16x16 像素的角色放大至合適大小。

在 Bevy 的 ECS 架構下，我保持了原有的簡潔結構：Player Component 標示角色實體，`Health` 和 `Velocity` 管理狀態和移動。

那麼角色圖片要放在哪裡呢？

只要在專案中建立一個 `assets` 的資料夾，裡面就可以放入我們的素材圖片了。由於下載整包的資源非常多，其中有包含角色、地形跟建築的圖片，我們先挑出角色圖片，然後在專案中的 `assets` 底下再新增一個資料夾命名為 `characters`，然後把角色圖片放進去。

![assets 架構](https://ithelp.ithome.com.tw/upload/images/20250917/20120293EShsoolclf.png)

## 程式碼解析

這次主要是將原本的藍色方塊替換為素材圖片，主要改動集中在 `spawn_player` 函式：

```rust
fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Player,
        Sprite::from_image(asset_server.load("characters/knight_lv1.png")),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
            .with_scale(Vec3::splat(4.0)),
        Health { current: 100, max: 100 },
        Velocity { x: 0.0, y: 0.0 },
    ));
    info!("騎士玩家已誕生！");
}
```

關鍵改動有三點：

1. **角色圖載入**：使用 `Sprite::from_image(asset_server.load(...))` 取代之前的藍色方塊。
2. **縮放設定**：`Transform::with_scale(Vec3::splat(4.0))` 將 16x16 像素的角色放大 4 倍，變成 64x64 像素。
3. **資源參數**：函式簽名加入 `asset_server: Res<AssetServer>` 來載入圖片。

移動系統保持不變，依然使用 WASD 或方向鍵控制，速度設定為 300 單位/秒。`Velocity` 元件一樣使用簡單的 x, y 分量記錄，目前還沒有導入複雜的向量計算。

## 像素風格設定

為了確保騎士角色的像素風格正確顯示，我在 `main()` 函式中加入了重要的圖片渲染設定：

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup, spawn_player))
        .add_systems(Update, (movement_system, health_system))
        .run();
}
```

`ImagePlugin::default_nearest()` 設定使用最近鄰插值法，這對像素的角色非常重要：

- 避免在縮放時產生模糊效果
- 保持每個像素格子的銳利邊緣
- 重現經典 16-bit 遊戲的視覺風格

比較一下加上 `ImagePlugin::default_nearest()` 的差異，我把比例放大到 6 倍：

![沒處理圖片縮放](https://github.com/bucky0112/blog-images/blob/main/images/CleanShot%202025-09-17%20at%2022.57.34.gif?raw=true)

![有處理圖片縮放](https://github.com/bucky0112/blog-images/blob/main/images/CleanShot%202025-09-17%20at%2022.58.31.gif?raw=true)

經由上面的圖片比對，應該可以分辨出來，如果角色沒特別經由縮放處理的話，角色看起來會是模糊的；但是有處理過的角色，看起來會保持像素特有的格子風格。透過適當的縮放處理，能在保持像素風格清晰度的同時，達到適合遊戲的視覺大小。

## 小結

這篇實作最大的收穫是：

- **資源載入**：學會如何在 Bevy 中正確載入和使用圖片資源，以及 `AssetServer` 的使用方式。
- **像素風格處理**：掌握了 `ImagePlugin::default_nearest()` 的重要性，確保像素的清晰顯示。
- **縮放技巧**：了解如何透過 `Transform::with_scale()` 將小尺寸素材放大到合適大小。
- **漸進式開發**：從簡單的方塊到真實角色，體會到遊戲開發的迭代過程。

主要的踩雷經驗：

- 16x16 像素的角色非常小，需要至少 4 倍縮放才能在螢幕上看清楚。
- 沒有設定 nearest 插值的話，像素會變得模糊失真。
- 資源路徑必須相對於 `assets` 資料夾，載入時不需要包含 "assets/" 前綴。

> 今天的程式碼分享在 [repo](https://github.com/bucky0112/rogue_lite_30)
