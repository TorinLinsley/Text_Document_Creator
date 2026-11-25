/// 功能：设置项以 JSON 文件形式保存在 exe 同目录
use crate::ffi::*;
use crate::sys::RegDeleteValueW;
use std::path::PathBuf;

pub const MOD_ALT: u32 = 1;
pub const MOD_CONTROL: u32 = 2;
pub const MOD_SHIFT: u32 = 4;
pub const MOD_WIN: u32 = 8;

pub const VK_SPACE: u32 = 0x20;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_UP: u32 = 0x26;
pub const VK_DOWN: u32 = 0x28;
pub const VK_RETURN: u32 = 0x0D;

pub const FILE_NAME: &str = "tdc_config.json";

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_NAME: &str = "Text_Document_Creator";
const TASK_NAME: &str = "TextDocumentCreator";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Copy)]
pub struct Settings {
    pub enable_create: bool,
    pub enable_notepad: bool,
    pub autostart: bool,
    pub vk_create: u32,
    pub mod_create: u32,
    pub vk_notepad: u32,
    pub mod_notepad: u32,
    pub vk_switch_create: u32,
    pub mod_switch_create: u32,
    pub vk_switch_notepad: u32,
    pub mod_switch_notepad: u32,
    pub language: usize,
    pub language_auto: bool,
    pub theme_mode: u32,
}

impl Settings {
    pub fn defaults() -> Settings {
        Settings {
            enable_create: true,
            enable_notepad: false,
            autostart: false,
            vk_create: 0x46,
            mod_create: MOD_WIN,
            vk_notepad: 0x4A,
            mod_notepad: MOD_WIN,
            vk_switch_create: 0,
            mod_switch_create: 0,
            vk_switch_notepad: 0,
            mod_switch_notepad: 0,
            language: crate::i18n::detect(),
            language_auto: true,
            theme_mode: 0,
        }
    }

    pub fn load() -> Settings {
        let text = match std::fs::read_to_string(config_path()) {
            Ok(text) => text,
            Err(_) => {
                let fresh = Settings::defaults();
                fresh.save();
                return fresh;
            }
        };
        let mut s = Settings::defaults();
        if let Some(value) = field(&text, "enable_create") {
            s.enable_create = value == "true";
        }
        if let Some(value) = field(&text, "enable_notepad") {
            s.enable_notepad = value == "true";
        }
        if let Some(value) = field(&text, "autostart") {
            s.autostart = value == "true";
        }
        if let Some(value) = field(&text, "language_auto") {
            s.language_auto = value == "true";
        }
        if let Some(value) = number(&text, "language") {
            if (value as usize) < crate::i18n::COUNT {
                s.language = value as usize;
            }
        }
        if let Some(value) = number(&text, "theme") {
            if value <= 2 {
                s.theme_mode = value;
            }
        }
        if let Some(value) = field(&text, "hotkey_create") {
            let (vk, mods) = crate::hotkey::parse_combo(&value);
            if vk != 0 {
                s.vk_create = vk;
                s.mod_create = mods;
            }
        }
        if let Some(value) = field(&text, "hotkey_notepad") {
            let (vk, mods) = crate::hotkey::parse_combo(&value);
            if vk != 0 {
                s.vk_notepad = vk;
                s.mod_notepad = mods;
            }
        }
        if let Some(value) = field(&text, "hotkey_toggle_create") {
            let (vk, mods) = crate::hotkey::parse_combo(&value);
            s.vk_switch_create = vk;
            s.mod_switch_create = mods;
        }
        if let Some(value) = field(&text, "hotkey_toggle_notepad") {
            let (vk, mods) = crate::hotkey::parse_combo(&value);
            s.vk_switch_notepad = vk;
            s.mod_switch_notepad = mods;
        }
        if s.language_auto {
            s.language = crate::i18n::detect();
        }
        s
    }

    pub fn save(&self) {
        let text = format!(
            "{{\n  \"version\": 1,\n  \"enable_create\": {},\n  \"enable_notepad\": {},\n  \"autostart\": {},\n  \"language_auto\": {},\n  \"language\": {},\n  \"theme\": {},\n  \"hotkey_create\": \"{}\",\n  \"hotkey_notepad\": \"{}\",\n  \"hotkey_toggle_create\": \"{}\",\n  \"hotkey_toggle_notepad\": \"{}\"\n}}\n",
            self.enable_create,
            self.enable_notepad,
            self.autostart,
            self.language_auto,
            self.language,
            self.theme_mode,
            crate::hotkey::shortcut_text(self.vk_create, self.mod_create),
            crate::hotkey::shortcut_text(self.vk_notepad, self.mod_notepad),
            crate::hotkey::shortcut_text(self.vk_switch_create, self.mod_switch_create),
            crate::hotkey::shortcut_text(self.vk_switch_notepad, self.mod_switch_notepad),
        );
        let path = config_path();
        let temp = path.with_extension("json.tmp");
        if std::fs::write(&temp, text.as_bytes()).is_ok() {
            let _ = std::fs::rename(&temp, &path);
        }
    }

    pub fn apply_autostart(&self) {
        let exe = std::env::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        unsafe {
            if self.autostart {
                let args = format!("/Create /TN \"{}\" /TR \"\\\"{}\\\"\" /SC ONLOGON /RL HIGHEST /F", TASK_NAME, exe);
                if !task_tool(&args) {
                    write_run_value(&exe);
                }
            } else {
                task_tool(&format!("/Delete /TN \"{}\" /F", TASK_NAME));
            }
            remove_run_value();
        }
    }

    pub fn reconcile_autostart(&self) {
        if self.autostart != autostart_registered() {
            self.apply_autostart();
        }
    }
}

pub fn config_path() -> PathBuf {
    match std::env::current_exe() {
        Ok(exe) => match exe.parent() {
            Some(dir) => dir.join(FILE_NAME),
            None => PathBuf::from(FILE_NAME),
        },
        Err(_) => PathBuf::from(FILE_NAME),
    }
}

pub fn startup_sync() {
    Settings::load().reconcile_autostart();
}

fn field(text: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let colon = rest.find(':')?;
    let mut chars = rest[colon + 1..].chars().peekable();
    loop {
        match chars.next() {
            Some(c) if c.is_whitespace() => continue,
            Some('"') => {
                let mut value = String::new();
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                    value.push(c);
                }
                return Some(value);
            }
            Some(c) => {
                let mut value = String::new();
                value.push(c);
                for c in chars.by_ref() {
                    if c == ',' || c == '\n' || c == '}' {
                        break;
                    }
                    value.push(c);
                }
                return Some(value.trim().to_string());
            }
            None => return None,
        }
    }
}

fn number(text: &str, key: &str) -> Option<u32> {
    field(text, key)?.parse::<u32>().ok()
}

unsafe fn task_tool(args: &str) -> bool {
    let mut info: crate::sys::STARTUPINFOW = core::mem::zeroed();
    info.cb = core::mem::size_of::<crate::sys::STARTUPINFOW>() as u32;
    let mut process: crate::sys::PROCESS_INFORMATION = core::mem::zeroed();
    let line = w(&format!("schtasks.exe {}", args));
    let created = crate::sys::CreateProcessW(core::ptr::null(), line.as_ptr(), core::ptr::null(), core::ptr::null(), 0, CREATE_NO_WINDOW, core::ptr::null(), core::ptr::null(), &mut info, &mut process);
    if created == 0 {
        return false;
    }
    crate::sys::WaitForSingleObject(process.hProcess, 10000);
    let mut code: u32 = 1;
    crate::sys::GetExitCodeProcess(process.hProcess, &mut code);
    CloseHandle(process.hProcess);
    CloseHandle(process.hThread);
    code == 0
}

unsafe fn task_present() -> bool {
    task_tool(&format!("/Query /TN \"{}\"", TASK_NAME))
}

unsafe fn write_run_value(exe: &str) {
    let sub = w(RUN_KEY);
    let mut key: HKEY = 0;
    if RegCreateKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, core::ptr::null(), 0, KEY_QUERY_VALUE | KEY_SET_VALUE, core::ptr::null_mut(), &mut key, core::ptr::null_mut()) != 0 {
        return;
    }
    let name = w(RUN_NAME);
    let command = format!("\"{}\"", exe);
    let data = w(&command);
    let bytes = (data.len() - 1) * 2;
    RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, data.as_ptr() as *const u8, bytes as u32);
    RegCloseKey(key);
}

unsafe fn remove_run_value() {
    let sub = w(RUN_KEY);
    let mut key: HKEY = 0;
    if RegCreateKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, core::ptr::null(), 0, KEY_QUERY_VALUE | KEY_SET_VALUE, core::ptr::null_mut(), &mut key, core::ptr::null_mut()) != 0 {
        return;
    }
    let name = w(RUN_NAME);
    RegDeleteValueW(key, name.as_ptr());
    RegCloseKey(key);
}

fn autostart_registered() -> bool {
    unsafe { task_present() }
}
