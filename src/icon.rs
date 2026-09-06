use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    },
    UI::WindowsAndMessaging::{
        GetClassLongPtrW, GetIconInfo, SendMessageW, GCLP_HICON, GCLP_HICONSM, HICON, ICONINFO,
        WM_GETICON,
    },
};

/// Best-effort small icon for `hwnd` as straight (unmultiplied) RGBA8 pixels: (width, height, rgba bytes).
/// Tries WM_GETICON (small, then small2, then big), then the window class's icon, in that order.
/// Both sources hand back a BORROWED icon handle owned by the window/class -- never destroyed here.
/// Returns None if every source fails (caller should just render the row without an icon).
pub fn get_window_icon_rgba(hwnd: HWND) -> Option<(u32, u32, Vec<u8>)> {
    let hicon = query_icon_handle(hwnd)?;
    icon_to_rgba(hicon)
}

fn query_icon_handle(hwnd: HWND) -> Option<HICON> {
    // ICON_SMALL, ICON_BIG, ICON_SMALL2 -- tried in the order most likely to give a crisp small icon.
    for &kind in &[0u32, 1u32, 2u32] {
        let result = unsafe {
            SendMessageW(
                hwnd,
                WM_GETICON,
                windows::Win32::Foundation::WPARAM(kind as usize),
                windows::Win32::Foundation::LPARAM(0),
            )
        };
        if result.0 != 0 {
            return Some(HICON(result.0));
        }
    }

    for &index in &[GCLP_HICONSM, GCLP_HICON] {
        let raw = unsafe { GetClassLongPtrW(hwnd, index) };
        if raw != 0 {
            return Some(HICON(raw as isize));
        }
    }

    None
}

/// Converts a borrowed HICON into an RGBA buffer via GetIconInfo + GetDIBits.
/// Cleans up the two bitmaps GetIconInfo hands back (caller-owned per its docs) but never touches `hicon` itself.
fn icon_to_rgba(hicon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    unsafe {
        let mut info = ICONINFO::default();
        if GetIconInfo(hicon, &mut info).is_err() {
            return None;
        }
        // hbmMask is always populated; hbmColor is null for legacy monochrome-only icons (rare in practice).
        if info.hbmColor.is_invalid() {
            let _ = DeleteObject(info.hbmMask);
            return None;
        }

        let mut bitmap = BITMAP::default();
        let bytes_written = GetObjectW(
            info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut std::ffi::c_void),
        );
        if bytes_written == 0 {
            let _ = DeleteObject(info.hbmColor);
            let _ = DeleteObject(info.hbmMask);
            return None;
        }

        let width = bitmap.bmWidth as u32;
        let height = bitmap.bmHeight as u32;
        let mut buffer = vec![0u8; (width as usize) * (height as usize) * 4];

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // negative = top-down rows, matching egui::ColorImage's row order
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let dc = CreateCompatibleDC(None);
        let copied = GetDIBits(
            dc,
            info.hbmColor,
            0,
            height,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            &mut bmi,
            DIB_RGB_COLORS,
        );
        let _ = DeleteDC(dc);
        let _ = DeleteObject(info.hbmColor);
        let _ = DeleteObject(info.hbmMask);

        if copied == 0 {
            return None;
        }

        // GetDIBits fills BGRA; egui wants RGBA. Swap B/R per pixel in place.
        let mut has_alpha = false;
        for px in buffer.chunks_exact_mut(4) {
            px.swap(0, 2);
            if px[3] != 0 {
                has_alpha = true;
            }
        }
        // Some legacy icons report an all-zero alpha channel despite being visually opaque.
        if !has_alpha {
            for px in buffer.chunks_exact_mut(4) {
                px[3] = 255;
            }
        }

        Some((width, height, buffer))
    }
}
