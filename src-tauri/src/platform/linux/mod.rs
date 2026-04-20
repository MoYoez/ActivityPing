mod command;
mod desktop_entries;
mod foreground;
mod icons;
mod media;
mod self_test;
mod theme_icons;
mod wayland;
mod x11;

pub use foreground::get_foreground_snapshot_for_reporting;
pub use icons::get_foreground_app_icon;
pub use media::get_now_playing;
pub use self_test::{request_accessibility_permission, run_self_test};
