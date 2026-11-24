/// 功能：全局快捷键钩子、键名文本、文件创建与记事本启动
use crate::config::*;
use crate::ffi::*;
use crate::sys::*;
use core::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};

pub const MSG_CREATE_FILE: u32 = WM_APP + 1;
pub const MSG_OPEN_NOTEPAD: u32 = WM_APP + 2;
pub const MSG_TOGGLE_CREATE: u32 = WM_APP + 3;
pub const MSG_TOGGLE_NOTEPAD: u32 = WM_APP + 4;

static CAPTURING: AtomicBool = AtomicBool::new(false);
static mut SETTINGS: Option<Settings> = None;
static mut TARGET: HWND = 0;
static mut HOOK: isize = 0;
static mut HOOK_THREAD: u32 = 0;
static LAST_FIRE: AtomicU64 = AtomicU64::new(0);
static PRESSED: AtomicU32 = AtomicU32::new(0);
static FOCUS: AtomicUsize = AtomicUsize::new(0);
static WIN_SUPPRESS_UP: AtomicBool = AtomicBool::new(false);
static SWALLOW_UP: AtomicU32 = AtomicU32::new(0);

const BIT_CTRL_LEFT: u32 = 1 << 0;
const BIT_CTRL_RIGHT: u32 = 1 << 1;
const BIT_SHIFT_LEFT: u32 = 1 << 2;
const BIT_SHIFT_RIGHT: u32 = 1 << 3;
const BIT_ALT_LEFT: u32 = 1 << 4;
const BIT_ALT_RIGHT: u32 = 1 << 5;
const BIT_WIN_LEFT: u32 = 1 << 6;
const BIT_WIN_RIGHT: u32 = 1 << 7;
const BIT_WIN: u32 = BIT_WIN_LEFT | BIT_WIN_RIGHT;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;

fn modifier_bit(vk: u32) -> u32 {
    match vk {
        0x11 | 0xA2 => BIT_CTRL_LEFT,
        0xA3 => BIT_CTRL_RIGHT,
        0x10 | 0xA0 => BIT_SHIFT_LEFT,
        0xA1 => BIT_SHIFT_RIGHT,
        0x12 | 0xA4 => BIT_ALT_LEFT,
        0xA5 => BIT_ALT_RIGHT,
        0x5B => BIT_WIN_LEFT,
        0x5C => BIT_WIN_RIGHT,
        _ => 0,
    }
}

pub fn tracked_mods() -> u32 {
    let bits = PRESSED.load(Ordering::Relaxed);
    let mut mods = 0;
    if bits & (BIT_CTRL_LEFT | BIT_CTRL_RIGHT) != 0 {
        mods |= MOD_CONTROL;
    }
    if bits & (BIT_SHIFT_LEFT | BIT_SHIFT_RIGHT) != 0 {
        mods |= MOD_SHIFT;
    }
    if bits & (BIT_ALT_LEFT | BIT_ALT_RIGHT) != 0 {
        mods |= MOD_ALT;
    }
    if bits & BIT_WIN != 0 {
        mods |= MOD_WIN;
    }
    mods
}

pub fn set_capturing(value: bool) {
    CAPTURING.store(value, Ordering::Relaxed);
}

pub fn is_modifier(vk: u32) -> bool {
    matches!(vk, 0x10 | 0x11 | 0x12 | 0x5B | 0x5C | 0xA0 | 0xA1 | 0xA2 | 0xA3 | 0xA4 | 0xA5)
}

pub fn current_mods() -> u32 {
    let tracked = tracked_mods();
    if tracked != 0 {
        return tracked;
    }
    unsafe {
        let mut mods = 0;
        if GetAsyncKeyState(0x11) as u16 & 0x8000 != 0 {
            mods |= MOD_CONTROL;
        }
        if GetAsyncKeyState(0x10) as u16 & 0x8000 != 0 {
            mods |= MOD_SHIFT;
        }
        if GetAsyncKeyState(0x12) as u16 & 0x8000 != 0 {
            mods |= MOD_ALT;
        }
        if GetAsyncKeyState(0x5B) as u16 & 0x8000 != 0 || GetAsyncKeyState(0x5C) as u16 & 0x8000 != 0 {
            mods |= MOD_WIN;
        }
        mods
    }
}

pub fn shortcut_text(vk: u32, mods: u32) -> String {
    if vk == 0 {
        return String::new();
    }
    let mut parts: Vec<String> = Vec::new();
    if mods & MOD_CONTROL != 0 {
        parts.push("Ctrl".to_string());
    }
    if mods & MOD_SHIFT != 0 {
        parts.push("Shift".to_string());
    }
    if mods & MOD_ALT != 0 {
        parts.push("Alt".to_string());
    }
    if mods & MOD_WIN != 0 {
        parts.push("Win".to_string());
    }
    parts.push(key_name(vk));
    parts.join("+")
}

pub fn key_name(vk: u32) -> String {
    if (0x41..=0x5A).contains(&vk) {
        return ((vk as u8) as char).to_string();
    }
    if (0x30..=0x39).contains(&vk) {
        return ((vk as u8) as char).to_string();
    }
    if (0x60..=0x69).contains(&vk) {
        return format!("Num{}", vk - 0x60);
    }
    if (0x70..=0x87).contains(&vk) {
        return format!("F{}", vk - 0x6F);
    }
    let name = match vk {
        0x08 => "Backspace",
        0x09 => "Tab",
        0x0C => "Clear",
        0x0D => "Enter",
        0x13 => "Pause",
        0x14 => "CapsLock",
        0x1B => "Escape",
        0x5D => "Menu",
        0x20 => "Space",
        0x21 => "PageUp",
        0x22 => "PageDown",
        0x23 => "End",
        0x24 => "Home",
        0x25 => "Left",
        0x26 => "Up",
        0x27 => "Right",
        0x28 => "Down",
        0x2C => "PrintScreen",
        0x2D => "Insert",
        0x2E => "Delete",
        0x6A => "Num*",
        0x6B => "Num+",
        0x6D => "Num-",
        0x6E => "Num.",
        0x6F => "Num/",
        0x90 => "NumLock",
        0x91 => "ScrollLock",
        0xBA => ";",
        0xBB => "=",
        0xBC => ",",
        0xBD => "-",
        0xBE => ".",
        0xBF => "/",
        0xC0 => "Backquote",
        0xDB => "[",
        0xDC => "Backslash",
        0xDD => "]",
        0xDE => "'",
        0xE2 => "Oem102",
        0xB0 => "MediaNext",
        0xB1 => "MediaPrev",
        0xB2 => "MediaStop",
        0xB3 => "MediaPlay",
        0xAD => "VolumeMute",
        0xAE => "VolumeDown",
        0xAF => "VolumeUp",
        0xA6 => "BrowserBack",
        0xA7 => "BrowserForward",
        0xA8 => "BrowserRefresh",
        0xA9 => "BrowserStop",
        0xAA => "BrowserSearch",
        0xAB => "BrowserFav",
        _ => return format!("VK-{}", vk),
    };
    name.to_string()
}

pub unsafe fn update(settings: Settings) {
    SETTINGS = Some(settings);
}

pub unsafe fn install(hwnd: HWND) -> bool {
    TARGET = hwnd;
    std::thread::spawn(|| unsafe {
        let thread = GetCurrentThread();
        SetThreadPriority(thread, THREAD_PRIORITY_TIME_CRITICAL);
        HOOK_THREAD = GetCurrentThreadId();
        let instance = GetModuleHandleW(core::ptr::null());
        HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, hook_proc, instance, 0);
        if HOOK == 0 {
            HOOK_THREAD = 0;
            return;
        }
        let mut message: MSG = core::mem::zeroed();
        while GetMessageW(&mut message, 0, 0, 0) > 0 {}
        UnhookWindowsHookEx(HOOK);
        HOOK = 0;
        HOOK_THREAD = 0;
    });
    let mut spin = 0;
    while spin < 250 && HOOK == 0 {
        Sleep(2);
        spin += 1;
    }
    HOOK != 0
}

pub unsafe fn uninstall() {
    if HOOK_THREAD != 0 {
        PostThreadMessageW(HOOK_THREAD, WM_QUIT, 0, 0);
        HOOK_THREAD = 0;
    }
}
const INJECT_MAGIC: usize = 0x5444_4331;
const KEYEVENTF_KEYUP: u32 = 2;

static WIN_LOCAL: AtomicBool = AtomicBool::new(false);
static WIN_REPLAYED: AtomicBool = AtomicBool::new(false);
static WIN_FIRED: AtomicBool = AtomicBool::new(false);
static WIN_USED: AtomicBool = AtomicBool::new(false);
static WIN_CAPTURED: AtomicBool = AtomicBool::new(false);
static CAP_VK: AtomicU32 = AtomicU32::new(0);
static CAP_MODS: AtomicU32 = AtomicU32::new(0);
pub const MSG_CAPTURED: u32 = WM_APP + 11;

pub fn captured() -> (u32, u32) {
    (CAP_VK.load(Ordering::Relaxed), CAP_MODS.load(Ordering::Relaxed))
}

unsafe fn send_key(vk: u32, down: bool) {
    let flags = if down { 0 } else { KEYEVENTF_KEYUP };
    keybd_event(vk as u8, 0, flags, INJECT_MAGIC);
}

unsafe extern "system" fn hook_proc(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code != 0 {
        return CallNextHookEx(HOOK, code, wp, lp);
    }
    let message = wp as u32;
    let info = &*(lp as *const KBDLLHOOKSTRUCT);
    let vk = info.vkCode & 0xFF;
    let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
    let up = message == WM_KEYUP || message == WM_SYSKEYUP;
    if info.dwExtraInfo == INJECT_MAGIC {
        return CallNextHookEx(HOOK, code, wp, lp);
    }
    let bit = modifier_bit(vk);
    let win_key = vk == VK_LWIN || vk == VK_RWIN;
    if win_key && down {
        PRESSED.fetch_or(bit, Ordering::Relaxed);
        WIN_LOCAL.store(true, Ordering::Relaxed);
        WIN_REPLAYED.store(false, Ordering::Relaxed);
        WIN_FIRED.store(false, Ordering::Relaxed);
        WIN_USED.store(false, Ordering::Relaxed);
        WIN_CAPTURED.store(CAPTURING.load(Ordering::Relaxed), Ordering::Relaxed);
        return 1;
    }
    if win_key && up {
        let local = WIN_LOCAL.swap(false, Ordering::Relaxed);
        PRESSED.fetch_and(!BIT_WIN, Ordering::Relaxed);
        if !local {
            return CallNextHookEx(HOOK, code, wp, lp);
        }
        let replayed = WIN_REPLAYED.swap(false, Ordering::Relaxed);
        let fired = WIN_FIRED.swap(false, Ordering::Relaxed);
        let used = WIN_USED.swap(false, Ordering::Relaxed);
        let captured = WIN_CAPTURED.swap(false, Ordering::Relaxed);
        if replayed {
            send_key(VK_LWIN, false);
        } else if !fired && !used && !captured && !CAPTURING.load(Ordering::Relaxed) {
            send_key(VK_LWIN, true);
            send_key(VK_LWIN, false);
        }
        return 1;
    }
    if bit != 0 {
        if down {
            PRESSED.fetch_or(bit, Ordering::Relaxed);
        } else {
            PRESSED.fetch_and(!bit, Ordering::Relaxed);
        }
    }
    if down && bit == 0 && CAPTURING.load(Ordering::Relaxed) {
        let recorded = if vk == VK_ESCAPE { 0 } else { vk };
        CAP_VK.store(recorded, Ordering::Relaxed);
        CAP_MODS.store(tracked_mods(), Ordering::Relaxed);
        PostMessageW(TARGET, MSG_CAPTURED, 0, 0);
        return 1;
    }
    if down && bit == 0 && WIN_LOCAL.load(Ordering::Relaxed) {
        WIN_USED.store(true, Ordering::Relaxed);
    }
    if up && SWALLOW_UP.swap(0, Ordering::Relaxed) == vk {
        return 1;
    }
    if down && !CAPTURING.load(Ordering::Relaxed) {
        let mods = tracked_mods();
        if let Some(settings) = SETTINGS {
            let hit = |key: u32, key_mods: u32| key != 0 && vk == key && mods == key_mods;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            let repeat = now.saturating_sub(LAST_FIRE.load(Ordering::Relaxed)) < 120;
            let mut fired = false;
            let mut action = 0;
            if hit(settings.vk_create, settings.mod_create) && settings.enable_create {
                action = MSG_CREATE_FILE;
                fired = true;
            } else if hit(settings.vk_notepad, settings.mod_notepad) && settings.enable_notepad {
                action = MSG_OPEN_NOTEPAD;
                fired = true;
            } else if hit(settings.vk_switch_create, settings.mod_switch_create) {
                action = MSG_TOGGLE_CREATE;
                fired = true;
            } else if hit(settings.vk_switch_notepad, settings.mod_switch_notepad) {
                action = MSG_TOGGLE_NOTEPAD;
                fired = true;
            }
            if fired {
                if !repeat {
                    LAST_FIRE.store(now, Ordering::Relaxed);
                    FOCUS.store(GetForegroundWindow() as usize, Ordering::Relaxed);
                    PostMessageW(TARGET, action, 0, 0);
                }
                if mods & MOD_WIN != 0 {
                    WIN_FIRED.store(true, Ordering::Relaxed);
                }
                SWALLOW_UP.store(vk, Ordering::Relaxed);
                return 1;
            }
        }
        if WIN_LOCAL.load(Ordering::Relaxed) && !WIN_REPLAYED.load(Ordering::Relaxed) {
            send_key(VK_LWIN, true);
            WIN_REPLAYED.store(true, Ordering::Relaxed);
        }
    }
    CallNextHookEx(HOOK, code, wp, lp)
}
pub fn document_name(lang: usize) -> &'static str {
    match lang {
        1 => "新增文字文件",
        2 => "New Text Document",
        _ => "新建文本文档",
    }
}

pub unsafe fn create_text_file(lang: usize) -> Result<String, ()> {
    let folder = current_folder();
    let base = document_name(lang);
    let mut index = 0;
    loop {
        let name = if index == 0 {
            format!("{}.txt", base)
        } else {
            format!("{}_{}.txt", base, index)
        };
        let path = std::path::Path::new(&folder).join(&name);
        match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return Ok(path.to_string_lossy().to_string()),
            Err(error) => {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    index += 1;
                    if index > 99 {
                        return Err(());
                    }
                } else {
                    return Err(());
                }
            }
        }
    }
}

pub unsafe fn current_folder() -> String {
    let stored = FOCUS.swap(0, Ordering::Relaxed);
    let foreground = if stored != 0 { stored as HWND } else { GetForegroundWindow() };
    if foreground != 0 {
        let mut buffer = [0u16; 128];
        let len = GetClassNameW(foreground, buffer.as_mut_ptr(), buffer.len() as i32).max(0) as usize;
        let class = String::from_utf16_lossy(&buffer[..len.min(buffer.len())]);
        if class.eq_ignore_ascii_case("CabinetWClass") || class.eq_ignore_ascii_case("ExploreWClass") {
            if let Some(path) = explorer_folder(foreground) {
                if std::path::Path::new(&path).exists() {
                    return path;
                }
            }
        }
    }
    desktop_folder()
}

pub unsafe fn desktop_folder() -> String {
    let mut buffer = [0u16; 260];
    if SHGetFolderPathW(0, CSIDL_DESKTOPDIRECTORY, 0, 0, buffer.as_mut_ptr()) != 0 {
        return String::new();
    }
    let len = buffer.iter().position(|c| *c == 0).unwrap_or(0);
    String::from_utf16_lossy(&buffer[..len])
}

unsafe fn explorer_folder(hwnd: HWND) -> Option<String> {
    let clsid = GUID { d1: 0x9BA0_5972, d2: 0xF6A8, d3: 0x11CF, d4: [0xA4, 0x42, 0x00, 0xA0, 0xC9, 0x0A, 0x8F, 0x39] };
    let iid = GUID { d1: 0x85CB_6900, d2: 0x4D95, d3: 0x11CF, d4: [0x96, 0x0C, 0x00, 0x80, 0xC7, 0xF4, 0xEE, 0x85] };
    CoInitializeEx(core::ptr::null_mut(), COINIT_APARTMENTTHREADED);
    let mut shell: *mut c_void = core::ptr::null_mut();
    let mut result = None;
    if CoCreateInstance(&clsid, core::ptr::null_mut(), CLSCTX_ALL, &iid, &mut shell) == 0 && !shell.is_null() {
        let count_fn: GetCountFn = core::mem::transmute(vtable_entry(shell, 7));
        let item_fn: ItemFn = core::mem::transmute(vtable_entry(shell, 8));
        let mut count = 0;
        count_fn(shell, &mut count);
        for index in 0..count.min(64) {
            let mut dispatch: *mut c_void = core::ptr::null_mut();
            if item_fn(shell, VARIANT::from_i4(index), &mut dispatch) != 0 || dispatch.is_null() {
                continue;
            }
            if let Some(window) = property_usize(dispatch, "HWND") {
                if window == hwnd as usize {
                    if let Some(url) = property_string(dispatch, "LocationURL") {
                        if let Some(path) = url_to_path(&url) {
                            result = Some(path);
                        }
                    }
                }
            }
            let release: ReleaseFn = core::mem::transmute(vtable_entry(dispatch, 2));
            release(dispatch);
            if result.is_some() {
                break;
            }
        }
        let release: ReleaseFn = core::mem::transmute(vtable_entry(shell, 2));
        release(shell);
    }
    CoUninitialize();
    result
}

unsafe fn property_value(dispatch: *mut c_void, name: &str) -> Option<VARIANT> {
    let get_ids: GetIDsOfNamesFn = core::mem::transmute(vtable_entry(dispatch, 5));
    let invoke: InvokeFn = core::mem::transmute(vtable_entry(dispatch, 6));
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    let null_guid = GUID::default();
    let mut dispid: i32 = 0;
    if get_ids(dispatch, &null_guid, &wide.as_ptr(), 1, 0x0400, &mut dispid) != 0 {
        return None;
    }
    let mut result = VARIANT::empty();
    let mut params = DISPPARAMS { rgvarg: core::ptr::null_mut(), rgdispidNamedArgs: core::ptr::null_mut(), cArgs: 0, cNamedArgs: 0 };
    if invoke(dispatch, dispid, &null_guid, 0x0400, 2, &mut params, &mut result, core::ptr::null_mut(), core::ptr::null_mut()) != 0 {
        return None;
    }
    Some(result)
}

unsafe fn property_usize(dispatch: *mut c_void, name: &str) -> Option<usize> {
    let value = property_value(dispatch, name)?;
    match value.vt {
        3 | 0x13 => Some(value.as_i4() as usize),
        0x14 | 0x15 => Some(value.as_usize()),
        _ => None,
    }
}

unsafe fn property_string(dispatch: *mut c_void, name: &str) -> Option<String> {
    let value = property_value(dispatch, name)?;
    if value.vt != VT_BSTR {
        return None;
    }
    let pointer = value.as_usize() as *const u16;
    if pointer.is_null() {
        return None;
    }
    let mut len = 0usize;
    while *pointer.add(len) != 0 && len < 4096 {
        len += 1;
    }
    Some(String::from_utf16_lossy(core::slice::from_raw_parts(pointer, len)))
}

fn url_to_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file:///").or_else(|| url.strip_prefix("file://"))?;
    let mut text = String::with_capacity(rest.len());
    let bytes = rest.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &rest[index + 1..index + 3];
            if let Ok(value) = u8::from_str_radix(hex, 16) {
                text.push(value as char);
                index += 3;
                continue;
            }
        }
        text.push(bytes[index] as char);
        index += 1;
    }
    let decoded = text.as_bytes().to_vec();
    let utf8 = String::from_utf8(decoded).unwrap_or(text);
    let normalized = utf8.replace('/', "\\");
    let trimmed = normalized.trim_start_matches('\\').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

struct PidWindow {
    pid: u32,
    hwnd: HWND,
}

unsafe extern "system" fn find_pid_window(hwnd: HWND, param: LPARAM) -> i32 {
    let target = &mut *(param as *mut PidWindow);
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid != target.pid {
        return 1;
    }
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    target.hwnd = hwnd;
    0
}

pub unsafe fn open_notepad() -> bool {
    let mut dir = [0u16; 260];
    let len = GetWindowsDirectoryW(dir.as_mut_ptr(), 260);
    if len == 0 {
        return false;
    }
    let mut path: Vec<u16> = dir[..len as usize].to_vec();
    path.extend("\\System32\\notepad.exe".encode_utf16());
    path.push(0);
    let mut info: STARTUPINFOW = core::mem::zeroed();
    info.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process: PROCESS_INFORMATION = core::mem::zeroed();
    let created = CreateProcessW(
        core::ptr::null(),
        path.as_ptr(),
        core::ptr::null(),
        core::ptr::null(),
        0,
        0,
        core::ptr::null(),
        core::ptr::null(),
        &mut info,
        &mut process,
    );
    if created == 0 {
        return false;
    }
    let mut window: HWND = 0;
    for _ in 0..14 {
        let mut ctx = PidWindow { pid: process.dwProcessId, hwnd: 0 };
        EnumWindows(Some(find_pid_window), &mut ctx as *mut PidWindow as LPARAM);
        if ctx.hwnd != 0 {
            window = ctx.hwnd;
            break;
        }
        Sleep(25);
    }
    if window != 0 {
        let target_thread = GetWindowThreadProcessId(window, core::ptr::null_mut());
        let current_thread = GetCurrentThreadId();
        AttachThreadInput(current_thread, target_thread, 1);
        AllowSetForegroundWindow(0xFFFF_FFFF);
        SetForegroundWindow(window);
        SetActiveWindow(window);
        AttachThreadInput(current_thread, target_thread, 0);
    }
    CloseHandle(process.hThread);
    CloseHandle(process.hProcess);
    true
}
