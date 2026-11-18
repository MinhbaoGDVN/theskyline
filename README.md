# The Skyline

Game 3D thế giới mở viết bằng Rust và macroquad. Lái ô tô, xe máy, máy bay, xe tăng quanh thành phố.

## Yêu cầu

- Rust (rustup.rs)
- Windows 10/11, GPU hỗ trợ OpenGL 3.3 trở lên

## Chạy game

```powershell
cargo run --release
```

## Điều khiển

| Phím | Tác dụng |
| --- | --- |
| W / S | Ga / phanh lùi (máy bay: ngóc đầu / cúi đầu) |
| A / D | Lái trái / phải (máy bay: nghiêng cánh để rẽ) |
| Space | Phanh tay |
| Shift / Ctrl | Tăng / giảm ga (máy bay) |
| F | Bắn đạn (xe tăng) |
| 1, 2, 3, 4 | Chọn ô tô, xe máy, máy bay, xe tăng |
| Q / E | Đổi phương tiện trước / sau |
| R | Hồi phục phương tiện (khi máy bay rơi) |

## Cấu trúc

```
theskyline/
├── Cargo.toml
├── assets/
│   ├── models/     # OBJ + palette.mtl (car, bike, plane, tank)
│   └── fonts/      # ui.ttf, không đưa lên git
└── src/
    ├── main.rs     # Cửa sổ và vòng lặp chính
    ├── game.rs     # Trạng thái game, cập nhật và vẽ
    ├── config.rs   # Hằng số thế giới
    ├── obj.rs      # Đọc file OBJ và MTL
    ├── models.rs   # Nạp model
    ├── render.rs   # Đổ bóng theo đỉnh, vẽ model, gom mesh
    ├── city.rs     # Đường, khối nhà, tòa nhà, va chạm, phá hủy
    ├── vehicle.rs  # Vật lý ô tô, xe máy, xe tăng, máy bay
    ├── traffic.rs  # Xe NPC
    ├── combat.rs   # Đạn, vụ nổ, hạt bụi
    ├── camera.rs   # Camera bám xe
    ├── input.rs    # Đọc phím
    └── hud.rs      # Bảng thông tin, thông báo, hướng dẫn
```

## Tòa nhà bị phá

Mỗi tòa có 150 máu. Đạn trúng trực tiếp gây 80 sát thương, nên cần 2 phát để sập. Tòa bị hư hại sẽ tối dần theo máu. Vụ nổ chỉ ảnh hưởng tòa trong bán kính 6 m.

## Giới hạn đồ họa

Ánh sáng được tính sẵn vào màu đỉnh, không có bóng đổ và sương mù, để chạy được trên GPU yếu.