// ─────────────────────────────────────────────────────────────────────────────
//  Win32 overlay setup (Windows only)
// ─────────────────────────────────────────────────────────────────────────────

/// Configure the overlay window via Win32:
///   - WS_EX_LAYERED + WS_EX_TRANSPARENT: color-key transparency + click-through
///   - SetLayeredWindowAttributes with LWA_COLORKEY: DWM makes RGB(1,0,1) transparent
///     at the compositor level — works on all GPU vendors including NVIDIA, unlike
///     per-pixel alpha which requires the GPU to correctly output the alpha channel.
///   - Maximize to fill the current monitor, remove title bar
#[cfg(windows)]
pub unsafe fn win32_setup_overlay() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::*;

    let title: Vec<u16> = OsStr::new("MediaChat")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
    if hwnd.is_null() {
        log::warn!("[overlay] HWND not found — Win32 setup skipped");
        return;
    }

    // Remove title bar / borders
    let style = GetWindowLongW(hwnd, GWL_STYLE);
    SetWindowLongW(
        hwnd,
        GWL_STYLE,
        style
            & !(WS_CAPTION as i32
                | WS_THICKFRAME as i32
                | WS_MINIMIZEBOX as i32
                | WS_MAXIMIZEBOX as i32
                | WS_SYSMENU as i32),
    );

    // Set extended styles: WS_EX_LAYERED (required for color key) + WS_EX_TRANSPARENT (click-through)
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
    SetWindowLongW(
        hwnd,
        GWL_EXSTYLE,
        ex_style | WS_EX_LAYERED as i32 | WS_EX_TRANSPARENT as i32,
    );

    // Color key: RGB(1, 0, 1) = COLORREF 0x00010001 (format: 0x00BBGGRR).
    // DWM makes every pixel of this exact color transparent — GPU-independent.
    // Must match the `key` color in update() and the value in clear_color().
    SetLayeredWindowAttributes(hwnd, 0, 0, LWA_COLORKEY); // black = key color

    SetWindowPos(
        hwnd,
        std::ptr::null_mut(),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
    );

    ShowWindow(hwnd, SW_MAXIMIZE);
    log::info!("[overlay] Win32 color-key overlay setup complete (HWND={hwnd:?}, key=RGB(1,0,1))");
}
