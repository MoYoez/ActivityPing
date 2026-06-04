use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Mutex, OnceLock,
};

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetMessageW, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

static SUBSCRIBERS: OnceLock<Mutex<Vec<Sender<()>>>> = OnceLock::new();

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _id_thread: u32,
    _time: u32,
) {
    if let Some(lock) = SUBSCRIBERS.get() {
        if let Ok(mut subscribers) = lock.lock() {
            // Drop any subscriber whose receiver has been dropped.
            subscribers.retain(|sender| sender.send(()).is_ok());
        }
    }
}

/// Spawns the dedicated hook thread once. The thread installs a WinEvent hook
/// for foreground-window changes and pumps its message loop so the OS can
/// deliver callbacks. Subsequent calls are no-ops.
fn ensure_hook_thread() {
    SUBSCRIBERS.get_or_init(|| {
        std::thread::Builder::new()
            .name("activityping-foreground-hook".into())
            .spawn(|| unsafe {
                let hook = SetWinEventHook(
                    EVENT_SYSTEM_FOREGROUND,
                    EVENT_SYSTEM_FOREGROUND,
                    None,
                    Some(win_event_proc),
                    0,
                    0,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                );
                if hook.0.is_null() {
                    eprintln!("SetWinEventHook failed; foreground change events are unavailable.");
                    return;
                }

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).into() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            })
            .expect("failed to start the foreground event listener thread");
        Mutex::new(Vec::new())
    });
}

/// Subscribes to foreground-window change events. Each foreground switch sends
/// a `()` on the returned receiver, which callers can use to wake an otherwise
/// idle capture loop early.
pub fn subscribe_foreground_changes() -> Receiver<()> {
    ensure_hook_thread();
    let (tx, rx) = channel();
    if let Some(lock) = SUBSCRIBERS.get() {
        if let Ok(mut subscribers) = lock.lock() {
            subscribers.push(tx);
        }
    }
    rx
}
