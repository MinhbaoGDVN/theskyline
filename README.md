# The Skyline

Game lái xe thế giới mở viết bằng **Rust + macroquad**. Lái ô tô, xe máy, máy bay và xe tăng quanh một thành phố 3D.

## Yêu cầu

- Rust (cài qua [rustup.rs](https://rustup.rs))
- Windows 10/11, GPU hỗ trợ OpenGL 3.3 trở lên (đủ cho Intel HD 4000)
- Tùy chọn, để nhúng icon vào `theskyline.exe`: **WinLibs** (cung cấp `windres`)

## Chạy game

```powershell
cargo run --release
```

Lần đầu build sẽ tải và biên dịch nhiều thư viện, mất vài phút.

## Cấu trúc thư mục

```
theskyline/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── assets/
│   ├── icons/      # icon.png (cửa sổ, Discord), icon.ico (file exe)
│   ├── models/     # model OBJ: car, bike, plane, tank + palette.mtl
│   └── fonts/      # ui.ttf (không đưa lên git)
└── src/
    ├── main.rs         # khởi tạo cửa sổ, vòng lặp chính
    ├── game.rs         # trạng thái game, cập nhật, vẽ, HUD
    ├── vehicle.rs      # vật lý và hộp số của xe
    ├── city.rs         # đường, khối nhà, tòa nhà, phá hủy
    ├── traffic.rs      # xe NPC
    ├── combat.rs       # đạn, vụ nổ, hạt bụi
    ├── camera.rs       # camera bám xe, culling
    ├── render.rs       # đổ bóng theo đỉnh, gom mesh
    ├── shadow.rs       # bóng đổ phẳng
    ├── minimap.rs      # minimap góc phải trên
    ├── hud.rs          # bảng thông tin, thông báo
    ├── input.rs        # đọc phím
    ├── models.rs, obj.rs  # nạp model OBJ
    ├── discord.rs      # Discord Rich Presence
    ├── icon.rs         # icon cửa sổ game
    ├── build.rs        # nhúng icon vào exe (Windows)
    └── .config         # hằng số cấu hình
```

## Điều khiển

| Phím | Tác dụng |
| --- | --- |
| W / S | Ga / phanh lùi (máy bay: ngóc / cúi đầu) |
| A / D | Lái trái / phải (máy bay: nghiêng cánh để rẽ) |
| Space | Phanh tay |
| Shift / Ctrl | Tăng / giảm ga (máy bay) |
| X / Z | Lên số / xuống số |
| T | Bật / tắt số tự động |
| F | Bắn đạn (xe tăng) |
| 1, 2, 3, 4 | Chọn ô tô, xe máy, máy bay, xe tăng |
| Q / E | Đổi phương tiện trước / sau |
| R | Hồi phục phương tiện (đưa về chỗ trống gần nhất) |
| Shift + D + F12 | Bật / tắt dev mode |

## Luật chơi

- **Ô tô, xe máy, xe tăng** có hộp số 4 cấp. Giữ W để tốc độ trên UI tăng dần, đạt giới hạn của số hiện tại thì phải lên số.
- **Máy bay** không có số, điều khiển bằng ga. Lao xuống quá mạnh hoặc đâm nhà sẽ rơi, bấm R để hồi phục.
- **Xe tăng** bắn đạn phá nhà. Mỗi tòa có 150 máu, trúng trực tiếp 2 phát mới sập.
- **Va chạm NPC:** xe mặt đất đâm vào xe NPC sẽ bật lại, xe NPC bị hất văng.
- Tốc độ trên UI là nguồn chuyển động của xe mặt đất.

## HUD

- **Người chơi:** tên xe, tốc độ, số hiện tại, danh sách phương tiện và minimap.
- **Dev mode** (Shift + D + F12): thêm phiên bản, FPS, TPS, frame time, quãng đường thực tế, đo thẳng và số va chạm mỗi giây.

## Discord Rich Presence

Game hiển thị trạng thái trên Discord (tên xe, tốc độ, thời gian chơi). Cần:
1. Tạo ứng dụng trên [Discord Developer Portal](https://discord.com/developers/applications), lấy **Application ID** và đặt vào `src/discord.rs`.
2. Vào **Rich Presence → Art Assets**, tải `icon.png` lên với key `theskyline`.

Nếu Discord không mở, game vẫn chạy bình thường, chỉ không có Rich Presence.

## Icon

- **Cửa sổ game:** lấy từ `assets/icons/icon.png` (đọc khi chạy).
- **File `.exe`:** lấy từ `assets/icons/icon.ico` khi build, cần `windres` (WinLibs). Nếu không có, build vẫn thành công nhưng `.exe` không có icon.

## Lưu ý

- `assets/fonts/ui.ttf` không đưa lên git. Copy một font có tiếng Việt (ví dụ `arial.ttf` trong `C:\Windows\Fonts`) vào `assets/fonts/ui.ttf`.
- Khi làm việc nhóm, mỗi người nên làm trên nhánh riêng và mở Pull Request để gộp vào `main`.