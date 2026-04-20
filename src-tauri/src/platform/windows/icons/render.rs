use std::{ffi::c_void, mem::size_of};

use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, MAX_PATH},
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
            SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        },
        UI::{
            Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
            WindowsAndMessaging::{
                DestroyIcon, DrawIconEx, PrivateExtractIconsW, DI_NORMAL, HICON,
            },
        },
    },
};

pub(super) fn render_executable_icon_png(
    executable_path: &str,
    target_size: i32,
) -> Result<Vec<u8>, String> {
    if let Some(hicon) = extract_executable_icon(executable_path, target_size) {
        let render_result = render_hicon_png(hicon, target_size);
        let _ = unsafe { DestroyIcon(hicon) };
        return render_result;
    }

    let wide_path = encode_wide(executable_path);
    let mut file_info = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            Default::default(),
            Some(&mut file_info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if result == 0 || file_info.hIcon.is_invalid() {
        return Err("Failed to read the app icon handle.".to_string());
    }

    let hicon = file_info.hIcon;
    let render_result = render_hicon_png(hicon, target_size);
    let _ = unsafe { DestroyIcon(hicon) };
    render_result
}

fn extract_executable_icon(executable_path: &str, target_size: i32) -> Option<HICON> {
    let wide_path = encode_wide(executable_path);
    if wide_path.len() > MAX_PATH as usize {
        return None;
    }

    let mut fixed_path = [0u16; MAX_PATH as usize];
    let path_len = wide_path.len().min(fixed_path.len());
    fixed_path[..path_len].copy_from_slice(&wide_path[..path_len]);

    let mut icons = [HICON::default(); 1];
    let extracted = unsafe {
        PrivateExtractIconsW(
            &fixed_path,
            0,
            target_size,
            target_size,
            Some(&mut icons),
            None,
            0,
        )
    };
    (extracted > 0 && !icons[0].is_invalid()).then_some(icons[0])
}

fn render_hicon_png(hicon: HICON, target_size: i32) -> Result<Vec<u8>, String> {
    let screen_dc = unsafe { GetDC(HWND(std::ptr::null_mut())) };
    if screen_dc.is_invalid() {
        return Err("Failed to create the screen drawing context.".to_string());
    }

    let memory_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if memory_dc.is_invalid() {
        let _ = unsafe { ReleaseDC(HWND(std::ptr::null_mut()), screen_dc) };
        return Err("Failed to create the memory drawing context.".to_string());
    }

    let mut bitmap_info = BITMAPINFO::default();
    bitmap_info.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bitmap_info.bmiHeader.biWidth = target_size;
    bitmap_info.bmiHeader.biHeight = -target_size;
    bitmap_info.bmiHeader.biPlanes = 1;
    bitmap_info.bmiHeader.biBitCount = 32;
    bitmap_info.bmiHeader.biCompression = BI_RGB.0;

    let mut bits_ptr = std::ptr::null_mut::<c_void>();
    let bitmap = unsafe {
        CreateDIBSection(
            screen_dc,
            &bitmap_info,
            DIB_RGB_COLORS,
            &mut bits_ptr,
            None,
            0,
        )
    }
    .map_err(|error| format!("Failed to create the icon bitmap: {error}"))?;

    let old_object = unsafe { SelectObject(memory_dc, bitmap) };
    let draw_result = unsafe {
        DrawIconEx(
            memory_dc,
            0,
            0,
            hicon,
            target_size,
            target_size,
            0,
            None,
            DI_NORMAL,
        )
    };

    let _ = unsafe { SelectObject(memory_dc, old_object) };
    let _ = unsafe { DeleteDC(memory_dc) };
    let _ = unsafe { ReleaseDC(HWND(std::ptr::null_mut()), screen_dc) };

    draw_result.map_err(|error| format!("Failed to draw the app icon: {error}"))?;
    if bits_ptr.is_null() {
        let _ = unsafe { DeleteObject(bitmap) };
        return Err("The app icon bitmap buffer is empty.".to_string());
    }

    let pixel_len = (target_size as usize)
        .saturating_mul(target_size as usize)
        .saturating_mul(4);
    let raw_bgra = unsafe { std::slice::from_raw_parts(bits_ptr as *const u8, pixel_len) };
    let mut rgba = raw_bgra.to_vec();
    let _ = unsafe { DeleteObject(bitmap) };

    for pixel in rgba.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }

    let mut png = Vec::new();
    PngEncoder::new(&mut png)
        .write_image(
            &rgba,
            target_size as u32,
            target_size as u32,
            ColorType::Rgba8.into(),
        )
        .map_err(|error| format!("Failed to encode the app icon PNG: {error}"))?;

    Ok(png)
}

fn encode_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
