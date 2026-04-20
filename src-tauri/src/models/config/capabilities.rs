use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    pub realtime_reporter: bool,
    pub tray: bool,
    pub platform_self_test: bool,
    pub discord_presence: bool,
    pub autostart: bool,
}

#[cfg(desktop)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: true,
        tray: true,
        platform_self_test: true,
        discord_presence: true,
        autostart: true,
    }
}

#[cfg(mobile)]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}

#[cfg(not(any(desktop, mobile)))]
pub fn default_client_capabilities() -> ClientCapabilities {
    ClientCapabilities {
        realtime_reporter: false,
        tray: false,
        platform_self_test: false,
        discord_presence: false,
        autostart: false,
    }
}
