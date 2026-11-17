use serde::{Deserialize, Serialize};

use crate::util::{data_root, log};

fn yes() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: usize,
    /// Tự mở khi khởi động ứng dụng
    #[serde(default = "yes")]
    pub autostart: bool,
    /// Tắt thông báo cho riêng tài khoản này
    #[serde(default)]
    pub muted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// Đường dẫn Zalo.exe do người dùng chọn (nếu không tự tìm được)
    #[serde(default)]
    pub zalo_path: Option<String>,
    /// true = nhúng cửa sổ Zalo vào trong ứng dụng; false = cửa sổ gắn kèm
    #[serde(default = "yes")]
    pub embed: bool,
    #[serde(default)]
    pub selected: Option<String>,
    /// Hiện thông báo góc phải khi có tin mới
    #[serde(default = "yes")]
    pub notify: bool,
    /// Ẩn nội dung tin trong thông báo
    #[serde(default)]
    pub hide_content: bool,
    /// Bấm X thì chạy ẩn dưới khay hệ thống
    #[serde(default = "yes")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub tray_hint_shown: bool,
    /// Thanh tài khoản thu gọn (chỉ hiện biểu tượng)
    #[serde(default)]
    pub sidebar_collapsed: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            accounts: vec![],
            zalo_path: None,
            embed: true,
            selected: None,
            notify: true,
            hide_content: false,
            close_to_tray: true,
            tray_hint_shown: false,
            sidebar_collapsed: false,
        }
    }
}

impl Config {
    pub fn load() -> Config {
        let p = data_root().join("config.json");
        match std::fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
                log(&format!("config.json lỗi, dùng mặc định: {e}"));
                let _ = std::fs::copy(&p, data_root().join("config.broken.json"));
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) {
        let p = data_root().join("config.json");
        let tmp = data_root().join("config.json.tmp");
        if let Ok(s) = serde_json::to_string_pretty(self) {
            if std::fs::write(&tmp, s).is_ok() {
                let _ = std::fs::rename(&tmp, &p);
            }
        }
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.accounts.iter().position(|a| a.id == id)
    }
}

/// Bảng màu avatar (nền nhạt, chữ đậm) dạng 0xRRGGBB
pub const COLORS: [(u32, u32); 8] = [
    (0xE1F5EE, 0x085041), // xanh ngọc
    (0xEEEDFE, 0x3C3489), // tím
    (0xFAECE7, 0x712B13), // cam đất
    (0xE6F1FB, 0x0C447C), // xanh dương
    (0xFBEAF0, 0x72243E), // hồng
    (0xFAEEDA, 0x633806), // vàng
    (0xEAF3DE, 0x27500A), // xanh lá
    (0xF1EFE8, 0x444441), // xám
];
