//! Virtual-key helpers: names for the UI and translation of egui keys.

use eframe::egui::Key;

pub fn key_name(vk: u32) -> String {
    match vk {
        0 => return "None".to_string(),
        0x08 => return "Backspace".to_string(),
        0x09 => return "Tab".to_string(),
        0x0D => return "Enter".to_string(),
        0x14 => return "CapsLock".to_string(),
        0x1B => return "Escape".to_string(),
        0x20 => return "Space".to_string(),
        0x21 => return "PageUp".to_string(),
        0x22 => return "PageDown".to_string(),
        0x23 => return "End".to_string(),
        0x24 => return "Home".to_string(),
        0x25 => return "Left".to_string(),
        0x26 => return "Up".to_string(),
        0x27 => return "Right".to_string(),
        0x28 => return "Down".to_string(),
        0x2D => return "Insert".to_string(),
        0x2E => return "Delete".to_string(),
        0x5B => return "LWin".to_string(),
        0x5C => return "RWin".to_string(),
        0x5D => return "Apps".to_string(),
        0x90 => return "NumLock".to_string(),
        0x91 => return "ScrollLock".to_string(),
        0xBA => return ";".to_string(),
        0xBB => return "=".to_string(),
        0xBC => return ",".to_string(),
        0xBD => return "-".to_string(),
        0xBE => return ".".to_string(),
        0xBF => return "/".to_string(),
        0xC0 => return "`".to_string(),
        0xDB => return "[".to_string(),
        0xDC => return "\\".to_string(),
        0xDD => return "]".to_string(),
        0xDE => return "'".to_string(),
        _ => {}
    }

    if (0x30..=0x39).contains(&vk) {
        return char::from_u32(vk).unwrap_or('?').to_string();
    }
    if (0x41..=0x5A).contains(&vk) {
        return char::from_u32(vk).unwrap_or('?').to_string();
    }
    if (0x60..=0x69).contains(&vk) {
        return format!("NumPad{}", vk - 0x60);
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }

    format!("0x{vk:02X}")
}

/// Keys that are modifiers or would be swallowed by the dialog itself are not
/// captured as hotkeys.
pub fn key_to_vk(key: Key) -> Option<u32> {
    let vk = match key {
        Key::Num0 => 0x30,
        Key::Num1 => 0x31,
        Key::Num2 => 0x32,
        Key::Num3 => 0x33,
        Key::Num4 => 0x34,
        Key::Num5 => 0x35,
        Key::Num6 => 0x36,
        Key::Num7 => 0x37,
        Key::Num8 => 0x38,
        Key::Num9 => 0x39,
        Key::A => 0x41,
        Key::B => 0x42,
        Key::C => 0x43,
        Key::D => 0x44,
        Key::E => 0x45,
        Key::F => 0x46,
        Key::G => 0x47,
        Key::H => 0x48,
        Key::I => 0x49,
        Key::J => 0x4A,
        Key::K => 0x4B,
        Key::L => 0x4C,
        Key::M => 0x4D,
        Key::N => 0x4E,
        Key::O => 0x4F,
        Key::P => 0x50,
        Key::Q => 0x51,
        Key::R => 0x52,
        Key::S => 0x53,
        Key::T => 0x54,
        Key::U => 0x55,
        Key::V => 0x56,
        Key::W => 0x57,
        Key::X => 0x58,
        Key::Y => 0x59,
        Key::Z => 0x5A,
        Key::F1 => 0x70,
        Key::F2 => 0x71,
        Key::F3 => 0x72,
        Key::F4 => 0x73,
        Key::F5 => 0x74,
        Key::F6 => 0x75,
        Key::F7 => 0x76,
        Key::F8 => 0x77,
        Key::F9 => 0x78,
        Key::F10 => 0x79,
        Key::F11 => 0x7A,
        Key::F12 => 0x7B,
        Key::F13 => 0x7C,
        Key::F14 => 0x7D,
        Key::F15 => 0x7E,
        Key::F16 => 0x7F,
        Key::F17 => 0x80,
        Key::F18 => 0x81,
        Key::F19 => 0x82,
        Key::F20 => 0x83,
        Key::Space => 0x20,
        Key::PageUp => 0x21,
        Key::PageDown => 0x22,
        Key::End => 0x23,
        Key::Home => 0x24,
        Key::ArrowLeft => 0x25,
        Key::ArrowUp => 0x26,
        Key::ArrowRight => 0x27,
        Key::ArrowDown => 0x28,
        Key::Insert => 0x2D,
        Key::Delete => 0x2E,
        Key::Backspace => 0x08,
        Key::Minus => 0xBD,
        Key::Equals => 0xBB,
        Key::OpenBracket => 0xDB,
        Key::CloseBracket => 0xDD,
        Key::Backslash => 0xDC,
        Key::Semicolon => 0xBA,
        Key::Quote => 0xDE,
        Key::Comma => 0xBC,
        Key::Period => 0xBE,
        Key::Slash => 0xBF,
        Key::Backtick => 0xC0,
        _ => return None,
    };
    Some(vk)
}
