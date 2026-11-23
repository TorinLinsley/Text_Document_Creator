/// 功能：亮色与暗色配色方案及颜色运算
use crate::ffi::*;

#[derive(Clone, Copy)]
pub struct Theme {
    pub dark: bool,
    pub bg: u32,
    pub rail: u32,
    pub surface: u32,
    pub surface2: u32,
    pub border: u32,
    pub text: u32,
    pub dim: u32,
    pub accent: u32,
    pub accent2: u32,
    pub hover: u32,
    pub press: u32,
    pub sel: u32,
    pub danger: u32,
    pub shadow: u32,
}

impl Theme {
    pub fn light() -> Theme {
        Theme {
            dark: false,
            bg: rgb(0xF4, 0xF5, 0xFA),
            rail: rgb(0xFB, 0xFB, 0xFE),
            surface: rgb(0xFF, 0xFF, 0xFF),
            surface2: rgb(0xF7, 0xF8, 0xFC),
            border: rgb(0xE3, 0xE6, 0xF0),
            text: rgb(0x15, 0x17, 0x1C),
            dim: rgb(0x6E, 0x73, 0x83),
            accent: rgb(0x6C, 0x5C, 0xE7),
            accent2: rgb(0x8B, 0x7C, 0xFF),
            hover: 0x14000000,
            press: 0x22000000,
            sel: 0x1F6C5CE7,
            danger: rgb(0xD9, 0x3A, 0x4A),
            shadow: 0x1A101828,
        }
    }

    pub fn dark() -> Theme {
        Theme {
            dark: true,
            bg: rgb(0x0E, 0x10, 0x15),
            rail: rgb(0x12, 0x14, 0x1A),
            surface: rgb(0x17, 0x1A, 0x21),
            surface2: rgb(0x1C, 0x20, 0x28),
            border: rgb(0x27, 0x2B, 0x36),
            text: rgb(0xE9, 0xEC, 0xF2),
            dim: rgb(0x9A, 0xA2, 0xB1),
            accent: rgb(0x8B, 0x7C, 0xFF),
            accent2: rgb(0xA9, 0x9C, 0xFF),
            hover: 0x18FFFFFF,
            press: 0x28FFFFFF,
            sel: 0x338B7CFF,
            danger: rgb(0xF8, 0x71, 0x7C),
            shadow: 0x33000000,
        }
    }

    pub fn load() -> Theme {
        if system_uses_light() {
            Theme::light()
        } else {
            Theme::dark()
        }
    }
}

pub fn lerp(a: Theme, b: Theme, t: f32) -> Theme {
    let t = t.clamp(0.0, 1.0);
    let ch = |x: u32, y: u32, shift: u32| {
        let p = ((x >> shift) & 0xFF) as f32;
        let q = ((y >> shift) & 0xFF) as f32;
        ((p + (q - p) * t).round() as u32) & 0xFF
    };
    let f = |x: u32, y: u32| (ch(x, y, 24) << 24) | (ch(x, y, 16) << 16) | (ch(x, y, 8) << 8) | ch(x, y, 0);
    Theme {
        dark: if t < 0.5 { a.dark } else { b.dark },
        bg: f(a.bg, b.bg),
        rail: f(a.rail, b.rail),
        surface: f(a.surface, b.surface),
        surface2: f(a.surface2, b.surface2),
        border: f(a.border, b.border),
        text: f(a.text, b.text),
        dim: f(a.dim, b.dim),
        accent: f(a.accent, b.accent),
        accent2: f(a.accent2, b.accent2),
        hover: f(a.hover, b.hover),
        press: f(a.press, b.press),
        sel: f(a.sel, b.sel),
        danger: f(a.danger, b.danger),
        shadow: f(a.shadow, b.shadow),
    }
}

pub fn system_prefers_dark() -> bool {
    !system_uses_light()
}

pub const fn rgb(r: u32, g: u32, b: u32) -> u32 {
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

pub const fn argb(a: u32, c: u32) -> u32 {
    (a << 24) | (c & 0x00FF_FFFF)
}

pub fn with_alpha(c: u32, a: f32) -> u32 {
    let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
    (alpha << 24) | (c & 0x00FF_FFFF)
}

pub fn mix(a: u32, b: u32, t: f32) -> u32 {
    let t = t.clamp(0.0, 1.0);
    let ch = |shift: u32| {
        let x = ((a >> shift) & 0xFF) as f32;
        let y = ((b >> shift) & 0xFF) as f32;
        ((x + (y - x) * t).round() as u32) & 0xFF
    };
    (0xFF00_0000) | (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

pub fn lighten(c: u32, amount: f32) -> u32 {
    mix(c, 0xFFFFFFFF, amount)
}

pub fn darken(c: u32, amount: f32) -> u32 {
    mix(c, 0xFF000000, amount)
}

fn system_uses_light() -> bool {
    unsafe {
    let path = w("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let name = w("AppsUseLightTheme");
    let mut key: HKEY = 0;
    if RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
        return true;
    }
    let mut value: u32 = 1;
    let mut size: u32 = 4;
    let mut kind: u32 = 0;
    let status = RegQueryValueExW(key, name.as_ptr(), core::ptr::null_mut(), &mut kind, &mut value as *mut u32 as *mut u8, &mut size);
    RegCloseKey(key);
    if status != 0 || kind != REG_DWORD {
        return true;
    }
    value != 0
    }
}
