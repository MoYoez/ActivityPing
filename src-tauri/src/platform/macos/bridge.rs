use std::ffi::{c_char, CStr, CString};

unsafe extern "C" {
    fn waken_frontmost_app_name() -> *mut c_char;
    fn waken_frontmost_app_bundle_identifier() -> *mut c_char;
    fn waken_frontmost_window_title() -> *mut c_char;
    fn waken_bundle_icon_png_base64(
        bundle_identifier: *const c_char,
        target_size: i32,
    ) -> *mut c_char;
    fn waken_bundle_display_name(bundle_identifier: *const c_char) -> *mut c_char;
    fn waken_accessibility_is_trusted() -> bool;
    fn waken_request_accessibility_permission() -> bool;
    fn waken_string_free(value: *mut c_char);
}

pub(super) fn read_frontmost_app_name() -> Option<String> {
    read_bridge_string(waken_frontmost_app_name)
}

pub(super) fn read_frontmost_app_bundle_identifier() -> Option<String> {
    read_bridge_string(waken_frontmost_app_bundle_identifier)
}

pub(super) fn read_frontmost_window_title() -> Option<String> {
    read_bridge_string(waken_frontmost_window_title)
}

pub(super) fn read_bundle_icon_png_base64(
    bundle_identifier: &str,
    target_size: i32,
) -> Option<String> {
    let c_bundle_identifier = CString::new(bundle_identifier.trim()).ok()?;
    let ptr = unsafe { waken_bundle_icon_png_base64(c_bundle_identifier.as_ptr(), target_size) };
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(value)
}

pub fn read_bundle_display_name(bundle_identifier: &str) -> Option<String> {
    let c_bundle_identifier = CString::new(bundle_identifier.trim()).ok()?;
    read_bridge_string_with_input(waken_bundle_display_name, c_bundle_identifier.as_c_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub(super) fn accessibility_permission_granted() -> bool {
    unsafe { waken_accessibility_is_trusted() }
}

pub(super) fn request_accessibility_permission_via_bridge() -> bool {
    unsafe { waken_request_accessibility_permission() }
}

fn read_bridge_string(fetch: unsafe extern "C" fn() -> *mut c_char) -> Option<String> {
    let ptr = unsafe { fetch() };
    if ptr.is_null() {
        return None;
    }

    let value = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(value)
}

fn read_bridge_string_with_input(
    fetch: unsafe extern "C" fn(*const c_char) -> *mut c_char,
    value: &std::ffi::CStr,
) -> Option<String> {
    let ptr = unsafe { fetch(value.as_ptr()) };
    if ptr.is_null() {
        return None;
    }

    let decoded = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
    unsafe { waken_string_free(ptr) };
    Some(decoded)
}
