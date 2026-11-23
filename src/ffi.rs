#![allow(non_snake_case, non_camel_case_types, dead_code)]

// 功能：Win32 / GDI+ / DWM 的 FFI 绑定与原语类型
pub use core::ffi::c_void;

pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HMENU = isize;
pub type HICON = isize;
pub type HCURSOR = isize;
pub type HBRUSH = isize;
pub type HFONT = isize;
pub type HDC = isize;
pub type HBITMAP = isize;
pub type HPEN = isize;
pub type HGDIOBJ = isize;
pub type HMONITOR = isize;
pub type HGLOBAL = isize;
pub type HANDLE = isize;
pub type HKEY = isize;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

pub type WNDPROC = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: Option<WNDPROC>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: i32,
    pub rcPaint: RECT,
    pub fRestore: i32,
    pub fIncUpdate: i32,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct MINMAXINFO {
    pub ptReserved: POINT,
    pub ptMaxSize: POINT,
    pub ptMaxPosition: POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
}

#[repr(C)]
pub struct WINDOWPOS {
    pub hwnd: HWND,
    pub hwndInsertAfter: HWND,
    pub x: i32,
    pub y: i32,
    pub cx: i32,
    pub cy: i32,
    pub flags: u32,
}

#[repr(C)]
pub struct NCCALCSIZE_PARAMS {
    pub rgrc: [RECT; 3],
    pub lppos: *mut WINDOWPOS,
}

#[repr(C)]
pub struct CREATESTRUCTW {
    pub lpCreateParams: *mut c_void,
    pub hInstance: HINSTANCE,
    pub hMenu: HMENU,
    pub hwndParent: HWND,
    pub cy: i32,
    pub cx: i32,
    pub y: i32,
    pub x: i32,
    pub style: i32,
    pub lpszName: *const u16,
    pub lpszClass: *const u16,
    pub dwExStyle: u32,
}

#[repr(C)]
pub struct MARGINS {
    pub cxLeftWidth: i32,
    pub cxRightWidth: i32,
    pub cyTopHeight: i32,
    pub cyBottomHeight: i32,
}

#[repr(C)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[repr(C)]
pub struct TRACKMOUSEEVENT {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub hwndTrack: HWND,
    pub dwHoverTime: u32,
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: LPARAM,
    pub lpfnHook: isize,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[repr(C)]
pub struct RGBQUAD {
    pub rgbBlue: u8,
    pub rgbGreen: u8,
    pub rgbRed: u8,
    pub rgbReserved: u8,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PointF {
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
pub struct GdiplusStartupInput {
    pub GdiplusVersion: u32,
    pub DebugEventCallback: isize,
    pub SuppressBackgroundThread: i32,
    pub SuppressExternalCodecs: i32,
}

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_KILLFOCUS: u32 = 0x0008;
pub const WM_ENABLE: u32 = 0x000A;
pub const WM_SETREDRAW: u32 = 0x000B;
pub const WM_SETTEXT: u32 = 0x000C;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_GETTEXT: u32 = 0x000D;
pub const WM_GETTEXTLENGTH: u32 = 0x000E;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_QUIT: u32 = 0x0012;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETTINGCHANGE: u32 = 0x001A;
pub const WM_SETCURSOR: u32 = 0x0020;
pub const WM_GETMINMAXINFO: u32 = 0x0024;
pub const WM_NCCREATE: u32 = 0x0081;
pub const WM_NCPAINT: u32 = 0x0085;
pub const WM_NCACTIVATE: u32 = 0x0086;
pub const WM_NCCALCSIZE: u32 = 0x0083;
pub const WM_NCHITTEST: u32 = 0x0084;
pub const WM_NCMOUSEMOVE: u32 = 0x00A0;
pub const WM_NCMOUSELEAVE: u32 = 0x02A2;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_KEYUP: u32 = 0x0101;
pub const WM_CHAR: u32 = 0x0102;
pub const WM_SYSKEYDOWN: u32 = 0x0104;
pub const WM_SYSKEYUP: u32 = 0x0105;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_MOUSELEAVE: u32 = 0x02A3;
pub const WM_CAPTURECHANGED: u32 = 0x0215;
pub const WM_DPICHANGED: u32 = 0x02E0;
pub const WM_CTLCOLOREDIT: u32 = 0x0133;
pub const WM_DROPFILES: u32 = 0x0233;
pub const WM_APP: u32 = 0x8000;

pub const HTNOWHERE: LRESULT = 0;
pub const HTCLIENT: LRESULT = 1;
pub const HTCAPTION: LRESULT = 2;
pub const HTREDUCE: LRESULT = 8;
pub const HTZOOM: LRESULT = 9;
pub const HTLEFT: LRESULT = 10;
pub const HTRIGHT: LRESULT = 11;
pub const HTTOP: LRESULT = 12;
pub const HTTOPLEFT: LRESULT = 13;
pub const HTTOPRIGHT: LRESULT = 14;
pub const HTBOTTOM: LRESULT = 15;
pub const HTBOTTOMLEFT: LRESULT = 16;
pub const HTBOTTOMRIGHT: LRESULT = 17;
pub const HTCLOSE: LRESULT = 20;

pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_DBLCLKS: u32 = 0x0008;

pub const WS_OVERLAPPED: u32 = 0x00000000;
pub const WS_POPUP: u32 = 0x80000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CLIPSIBLINGS: u32 = 0x04000000;
pub const WS_CLIPCHILDREN: u32 = 0x02000000;
pub const WS_TABSTOP: u32 = 0x00010000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_THICKFRAME: u32 = 0x00040000;
pub const WS_MINIMIZEBOX: u32 = 0x00020000;
pub const WS_MAXIMIZEBOX: u32 = 0x00010000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;

pub const WS_EX_ACCEPTFILES: u32 = 0x00000010;

pub const ES_LEFT: u32 = 0x0000;
pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_AUTOVSCROLL: u32 = 0x0040;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_NOHIDESEL: u32 = 0x0100;
pub const ES_WANTRETURN: u32 = 0x1000;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOWNOACTIVATE: i32 = 4;
pub const SW_SHOW: i32 = 5;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
#[link(name = "uxtheme")]
extern "system" {
    pub fn SetWindowTheme(hwnd: HWND, sub_app: *const u16, sub_id: *const u16) -> i32;
}

pub const VK_S: i32 = 0x53;
pub const VK_O: i32 = 0x4F;
pub const VK_N: i32 = 0x4E;
pub const WM_CTLCOLORSTATIC: u32 = 0x0138;
pub const SW_MAXIMIZE: i32 = 3;
pub const SW_MINIMIZE: i32 = 6;
pub const SW_RESTORE: i32 = 9;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SWP_FRAMECHANGED: u32 = 0x0020;

pub const GWLP_USERDATA: i32 = -21;
pub const GWLP_WNDPROC: i32 = -4;

pub const SM_CXSIZEFRAME: i32 = 32;
pub const SM_CYSIZEFRAME: i32 = 33;
pub const SM_CXPADDEDBORDER: i32 = 92;
pub const SM_CXICON: i32 = 11;
pub const SM_CXSMICON: i32 = 49;

pub const SPI_GETWORKAREA: u32 = 0x0030;

pub const IMAGE_ICON: u32 = 1;
pub const WM_DISPLAYCHANGE: u32 = 0x007E;
pub const MF_CHECKED: u32 = 0x0008;
pub const LR_DEFAULTSIZE: u32 = 0x0040;
pub const WS_EX_APPWINDOW: u32 = 0x00040000;
pub const LR_DEFAULTCOLOR: u32 = 0x0000;
pub const LR_SHARED: u32 = 0x8000;
pub const IDI_APPLICATION: *const u16 = 32512 as *const u16;
pub const IDC_ARROW: *const u16 = 32512 as *const u16;
pub const IDC_IBEAM: *const u16 = 32513 as *const u16;
pub const IDC_HAND: *const u16 = 32649 as *const u16;
pub const IDC_SIZEWE: *const u16 = 32644 as *const u16;
pub const IDC_SIZENS: *const u16 = 32645 as *const u16;
pub const IDC_SIZENWSE: *const u16 = 32642 as *const u16;
pub const IDC_SIZENESW: *const u16 = 32643 as *const u16;

pub const TME_LEAVE: u32 = 0x00000002;
pub const TME_NONCLIENT: u32 = 0x00000010;
pub const SRCCOPY: u32 = 0x00CC0020;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_RIGHT: u32 = 0x00000002;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_NOPREFIX: u32 = 0x00000800;
pub const DT_END_ELLIPSIS: u32 = 0x00008000;
pub const NULL_BRUSH_STOCK: i32 = 5;
pub const NULL_PEN_STOCK: i32 = 8;
pub const PS_SOLID_STYLE: i32 = 0;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const DEFAULT_CHARSET: u32 = 1;
pub const DI_NORMAL: u32 = 3;
pub const RGN_COPY_MODE: i32 = 5;

pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
pub const DWMWA_BORDER_COLOR: u32 = 34;
pub const DWMWCP_ROUND: i32 = 2;

pub const EM_GETSEL: u32 = 0x00B0;
pub const EM_SETSEL: u32 = 0x00B1;
pub const EM_GETRECT: u32 = 0x00B2;
pub const EM_SCROLLCARET: u32 = 0x00B7;
pub const EM_GETLINECOUNT: u32 = 0x00BA;
pub const EM_LINEINDEX: u32 = 0x00BB;
pub const EM_LINEFROMCHAR: u32 = 0x00C9;
pub const EM_SETLIMITTEXT: u32 = 0x00C5;
pub const EM_REPLACESEL: u32 = 0x00C2;
pub const EM_SETMARGINS: u32 = 0x00D3;
pub const EM_SETCUEBANNER: u32 = 0x1501;
pub const EN_CHANGE: u16 = 0x0300;
pub const EN_SETFOCUS: u16 = 0x0100;
pub const EN_KILLFOCUS: u16 = 0x0200;

pub const MB_OK: u32 = 0x00000000;
pub const MB_YESNOCANCEL: u32 = 0x00000003;
pub const MB_ICONWARNING: u32 = 0x00000030;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;

pub const OFN_OVERWRITEPROMPT: u32 = 0x00000002;
pub const OFN_PATHMUSTEXIST: u32 = 0x00000800;
pub const OFN_FILEMUSTEXIST: u32 = 0x00001000;
pub const OFN_EXPLORER: u32 = 0x00080000;
pub const OFN_NOCHANGEDIR: u32 = 0x00000008;

pub const CF_UNICODETEXT: u32 = 13;
pub const GMEM_MOVEABLE: u32 = 0x0002;

pub const KEY_DOWN: i16 = -32768;

pub const HKEY_CURRENT_USER: HKEY = 0x80000001u64 as isize;
pub const KEY_QUERY_VALUE: u32 = 0x0001;
pub const KEY_SET_VALUE: u32 = 0x0002;
pub const REG_SZ: u32 = 1;
pub const REG_DWORD: u32 = 4;

pub const PIXEL_32BPP_PARGB: i32 = 0x000E200B;
pub const SMOOTHING_ANTIALIAS: i32 = 4;
pub const TEXT_HINT_CLEARTYPE_GRIDFIT: i32 = 5;
pub const PIXEL_OFFSET_HALF: i32 = 4;
pub const INTERP_HIGH_QUALITY_BICUBIC: i32 = 7;
pub const FLUSH_SYNC: i32 = 1;
pub const UNIT_PIXEL: i32 = 2;
pub const FONT_STYLE_REGULAR: i32 = 0;
pub const FONT_STYLE_BOLD: i32 = 1;
pub const FILL_MODE_ALTERNATE: i32 = 0;
pub const WRAP_TILE_FLIP_XY: i32 = 0;
pub const LINE_CAP_ROUND: i32 = 2;
pub const LINE_JOIN_ROUND: i32 = 2;
pub const SF_ALIGN_NEAR: i32 = 0;
pub const SF_ALIGN_CENTER: i32 = 1;
pub const SF_ALIGN_FAR: i32 = 2;
pub const SF_FLAG_NO_WRAP: i32 = 0x1000;
pub const SF_TRIM_ELLIPSIS: i32 = 3;

/// 功能：把 Rust 字符串转成以 0 结尾的 UTF-16
pub fn w(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn RegisterClassExW(c: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(ex: u32, class: *const u16, name: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: HMENU, inst: HINSTANCE, param: *mut c_void) -> HWND;
    pub fn DefWindowProcW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn DestroyWindow(hwnd: HWND) -> i32;
    pub fn GetMessageW(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32;
    pub fn TranslateMessage(msg: *const MSG) -> i32;
    pub fn DispatchMessageW(msg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(code: i32);
    pub fn PostMessageW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> i32;
    pub fn SendMessageW(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn LoadIconW(inst: HINSTANCE, name: *const u16) -> HICON;
    pub fn LoadImageW(inst: HINSTANCE, name: *const u16, kind: u32, cx: i32, cy: i32, flags: u32) -> HANDLE;
    pub fn LoadCursorW(inst: HINSTANCE, name: *const u16) -> HCURSOR;
    pub fn SetCursor(cur: HCURSOR) -> HCURSOR;
    pub fn ShowWindow(hwnd: HWND, cmd: i32) -> i32;
    pub fn UpdateWindow(hwnd: HWND) -> i32;
    pub fn SetWindowPos(hwnd: HWND, after: HWND, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    pub fn GetClientRect(hwnd: HWND, r: *mut RECT) -> i32;
    pub fn GetWindowRect(hwnd: HWND, r: *mut RECT) -> i32;
    pub fn MoveWindow(hwnd: HWND, x: i32, y: i32, cx: i32, cy: i32, repaint: i32) -> i32;
    pub fn InvalidateRect(hwnd: HWND, r: *const RECT, erase: i32) -> i32;
    pub fn BeginPaint(hwnd: HWND, ps: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, ps: *const PAINTSTRUCT) -> i32;
    pub fn GetDC(hwnd: HWND) -> HDC;
    pub fn ReleaseDC(hwnd: HWND, hdc: HDC) -> i32;
    pub fn SetTimer(hwnd: HWND, id: usize, ms: u32, proc_: usize) -> usize;
    pub fn KillTimer(hwnd: HWND, id: usize) -> i32;
    pub fn SetFocus(hwnd: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn GetKeyState(vk: i32) -> i16;
    pub fn GetAsyncKeyState(vk: i32) -> i16;
    pub fn MessageBoxW(hwnd: HWND, text: *const u16, caption: *const u16, flags: u32) -> i32;
    pub fn SystemParametersInfoW(action: u32, param: u32, out: *mut c_void, winini: u32) -> i32;
    pub fn MonitorFromWindow(hwnd: HWND, flags: u32) -> HMONITOR;
    pub fn GetMonitorInfoW(mon: HMONITOR, info: *mut MONITORINFO) -> i32;
    pub fn GetDpiForWindow(hwnd: HWND) -> u32;
    pub fn GetDpiForSystem() -> u32;
    pub fn SetProcessDpiAwarenessContext(ctx: isize) -> i32;
    pub fn GetSystemMetricsForDpi(index: i32, dpi: u32) -> i32;
    pub fn GetSystemMetrics(index: i32) -> i32;
    pub fn AdjustWindowRectExForDpi(r: *mut RECT, style: u32, menu: i32, ex: u32, dpi: u32) -> i32;
    pub fn IsZoomed(hwnd: HWND) -> i32;
    pub fn IsIconic(hwnd: HWND) -> i32;
    pub fn SetForegroundWindow(hwnd: HWND) -> i32;
    pub fn SetWindowTextW(hwnd: HWND, text: *const u16) -> i32;
    pub fn GetWindowTextW(hwnd: HWND, buf: *mut u16, len: i32) -> i32;
    pub fn GetWindowTextLengthW(hwnd: HWND) -> i32;
    pub fn SetWindowLongPtrW(hwnd: HWND, index: i32, value: isize) -> isize;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
    pub fn SetCapture(hwnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> i32;
    pub fn ScreenToClient(hwnd: HWND, pt: *mut POINT) -> i32;
    pub fn ClientToScreen(hwnd: HWND, pt: *mut POINT) -> i32;
    pub fn OpenClipboard(hwnd: HWND) -> i32;
    pub fn EmptyClipboard() -> i32;
    pub fn SetClipboardData(format: u32, mem: HANDLE) -> HANDLE;
    pub fn CloseClipboard() -> i32;
    pub fn IsClipboardFormatAvailable(format: u32) -> i32;
    pub fn GetClipboardData(format: u32) -> HANDLE;
    pub fn TrackMouseEvent(t: *mut TRACKMOUSEEVENT) -> i32;
    pub fn EnableWindow(hwnd: HWND, enable: i32) -> i32;

    pub fn CallWindowProcW(proc_: WNDPROC, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT;
    pub fn FindWindowW(class: *const u16, name: *const u16) -> HWND;
    pub fn ShowWindowAsync(hwnd: HWND, cmd: i32) -> i32;
    pub fn BringWindowToTop(hwnd: HWND) -> i32;
    pub fn GetSysColor(index: i32) -> u32;
    pub fn GetParent(hwnd: HWND) -> HWND;
    pub fn GetDlgItem(hwnd: HWND, id: i32) -> HWND;
    pub fn GetWindowThreadProcessId(hwnd: HWND, pid: *mut u32) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HINSTANCE;
    pub fn GetUserDefaultUILanguage() -> u16;
    pub fn GetUserDefaultLocaleName(buf: *mut u16, len: i32) -> i32;
    pub fn QueryPerformanceCounter(v: *mut i64) -> i32;
    pub fn QueryPerformanceFrequency(v: *mut i64) -> i32;
    pub fn MultiByteToWideChar(cp: u32, flags: u32, s: *const u8, cb: i32, out: *mut u16, cch: i32) -> i32;
    pub fn WideCharToMultiByte(cp: u32, flags: u32, s: *const u16, cch: i32, out: *mut u8, cb: i32, def: *const u8, used: *mut i32) -> i32;
    pub fn GlobalAlloc(flags: u32, size: usize) -> HGLOBAL;
    pub fn GlobalLock(mem: HGLOBAL) -> *mut c_void;
    pub fn GlobalUnlock(mem: HGLOBAL) -> i32;
    pub fn GetTickCount64() -> u64;
    pub fn GetLastError() -> u32;
    pub fn CreateMutexW(sa: *mut c_void, owner: i32, name: *const u16) -> HANDLE;
    pub fn CloseHandle(h: HANDLE) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateDIBSection(hdc: HDC, bi: *const BITMAPINFO, usage: u32, bits: *mut *mut c_void, section: HANDLE, offset: u32) -> HBITMAP;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteDC(hdc: HDC) -> i32;
    pub fn DeleteObject(obj: HGDIOBJ) -> i32;
    pub fn BitBlt(dst: HDC, x: i32, y: i32, w: i32, h: i32, src: HDC, sx: i32, sy: i32, rop: u32) -> i32;
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn CreateFontW(h: i32, w: i32, esc: i32, orient: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, out_prec: u32, clip_prec: u32, quality: u32, pitch: u32, face: *const u16) -> HFONT;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetBkColor(hdc: HDC, color: u32) -> u32;
    pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
    pub fn CreateCompatibleBitmap(hdc: HDC, w: i32, h: i32) -> HBITMAP;
    pub fn CreatePen(style: i32, width: i32, color: u32) -> HPEN;
    pub fn RoundRect(hdc: HDC, l: i32, t: i32, r: i32, b: i32, ew: i32, eh: i32) -> i32;
    pub fn Ellipse(hdc: HDC, l: i32, t: i32, r: i32, b: i32) -> i32;
    pub fn Rectangle(hdc: HDC, l: i32, t: i32, r: i32, b: i32) -> i32;
    pub fn MoveToEx(hdc: HDC, x: i32, y: i32, old: *mut i32) -> i32;
    pub fn LineTo(hdc: HDC, x: i32, y: i32) -> i32;
    pub fn DrawIconEx(hdc: HDC, x: i32, y: i32, icon: HICON, w: i32, h: i32, step: u32, brush: HBRUSH, flags: u32) -> i32;
    pub fn SaveDC(hdc: HDC) -> i32;
    pub fn RestoreDC(hdc: HDC, level: i32) -> i32;
    pub fn IntersectClipRect(hdc: HDC, l: i32, t: i32, r: i32, b: i32) -> i32;
    pub fn BeginPath(hdc: HDC) -> i32;
    pub fn EndPath(hdc: HDC) -> i32;
    pub fn SelectClipPath(hdc: HDC, mode: i32) -> i32;
    pub fn GetTextExtentPoint32W(hdc: HDC, s: *const u16, len: i32, size: *mut SIZE) -> i32;
    pub fn DrawTextW(hdc: HDC, s: *const u16, len: i32, rect: *mut RECT, flags: u32) -> i32;
    pub fn DestroyIcon(icon: HICON) -> i32;
    pub fn GetStockObject(obj: i32) -> HGDIOBJ;
}

#[link(name = "gdiplus")]
unsafe extern "system" {
    pub fn GdiplusStartup(token: *mut usize, input: *const GdiplusStartupInput, output: *mut c_void) -> i32;
    pub fn GdiplusShutdown(token: usize);
    pub fn GdipCreateBitmapFromScan0(w: i32, h: i32, stride: i32, format: i32, scan0: *mut u8, out: *mut isize) -> i32;
    pub fn GdipCreateBitmapFromHICON(icon: HICON, out: *mut isize) -> i32;
    pub fn GdipDisposeImage(img: isize) -> i32;
    pub fn GdipGetImageGraphicsContext(img: isize, out: *mut isize) -> i32;
    pub fn GdipDeleteGraphics(g: isize) -> i32;
    pub fn GdipFlush(g: isize, intent: i32) -> i32;
    pub fn GdipGraphicsClear(g: isize, color: u32) -> i32;
    pub fn GdipSetSmoothingMode(g: isize, mode: i32) -> i32;
    pub fn GdipResetWorldTransform(g: isize) -> i32;
    pub fn GdipScaleWorldTransform(g: isize, sx: f32, sy: f32, order: i32) -> i32;
    pub fn GdipSetTextRenderingHint(g: isize, hint: i32) -> i32;
    pub fn GdipSetPixelOffsetMode(g: isize, mode: i32) -> i32;
    pub fn GdipSetInterpolationMode(g: isize, mode: i32) -> i32;
    pub fn GdipCreateSolidFill(color: u32, out: *mut isize) -> i32;
    pub fn GdipDeleteBrush(b: isize) -> i32;
    pub fn GdipFillRectangle(g: isize, brush: isize, x: f32, y: f32, w: f32, h: f32) -> i32;
    pub fn GdipDrawImageRectI(g: isize, img: isize, x: i32, y: i32, w: i32, h: i32) -> i32;
    pub fn GdipCreatePath(mode: i32, out: *mut isize) -> i32;
    pub fn GdipDeletePath(p: isize) -> i32;
    pub fn GdipAddPathArc(p: isize, x: f32, y: f32, w: f32, h: f32, start: f32, sweep: f32) -> i32;
    pub fn GdipAddPathLine(p: isize, x1: f32, y1: f32, x2: f32, y2: f32) -> i32;
    pub fn GdipAddPathRectangle(p: isize, x: f32, y: f32, w: f32, h: f32) -> i32;
    pub fn GdipAddPathEllipse(p: isize, x: f32, y: f32, w: f32, h: f32) -> i32;
    pub fn GdipClosePathFigure(p: isize) -> i32;
    pub fn GdipFillPath(g: isize, brush: isize, path: isize) -> i32;
    pub fn GdipDrawPath(g: isize, pen: isize, path: isize) -> i32;
    pub fn GdipCreatePen1(color: u32, width: f32, unit: i32, out: *mut isize) -> i32;
    pub fn GdipSetPenStartCap(p: isize, cap: i32) -> i32;
    pub fn GdipSetPenEndCap(p: isize, cap: i32) -> i32;
    pub fn GdipSetPenLineJoin(p: isize, join: i32) -> i32;
    pub fn GdipDeletePen(p: isize) -> i32;
    pub fn GdipCreateLineBrush(p1: *const PointF, p2: *const PointF, c1: u32, c2: u32, wrap: i32, out: *mut isize) -> i32;
    pub fn GdipCreateFontFamilyFromName(name: *const u16, collection: isize, out: *mut isize) -> i32;
    pub fn GdipDeleteFontFamily(f: isize) -> i32;
    pub fn GdipCreateFont(family: isize, size: f32, style: i32, unit: i32, out: *mut isize) -> i32;
    pub fn GdipDeleteFont(f: isize) -> i32;
    pub fn GdipGetFontHeight(font: isize, g: isize, out: *mut f32) -> i32;
    pub fn GdipCreateStringFormat(flags: i32, lang: u16, out: *mut isize) -> i32;
    pub fn GdipDeleteStringFormat(f: isize) -> i32;
    pub fn GdipSetStringFormatAlign(f: isize, align: i32) -> i32;
    pub fn GdipSetStringFormatLineAlign(f: isize, align: i32) -> i32;
    pub fn GdipSetStringFormatTrimming(f: isize, trim: i32) -> i32;
    pub fn GdipSetStringFormatHotkeyPrefix(f: isize, prefix: i32) -> i32;
    pub fn GdipDrawString(g: isize, s: *const u16, len: i32, font: isize, layout: *const f32, sf: isize, brush: isize) -> i32;
    pub fn GdipMeasureString(g: isize, s: *const u16, len: i32, font: isize, layout: *const f32, sf: isize, bounds: *mut f32, fitted: *mut i32, lines: *mut i32) -> i32;
    pub fn GdipSetClipRect(g: isize, x: f32, y: f32, w: f32, h: f32, combine: i32) -> i32;
    pub fn GdipResetClip(g: isize) -> i32;
}

#[link(name = "dwmapi")]
unsafe extern "system" {
    pub fn DwmSetWindowAttribute(hwnd: HWND, attr: u32, value: *const c_void, size: u32) -> i32;
    pub fn DwmExtendFrameIntoClientArea(hwnd: HWND, margins: *const MARGINS) -> i32;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    pub fn GetSaveFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
    pub fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn DragAcceptFiles(hwnd: HWND, accept: i32);
    pub fn DragQueryFileW(drop: HANDLE, index: u32, buf: *mut u16, len: u32) -> u32;
    pub fn DragFinish(drop: HANDLE);
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn RegOpenKeyExW(key: HKEY, sub: *const u16, opt: u32, access: u32, out: *mut HKEY) -> i32;
    pub fn RegQueryValueExW(key: HKEY, name: *const u16, reserved: *mut u32, kind: *mut u32, data: *mut u8, size: *mut u32) -> i32;
    pub fn RegSetValueExW(key: HKEY, name: *const u16, reserved: u32, kind: u32, data: *const u8, size: u32) -> i32;
    pub fn RegCreateKeyExW(key: HKEY, sub: *const u16, reserved: u32, class: *const u16, opt: u32, access: u32, sa: *mut c_void, out: *mut HKEY, disp: *mut u32) -> i32;
    pub fn RegCloseKey(key: HKEY) -> i32;
}
