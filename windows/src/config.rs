use std::fs::File;
use std::io::{ErrorKind, Write};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::util::{data_root, log, stamp};

fn yes() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: usize,
    #[serde(default = "yes")]
    pub autostart: bool,
    #[serde(default)]
    pub muted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub zalo_path: Option<String>,
    #[serde(default = "yes")]
    pub embed: bool,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(default = "yes")]
    pub notify: bool,
    #[serde(default)]
    pub hide_content: bool,
    #[serde(default = "yes")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub tray_hint_shown: bool,
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(skip)]
    pub read_only: bool,
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
            read_only: false,
        }
    }
}

const READ_RETRIES: u32 = 3;

impl Config {
    pub fn load() -> Config {
        let p = data_root().join("config.json");
        let mut attempt = 0;
        let bytes = loop {
            match std::fs::read(&p) {
                Ok(b) => break b,
                Err(e) if e.kind() == ErrorKind::NotFound => return Config::default(),
                Err(e) if attempt < READ_RETRIES => {
                    log(&format!("Đọc config.json lỗi, thử lại: {e}"));
                    attempt += 1;
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(e) => {
                    log(&format!("Không đọc được config.json, chạy chế độ không lưu: {e}"));
                    return Config { read_only: true, ..Config::default() };
                }
            }
        };
        match serde_json::from_slice(&bytes) {
            Ok(cfg) => cfg,
            Err(e) => {
                let backup = data_root().join(format!("config.broken-{}.json", stamp()));
                log(&format!("config.json lỗi, dùng mặc định: {e}"));
                match std::fs::write(&backup, &bytes) {
                    Ok(()) => Config::default(),
                    Err(e) => {
                        log(&format!("Không sao lưu được config.json, chạy chế độ không lưu: {e}"));
                        Config { read_only: true, ..Config::default() }
                    }
                }
            }
        }
    }

    pub fn save(&self) {
        if self.read_only {
            return;
        }
        let p = data_root().join("config.json");
        let tmp = data_root().join("config.json.tmp");
        let result = serde_json::to_vec_pretty(self).map_err(std::io::Error::from).and_then(|s| {
            let mut f = File::create(&tmp)?;
            f.write_all(&s)?;
            f.sync_all()?;
            drop(f);
            std::fs::rename(&tmp, &p)
        });
        if let Err(e) = result {
            log(&format!("Không lưu được config.json: {e}"));
        }
    }

    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.accounts.iter().position(|a| a.id == id)
    }
}

pub const COLORS: [(u32, u32); 8] = [
    (0xE1F5EE, 0x085041),
    (0xEEEDFE, 0x3C3489),
    (0xFAECE7, 0x712B13),
    (0xE6F1FB, 0x0C447C),
    (0xFBEAF0, 0x72243E),
    (0xFAEEDA, 0x633806),
    (0xEAF3DE, 0x27500A),
    (0xF1EFE8, 0x444441),
];
