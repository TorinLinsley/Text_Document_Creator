/// 功能：设置项在注册表中的持久化与开机自启动登记
use crate::ffi::*;
use crate::sys::RegDeleteValueW;

pub const MOD_ALT: u32 = 1;
pub const MOD_CONTROL: u32 = 2;
pub const MOD_SHIFT: u32 = 4;
pub const MOD_WIN: u32 = 8;

pub const VK_SPACE: u32 = 0x20;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_UP: u32 = 0x26;
pub const VK_DOWN: u32 = 0x28;
pub const VK_RETURN: u32 = 0x0D;

const ROOT: &str = "Software\\TextDocumentCreator";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_NAME: &str = "Text_Document_Creator";

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
        let mut s = Settings::defaults();
        s.enable_create = read_dword("EnableCreate", 1) != 0;
        s.enable_notepad = read_dword("EnableNotepad", 0) != 0;
        s.autostart = read_dword("AutoStart", 0) != 0;
        s.vk_create = read_dword("VkCreate", s.vk_create);
        s.mod_create = read_dword("ModCreate", s.mod_create);
        s.vk_notepad = read_dword("VkNotepad", s.vk_notepad);
        s.mod_notepad = read_dword("ModNotepad", s.mod_notepad);
        s.vk_switch_create = read_dword("VkSwitchCreate", 0);
        s.mod_switch_create = read_dword("ModSwitchCreate", 0);
        s.vk_switch_notepad = read_dword("VkSwitchNotepad", 0);
        s.mod_switch_notepad = read_dword("ModSwitchNotepad", 0);
        s.language_auto = read_dword("LanguageAuto", 1) != 0;
        let language = read_dword("Language", 0xFFFF_FFFF);
        if (language as usize) < crate::i18n::COUNT {
            s.language = language as usize;
        }
        if s.language_auto {
            s.language = crate::i18n::detect();
        }
        let mode = read_dword("ThemeMode", 0xFFFF_FFFF);
        if mode <= 2 {
            s.theme_mode = mode;
        }
        s
    }

    pub fn save(&self) {
        let pairs: [(&str, u32); 14] = [
            ("EnableCreate", self.enable_create as u32),
            ("EnableNotepad", self.enable_notepad as u32),
            ("AutoStart", self.autostart as u32),
            ("VkCreate", self.vk_create),
            ("ModCreate", self.mod_create),
            ("VkNotepad", self.vk_notepad),
            ("ModNotepad", self.mod_notepad),
            ("VkSwitchCreate", self.vk_switch_create),
            ("ModSwitchCreate", self.mod_switch_create),
            ("VkSwitchNotepad", self.vk_switch_notepad),
            ("ModSwitchNotepad", self.mod_switch_notepad),
            ("Language", self.language as u32),
            ("LanguageAuto", self.language_auto as u32),
            ("ThemeMode", self.theme_mode),
        ];
        unsafe {
            let sub = w(ROOT);
            let mut key: HKEY = 0;
            if RegCreateKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, core::ptr::null(), 0, KEY_QUERY_VALUE | KEY_SET_VALUE, core::ptr::null_mut(), &mut key, core::ptr::null_mut()) != 0 {
                return;
            }
            for (name, value) in pairs.iter() {
                let n = w(name);
                RegSetValueExW(key, n.as_ptr(), 0, REG_DWORD, value as *const u32 as *const u8, 4);
            }
            RegCloseKey(key);
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
}

impl Settings {
    pub fn reconcile_autostart(&self) {
        if self.autostart != autostart_registered() {
            self.apply_autostart();
        }
    }
}

pub fn startup_sync() {
    Settings::load().reconcile_autostart();
}

const TASK_NAME: &str = "TextDocumentCreator";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

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

fn legacy_run_present() -> bool {
    unsafe {
        let sub = w(RUN_KEY);
        let mut key: HKEY = 0;
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
            return false;
        }
        let name = w(RUN_NAME);
        let mut size: u32 = 0;
        let mut kind: u32 = 0;
        let status = RegQueryValueExW(key, name.as_ptr(), core::ptr::null_mut(), &mut kind, core::ptr::null_mut(), &mut size);
        RegCloseKey(key);
        status == 0 && size > 0
    }
}

fn autostart_registered() -> bool {
    unsafe { task_present() || legacy_run_present() }
}

fn read_dword(name: &str, fallback: u32) -> u32 {
    unsafe {
        let sub = w(ROOT);
        let mut key: HKEY = 0;
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
            return fallback;
        }
        let n = w(name);
        let mut value: u32 = fallback;
        let mut size: u32 = 4;
        let mut kind: u32 = 0;
        let status = RegQueryValueExW(key, n.as_ptr(), core::ptr::null_mut(), &mut kind, &mut value as *mut u32 as *mut u8, &mut size);
        RegCloseKey(key);
        if status != 0 || kind != REG_DWORD {
            return fallback;
        }
        value
    }
}
