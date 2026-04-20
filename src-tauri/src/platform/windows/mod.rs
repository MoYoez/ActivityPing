mod foreground;
mod icons;
mod media;
mod process;
mod self_test;

pub(super) const APP_ICON_SIZE: i32 = 256;

pub use foreground::get_foreground_snapshot_for_reporting;
pub use icons::get_foreground_app_icon;
pub use media::get_now_playing;
pub use self_test::{request_accessibility_permission, run_self_test};
