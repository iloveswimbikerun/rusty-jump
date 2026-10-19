# Rusty Jump

用 Rust + [Bevy](https://bevy.org) 0.20 寫的橫向捲軸闖關小遊戲，編譯成 WebAssembly，電腦和手機瀏覽器都能玩。

**線上遊玩：** https://iloveswimbikerun.github.io/rusty-jump/

## 操作

| | 電腦 | 手機（請橫向） |
|---|---|---|
| 移動 | ← → 或 A D | 左下 `<` `>` |
| 跳躍（按久跳高） | Space / ↑ / W / Z | 右下 `A`（右半邊螢幕都可以） |
| 開始 / 重來 | Enter / Space | 點一下螢幕 |

踩敵人、頂問號磚拿金幣、頂破磚塊，碰到旗桿過關，共三關（1-1 白天、1-2 黃昏、1-3 夜晚）。

## 本機建置

```sh
rustup target add wasm32-unknown-unknown   # Arch: pacman -S rust-wasm
cargo install trunk                        # Arch: pacman -S trunk binaryen
trunk serve --release                      # http://127.0.0.1:8080
```

部署：`trunk build --release --public-url ./` 後把 `dist/` 內容推到 `gh-pages` 分支。

除錯用：建置時設 `RJ_LEVEL=1`（第幾關，從 0 起算）或 `RJ_START_X=140`（起點欄位）可直接跳到指定位置。

所有圖形都是程式畫的色塊，沒有使用任何任天堂素材。
