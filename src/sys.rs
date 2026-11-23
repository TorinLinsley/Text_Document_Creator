/// 功能：托盘、全局钩子、Shell 与 COM 的补充 FFI 绑定
use crate::ffi::*;
use core::ffi::c_void;

pub type HRESULT = i32;
pub type BSTR = *const u16;
pub type HOOK = isize;

pub const WH_KEYBOARD_LL: i32 = 13;
pub const THREAD_PRIORITY_TIME_CRITICAL: i32 = 15;
pub const WM_KEYUP_LL: u32 = 0x0101;
pub const NIM_ADD: u32 = 0;
pub const NIM_MODIFY: u32 = 1;
pub const NIM_DELETE: u32 = 2;
pub const NIF_MESSAGE: u32 = 1;
pub const NIF_ICON: u32 = 2;
pub const NIF_TIP: u32 = 4;
pub const MF_STRING: u32 = 0x0000;
pub const MF_SEPARATOR: u32 = 0x0800;
pub const TPM_RETURNCMD: u32 = 0x0100;
pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const WM_CONTEXTMENU: u32 = 0x007B;
pub const WM_LBUTTONUP_W: u32 = 0x0202;
pub const WM_RBUTTONUP_W: u32 = 0x0205;
pub const WM_LBUTTONDBLCLK_W: u32 = 0x0203;
pub const WM_NULL_MSG: u32 = 0x0000;
pub const CSIDL_DESKTOPDIRECTORY: i32 = 0x10;
pub const SW_SHOWNORMAL_CMD: i32 = 1;
pub const CLSCTX_ALL: u32 = 23;
pub const COINIT_APARTMENTTHREADED: u32 = 2;
pub const DISPATCH_PROPERTYGET: u32 = 2;
pub const VT_I4: u16 = 3;
pub const VT_DISPATCH: u16 = 9;
pub const VT_BSTR: u16 = 8;
#[repr(C)]
pub struct STARTUPINFOW {
    pub cb: u32,
    pub lpReserved: *const u16,
    pub lpDesktop: *const u16,
    pub lpTitle: *const u16,
    pub dwX: u32,
    pub dwY: u32,
    pub dwXSize: u32,
    pub dwYSize: u32,
    pub dwXCountChars: u32,
    pub dwYCountChars: u32,
    pub dwFillAttribute: u32,
    pub dwFlags: u32,
    pub wShowWindow: u16,
    pub cbReserved2: u16,
    pub lpReserved2: *mut u8,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub dwProcessId: u32,
    pub dwThreadId: u32,
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetWindowsDirectoryW(buffer: *mut u16, size: u32) -> u32;
    pub fn CreateProcessW(application: *const u16, command: *const u16, process_attr: *const c_void, thread_attr: *const c_void, inherit: i32, flags: u32, env: *const c_void, dir: *const u16, info: *mut STARTUPINFOW, pi: *mut PROCESS_INFORMATION) -> i32;
    pub fn WaitForInputIdle(process: HANDLE, timeout: u32) -> u32;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn SetProcessWorkingSetSize(process: HANDLE, min: isize, max: isize) -> i32;
}

#[link(name = "user32")]
extern "system" {
    pub fn EnumWindows(callback: Option<unsafe extern "system" fn(HWND, LPARAM) -> i32>, param: LPARAM) -> i32;
    pub fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
}

pub const MOD_ALT_ASYNC: i32 = 0x12;
pub const MOD_CONTROL_ASYNC: i32 = 0x11;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GUID {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

#[repr(C)]
pub struct KBDLLHOOKSTRUCT {
    pub vkCode: u32,
    pub scanCode: u32,
    pub flags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uTimeout: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: GUID,
    pub hBalloonIcon: HICON,
}

#[repr(C)]
pub struct VARIANT {
    pub vt: u16,
    pub r1: u16,
    pub r2: u16,
    pub r3: u16,
    pub data: [u8; 16],
}

impl VARIANT {
    pub fn empty() -> VARIANT {
        VARIANT { vt: 0, r1: 0, r2: 0, r3: 0, data: [0; 16] }
    }

    pub fn from_i4(v: i32) -> VARIANT {
        let mut variant = VARIANT::empty();
        variant.vt = VT_I4;
        variant.data[0..4].copy_from_slice(&v.to_le_bytes());
        variant
    }

    pub fn as_usize(&self) -> usize {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&self.data[0..8]);
        usize::from_le_bytes(bytes)
    }

    pub fn as_i4(&self) -> i32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&self.data[0..4]);
        i32::from_le_bytes(bytes)
    }
}

pub type HOOKPROC = unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT;
pub type QueryInterfaceFn = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT;
pub type ReleaseFn = unsafe extern "system" fn(*mut c_void) -> u32;
pub type GetCountFn = unsafe extern "system" fn(*mut c_void, *mut i32) -> HRESULT;
pub type ItemFn = unsafe extern "system" fn(*mut c_void, VARIANT, *mut *mut c_void) -> HRESULT;
pub type GetIDsOfNamesFn = unsafe extern "system" fn(*mut c_void, *const GUID, *const BSTR, u32, u32, *mut i32) -> HRESULT;
pub type InvokeFn = unsafe extern "system" fn(*mut c_void, i32, *const GUID, u32, u16, *mut DISPPARAMS, *mut VARIANT, *mut c_void, *mut u32) -> HRESULT;

#[repr(C)]
pub struct DISPPARAMS {
    pub rgvarg: *mut VARIANT,
    pub rgdispidNamedArgs: *mut i32,
    pub cArgs: u32,
    pub cNamedArgs: u32,
}

pub unsafe fn vtable_entry(ptr: *mut c_void, index: usize) -> usize {
    let vtable = *(ptr as *mut *mut usize);
    *vtable.add(index)
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn Shell_NotifyIconW(op: u32, data: *mut NOTIFYICONDATAW) -> i32;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(menu: HMENU, flags: u32, id: usize, text: *const u16) -> i32;
    pub fn TrackPopupMenu(menu: HMENU, flags: u32, x: i32, y: i32, reserved: i32, hwnd: HWND, rect: *const RECT) -> i32;
    pub fn DestroyMenu(menu: HMENU) -> i32;
    pub fn GetForegroundWindow() -> HWND;
    pub fn GetClassNameW(hwnd: HWND, buf: *mut u16, len: i32) -> i32;
    pub fn GetCursorPos(pt: *mut POINT) -> i32;
    pub fn FindWindowExW(parent: HWND, after: HWND, class: *const u16, name: *const u16) -> HWND;
    pub fn AttachThreadInput(from: u32, to: u32, attach: i32) -> i32;
    pub fn SetActiveWindow(hwnd: HWND) -> HWND;
    pub fn AllowSetForegroundWindow(pid: u32) -> i32;
    pub fn SetWindowsHookExW(id: i32, proc_: HOOKPROC, inst: HINSTANCE, thread: u32) -> HOOK;
    pub fn CallNextHookEx(hook: HOOK, code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn UnhookWindowsHookEx(hook: HOOK) -> i32;
    pub fn MapVirtualKeyW(code: u32, map: u32) -> u32;
    pub fn GetKeyNameTextW(param: i32, buf: *mut u16, len: i32) -> i32;
    pub fn PostMessageW2(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> i32;
    pub fn IsWindowVisible(hwnd: HWND) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetCurrentThreadId() -> u32;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn SetThreadPriority(thread: HANDLE, priority: i32) -> i32;
    pub fn PostThreadMessageW(thread: u32, msg: u32, wp: WPARAM, lp: LPARAM) -> i32;
    pub fn GetModuleFileNameW(inst: HINSTANCE, buf: *mut u16, size: u32) -> u32;
    pub fn Sleep(ms: u32);
    pub fn GetCurrentProcessId() -> u32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn ShellExecuteW(hwnd: HWND, op: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32) -> isize;
    pub fn SHGetFolderPathW(hwnd: HWND, csidl: i32, token: HANDLE, flags: u32, path: *mut u16) -> HRESULT;
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoInitializeEx(reserved: *mut c_void, mode: u32) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(rclsid: *const GUID, outer: *mut c_void, ctx: u32, riid: *const GUID, out: *mut *mut c_void) -> HRESULT;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn RegDeleteValueW(key: HKEY, name: *const u16) -> i32;
}
