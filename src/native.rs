//! Thin wrappers over the Win32 calls the app needs: synthetic mouse input,
//! the cursor, window lookup for the dark title bar and recording filters.

use std::mem::size_of;

use windows::core::w;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetCursorPos, GetSystemMetrics, GetWindowRect, IsIconic, SetCursorPos,
    SM_CXSCREEN, SM_CYSCREEN,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButtonKind {
    Left,
    Right,
    Middle,
}

impl MouseButtonKind {
    pub fn flags(self) -> (MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS) {
        match self {
            MouseButtonKind::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButtonKind::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            MouseButtonKind::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        }
    }
}

fn mouse_input(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe {
        SendInput(inputs, size_of::<INPUT>() as i32);
    }
}

/// One click (or a double click) with the given button at the current cursor.
pub fn click(button: MouseButtonKind, double: bool) {
    let (down, up) = button.flags();
    let pairs = if double { 2 } else { 1 };
    let mut inputs = Vec::with_capacity(pairs * 2);
    for _ in 0..pairs {
        inputs.push(mouse_input(0, 0, down));
        inputs.push(mouse_input(0, 0, up));
    }
    send(&inputs);
}

pub fn set_cursor_pos(x: i32, y: i32) {
    unsafe {
        let _ = SetCursorPos(x, y);
    }
}

pub fn cursor_pos() -> (i32, i32) {
    let mut point = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut point);
    }
    (point.x, point.y)
}

pub fn screen_size() -> (i32, i32) {
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    (w.max(1), h.max(1))
}

/// Absolute move through `SendInput`, so the low-level hook (and thus remote
/// desktop tools) can see the motion.
pub fn move_absolute(x: i32, y: i32) {
    let (w, h) = screen_size();
    let nx = ((x as f64) * 65535.0 / ((w - 1).max(1) as f64)).round() as i32;
    let ny = ((y as f64) * 65535.0 / ((h - 1).max(1) as f64)).round() as i32;
    let input = mouse_input(nx, ny, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE);
    send(&[input]);
}

pub fn send_button(button: MouseButtonKind, down: bool) {
    let (d, u) = button.flags();
    send(&[mouse_input(0, 0, if down { d } else { u })]);
}

/// Finds our own eframe window by its title.
pub fn find_app_window() -> Option<HWND> {
    for title in [
        w!("Simple Auto Clicker 3.1.1"),
        w!("Simple Auto Clicker Advanced"),
        w!("Running - Simple Auto Clicker 3.1.1"),
        w!("Stopped - Simple Auto Clicker 3.1.1"),
    ] {
        let hwnd = unsafe { FindWindowW(None, title) };
        if let Ok(hwnd) = hwnd {
            if !hwnd.is_invalid() {
                return Some(hwnd);
            }
        }
    }
    None
}

/// Screen rectangle of the window as `(left, top, right, bottom)`.
pub fn window_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect).ok()? };
    Some((rect.left, rect.top, rect.right, rect.bottom))
}

pub fn is_minimized(hwnd: HWND) -> bool {
    unsafe { IsIconic(hwnd).as_bool() }
}

/// Dark or light title bar, supported on Windows 10 1809+ and Windows 11.
pub fn set_dark_title_bar(hwnd: HWND, dark: bool) {
    let value: i32 = if dark { 1 } else { 0 };
    unsafe {
        let ptr = &value as *const i32 as *const core::ffi::c_void;
        let bytes = size_of::<i32>() as u32;
        let mut attribute = DWMWA_USE_IMMERSIVE_DARK_MODE;
        if DwmSetWindowAttribute(hwnd, attribute, ptr, bytes).is_err() {
            // 19 - the attribute number used by early Windows 10 builds.
            attribute = windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE(19);
            let _ = DwmSetWindowAttribute(hwnd, attribute, ptr, bytes);
        }
    }
}
