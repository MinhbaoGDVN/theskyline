use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

/// Discord Application ID. Để trống = tắt Rich Presence.
const CLIENT_ID: &str = "1572574492604108932";

pub struct Rpc {
    client: Option<DiscordIpcClient>,
    last: Instant,
    start: i64,
}

impl Rpc {
    pub fn start() -> Self {
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let client = if CLIENT_ID.is_empty() {
            None
        } else {
            match DiscordIpcClient::new(CLIENT_ID) {
                Ok(mut c) => match c.connect() {
                    Ok(()) => Some(c),
                    Err(e) => {
                        eprintln!("[discord] không kết nối được (Discord có đang mở không?): {e}");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("[discord] lỗi tạo client: {e}");
                    None
                }
            }
        };
        Self {
            client,
            last: Instant::now() - Duration::from_secs(60),
            start,
        }
    }

    /// Cập nhật trạng thái, tối đa 5 giây một lần
    pub fn tick(&mut self, details: String, state: String) {
        if self.client.is_none() || self.last.elapsed() < Duration::from_secs(5) {
            return;
        }
        self.last = Instant::now();
        let act = activity::Activity::new()
            .details(&details)
            .state(&state)
            .assets(activity::Assets::new().large_text("The Skyline"))
            .timestamps(activity::Timestamps::new().start(self.start));
        let failed = match self.client.as_mut() {
            Some(c) => c.set_activity(act).is_err(),
            None => return,
        };
        if failed {
            eprintln!("[discord] mất kết nối, tắt Rich Presence");
            self.client = None;
        }
    }
}