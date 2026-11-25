// 功能：窗口过程、托盘菜单、全局钩子与消息循环
#![allow(non_snake_case, static_mut_refs, dead_code)]
#![windows_subsystem = "windows"]

mod anim;
mod app;
mod config;
mod ffi;
mod hotkey;
mod i18n;
mod render;
mod sys;
mod theme;

use app::App;
use ffi::*;
use sys::*;

const CLASS_NAME: &str = "TextDocCreatorSettings";
const WM_TRAY: u32 = 0x0400 + 100;
const ID_TRAY_SETTINGS: usize = 1001;
const ID_TRAY_EXIT: usize = 1002;
const ID_TRAY_AUTOSTART: usize = 1003;
const LOGICAL_W: f32 = 1060.0;
const LOGICAL_H: f32 = 620.0;
const MIN_W: f32 = 820.0;
const MIN_H: f32 = 520.0;

static mut QUIT: bool = false;

fn main() {
    unsafe {
        run();
    }
}

unsafe fn run() {
    SetProcessDpiAwarenessContext(-4);
    let instance = GetModuleHandleW(core::ptr::null());
    let mutex_name = w("Local\\TextDocumentCreatorSettings");
    let mutex = CreateMutexW(core::ptr::null_mut(), 0, mutex_name.as_ptr());
    if GetLastError() == 183 {
        let class = w(CLASS_NAME);
        let existing = FindWindowW(class.as_ptr(), core::ptr::null());
        if existing != 0 {
            ShowWindowAsync(existing, SW_RESTORE);
            SetForegroundWindow(existing);
        }
        if mutex != 0 {
            CloseHandle(mutex);
        }
        return;
    }
    config::migrate_autostart();
    let class = w(CLASS_NAME);
    let icon = LoadImageW(instance, 1 as *const u16, IMAGE_ICON, 0, 0, LR_DEFAULTSIZE | LR_SHARED);
    let small = LoadImageW(instance, 1 as *const u16, IMAGE_ICON, 16, 16, LR_SHARED);
    let cursor = LoadCursorW(0, IDC_ARROW);
    let wc = WNDCLASSEXW {
        cbSize: core::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: icon,
        hCursor: cursor,
        hbrBackground: 0,
        lpszMenuName: core::ptr::null(),
        lpszClassName: class.as_ptr(),
        hIconSm: small,
    };
    if RegisterClassExW(&wc) == 0 {
        return;
    }
    let (width, height, x, y) = initial_rect();
    let style = WS_POPUP | WS_THICKFRAME | WS_CLIPCHILDREN;
    let title = w("设置");
    let hwnd = CreateWindowExW(WS_EX_APPWINDOW, class.as_ptr(), title.as_ptr(), style, x, y, width, height, 0, 0, instance, core::ptr::null_mut());
    if hwnd == 0 {
        return;
    }
    apply_fit(hwnd);
    let corner = DWMWCP_ROUND;
    DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &corner as *const i32 as *const core::ffi::c_void, 4);
    ShowWindow(hwnd, SW_HIDE);
    SetTimer(hwnd, 2, 900, 0);
    let mut message: MSG = core::mem::zeroed();
    while GetMessageW(&mut message, 0, 0, 0) > 0 {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }
}

unsafe fn app_of(hwnd: HWND) -> Option<&'static mut App> {
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
    if pointer.is_null() {
        None
    } else {
        Some(&mut *pointer)
    }
}

unsafe fn scale_of(hwnd: HWND) -> f32 {
    let dpi = GetDpiForWindow(hwnd);
    (if dpi == 0 { 96 } else { dpi }) as f32 / 96.0
}

unsafe fn hit_button(hwnd: HWND, x: i32, y: i32) -> bool {
    let s = scale_of(hwnd);
    let mut rect: RECT = core::mem::zeroed();
    GetWindowRect(hwnd, &mut rect);
    let logical_w = (rect.right - rect.left) as f32 / s;
    let logical_x = (x - rect.left) as f32 / s;
    let logical_y = (y - rect.top) as f32 / s;
    logical_y < 44.0 && logical_x >= logical_w - 96.0
}

unsafe fn region_test(hwnd: HWND, lp: LPARAM) -> LRESULT {
    let s = scale_of(hwnd);
    let border = (8.0 * s).round() as i32;
    let corner = (border as f32 * 1.7).round() as i32;
    let mut rect: RECT = core::mem::zeroed();
    GetWindowRect(hwnd, &mut rect);
    let x = (lp & 0xFFFF) as i16 as i32;
    let y = ((lp >> 16) & 0xFFFF) as i16 as i32;
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    let left = x - rect.left;
    let top = y - rect.top;
    if IsZoomed(hwnd) == 0 {
        let right = width - left;
        let bottom = height - top;
        if left < border && top < corner {
            return HTTOPLEFT;
        }
        if right < border && top < corner {
            return HTTOPRIGHT;
        }
        if left < border && bottom < corner {
            return HTBOTTOMLEFT;
        }
        if right < border && bottom < corner {
            return HTBOTTOMRIGHT;
        }
        if left < border {
            return HTLEFT;
        }
        if right < border {
            return HTRIGHT;
        }
        if top < border {
            return HTTOP;
        }
        if bottom < border {
            return HTBOTTOM;
        }
    }
    if hit_button(hwnd, x, y) {
        return HTCLIENT;
    }
    if (top as f32) < 44.0 * s {
        return HTCAPTION;
    }
    HTCLIENT
}

/// 功能：按屏幕缩放与工作区限制求解物理窗口尺寸
unsafe fn fit_size(work_w: i32, work_h: i32, s: f32) -> (i32, i32) {
    let mut width = (LOGICAL_W * s).round() as i32;
    let mut height = (LOGICAL_H * s).round() as i32;
    let max_w = (work_w - (32.0 * s).round() as i32).max(480);
    let max_h = (work_h - (32.0 * s).round() as i32).max(360);
    if width > max_w {
        width = max_w;
    }
    if height > max_h {
        height = max_h;
    }
    let min_w = ((MIN_W * s).round() as i32).min(max_w);
    let min_h = ((MIN_H * s).round() as i32).min(max_h);
    if width < min_w {
        width = min_w;
    }
    if height < min_h {
        height = min_h;
    }
    (width, height)
}

/// 功能：计算启动时的窗口位置与尺寸
unsafe fn initial_rect() -> (i32, i32, i32, i32) {
    let dpi = GetDpiForSystem();
    let s = (if dpi == 0 { 96 } else { dpi }) as f32 / 96.0;
    let mut work: RECT = core::mem::zeroed();
    SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut work as *mut RECT as *mut core::ffi::c_void, 0);
    let (width, height) = fit_size(work.right - work.left, work.bottom - work.top, s);
    let x = work.left + ((work.right - work.left) - width) / 2;
    let y = work.top + ((work.bottom - work.top) - height) / 2;
    (width, height, x, y)
}

/// 功能：按当前显示器缩放与工作区重建窗口尺寸
unsafe fn apply_fit(hwnd: HWND) {
    let dpi = GetDpiForWindow(hwnd);
    let s = (if dpi == 0 { 96 } else { dpi }) as f32 / 96.0;
    let mut work: RECT = core::mem::zeroed();
    SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut work as *mut RECT as *mut core::ffi::c_void, 0);
    let (width, height) = fit_size(work.right - work.left, work.bottom - work.top, s);
    let mut current: RECT = core::mem::zeroed();
    GetWindowRect(hwnd, &mut current);
    let mut x = (current.left + current.right) / 2 - width / 2;
    let mut y = (current.top + current.bottom) / 2 - height / 2;
    if x < work.left {
        x = work.left;
    }
    if y < work.top {
        y = work.top;
    }
    if x + width > work.right {
        x = work.right - width;
    }
    if y + height > work.bottom {
        y = work.bottom - height;
    }
    SetWindowPos(hwnd, 0, x, y, width, height, SWP_NOZORDER | SWP_NOACTIVATE);
    InvalidateRect(hwnd, core::ptr::null(), 0);
}

unsafe fn trim_memory() {
    SetProcessWorkingSetSize(GetCurrentProcess(), -1, -1);
}

unsafe fn show_window(hwnd: HWND) {
    ShowWindow(hwnd, SW_SHOW);
    ShowWindowAsync(hwnd, SW_RESTORE);
    SetForegroundWindow(hwnd);
    InvalidateRect(hwnd, core::ptr::null(), 0);
}

unsafe fn tray_menu(hwnd: HWND) {
    let settings_label = match app_of(hwnd) {
        Some(state) => state.t().menu_settings,
        None => "设置",
    };
    let exit_label = match app_of(hwnd) {
        Some(state) => state.t().menu_exit,
        None => "退出",
    };
    let autostart_label = match app_of(hwnd) {
        Some(state) => state.t().menu_autostart,
        None => "开机自启动",
    };
    let autostart_on = match app_of(hwnd) {
        Some(state) => state.settings.autostart,
        None => false,
    };
    let settings_text = w(settings_label);
    let exit_text = w(exit_label);
    let autostart_text = w(autostart_label);
    let autostart_flags = MF_STRING | if autostart_on { MF_CHECKED } else { 0 };
    let menu = CreatePopupMenu();
    AppendMenuW(menu, MF_STRING, ID_TRAY_SETTINGS, settings_text.as_ptr());
    AppendMenuW(menu, autostart_flags, ID_TRAY_AUTOSTART, autostart_text.as_ptr());
    AppendMenuW(menu, MF_SEPARATOR, 0, core::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT, exit_text.as_ptr());
    let mut point: POINT = core::mem::zeroed();
    GetCursorPos(&mut point);
    SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, core::ptr::null());
    DestroyMenu(menu);
    PostMessageW(hwnd, WM_NULL_MSG, 0, 0);
    if command == ID_TRAY_SETTINGS as i32 {
        show_window(hwnd);
    } else if command == ID_TRAY_AUTOSTART as i32 {
        if let Some(state) = app_of(hwnd) {
            state.settings.autostart = !state.settings.autostart;
            state.settings.save();
            state.settings.apply_autostart();
            state.sync_toggles();
        }
    } else if command == ID_TRAY_EXIT as i32 {
        QUIT = true;
        DestroyWindow(hwnd);
    }
}

unsafe fn add_tray(hwnd: HWND, instance: HINSTANCE) {
    let mut data: NOTIFYICONDATAW = core::mem::zeroed();
    data.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = LoadImageW(instance, 1 as *const u16, IMAGE_ICON, 0, 0, LR_DEFAULTSIZE | LR_SHARED);
    let tip: Vec<u16> = "Text Document Creator".encode_utf16().collect();
    data.szTip[..tip.len().min(127)].copy_from_slice(&tip[..tip.len().min(127)]);
    Shell_NotifyIconW(NIM_ADD, &mut data);
}

unsafe fn remove_tray(hwnd: HWND) {
    let mut data: NOTIFYICONDATAW = core::mem::zeroed();
    data.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    Shell_NotifyIconW(NIM_DELETE, &mut data);
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let instance = GetModuleHandleW(core::ptr::null());
            let state = Box::into_raw(Box::new(App::new(hwnd, instance)));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);
            add_tray(hwnd, instance);
            hotkey::install(hwnd);
            if let Some(state) = app_of(hwnd) {
                hotkey::update(state.settings);
                let title = w(state.t().title);
                SetWindowTextW(hwnd, title.as_ptr());
            }
            return 1;
        }
        WM_NCCALCSIZE => {
            if wp != 0 {
                let params = lp as *mut NCCALCSIZE_PARAMS;
                if IsZoomed(hwnd) != 0 {
                    let dpi = GetDpiForWindow(hwnd);
                    let frame = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi) + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
                    (*params).rgrc[0].left += frame;
                    (*params).rgrc[0].right -= frame;
                    (*params).rgrc[0].top += frame;
                    (*params).rgrc[0].bottom -= frame;
                }
                return 0;
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_NCACTIVATE => {
            if let Some(state) = app_of(hwnd) {
                state.sync_frame();
                state.invalidate();
            }
            1
        }
        WM_SETCURSOR => {
            if lp as u32 & 0xFFFF == HTCLIENT as u32 {
                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                ScreenToClient(hwnd, &mut pt);
                if let Some(state) = app_of(hwnd) {
                    if state.about_ibeam(pt.x as f32, pt.y as f32) {
                        SetCursor(LoadCursorW(0, IDC_IBEAM));
                        return 1;
                    }
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_NCHITTEST => region_test(hwnd, lp),
        WM_GETMINMAXINFO => {
            let s = scale_of(hwnd);
            let info = lp as *mut MINMAXINFO;
            (*info).ptMinTrackSize.x = (MIN_W * s) as i32;
            (*info).ptMinTrackSize.y = (MIN_H * s) as i32;
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if let Some(state) = app_of(hwnd) {
                state.paint();
            }
            EndPaint(hwnd, &ps);
            let _ = hdc;
            0
        }
        WM_SIZE => {
            let width = (lp & 0xFFFF) as i32;
            let height = ((lp >> 16) & 0xFFFF) as i32;
            if let Some(state) = app_of(hwnd) {
                state.resize(width, height);
            }
            InvalidateRect(hwnd, core::ptr::null(), 0);
            0
        }
        WM_DPICHANGED => {
            let dpi = (wp & 0xFFFF) as u32;
            if let Some(state) = app_of(hwnd) {
                state.update_dpi(dpi);
            }
            apply_fit(hwnd);
            0
        }
        WM_DISPLAYCHANGE => {
            apply_fit(hwnd);
            0
        }
        WM_TIMER => {
            let mut alive = false;
            if let Some(state) = app_of(hwnd) {
                alive = state.on_timer();
            }
            if !alive {
                KillTimer(hwnd, 1);
                trim_memory();
            }
            0
        }
        WM_MOUSEMOVE => {
            if let Some(state) = app_of(hwnd) {
                let mut track = TRACKMOUSEEVENT {
                    cbSize: core::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                    dwFlags: TME_LEAVE,
                    hwndTrack: hwnd,
                    dwHoverTime: 0,
                };
                TrackMouseEvent(&mut track);
                let x = (lp & 0xFFFF) as i16 as f32;
                let y = ((lp >> 16) & 0xFFFF) as i16 as f32;
                state.on_mouse_move(x, y);
            }
            0
        }
        WM_MOUSELEAVE => {
            if let Some(state) = app_of(hwnd) {
                state.on_leave();
            }
            0
        }
        WM_LBUTTONDOWN => {
            if let Some(state) = app_of(hwnd) {
                let x = (lp & 0xFFFF) as i16 as f32;
                let y = ((lp >> 16) & 0xFFFF) as i16 as f32;
                SetFocus(hwnd);
                state.on_down(x, y);
            }
            0
        }
        WM_LBUTTONDBLCLK => {
            if let Some(state) = app_of(hwnd) {
                let x = (lp & 0xFFFF) as i16 as f32;
                let y = ((lp >> 16) & 0xFFFF) as i16 as f32;
                SetFocus(hwnd);
                state.on_dblclick(x, y);
            }
            0
        }
        WM_CAPTURECHANGED => {
            if let Some(state) = app_of(hwnd) {
                state.cancel_drag();
            }
            0
        }
        WM_LBUTTONUP => {
            if let Some(state) = app_of(hwnd) {
                let x = (lp & 0xFFFF) as i16 as f32;
                let y = ((lp >> 16) & 0xFFFF) as i16 as f32;
                state.on_up(x, y);
            }
            0
        }
        WM_MOUSEWHEEL => {
            if let Some(state) = app_of(hwnd) {
                let sx = (lp & 0xFFFF) as i16 as f32;
                let sy = ((lp >> 16) & 0xFFFF) as i16 as f32;
                state.on_wheel(((wp >> 16) & 0xFFFF) as i16, sx, sy);
            }
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            if let Some(state) = app_of(hwnd) {
                if state.on_key(wp as u32, true) {
                    return 0;
                }
            }
            DefWindowProcW(hwnd, msg, wp, lp)
        }
        WM_SETTINGCHANGE => {
            if let Some(state) = app_of(hwnd) {
                state.refresh_theme();
            }
            0
        }
        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            SetProcessWorkingSetSize(GetCurrentProcess(), -1, -1);
            0
        }
        WM_TRAY => {
            let action = lp as u32 & 0xFFFF;
            if action == WM_RBUTTONUP_W || action == WM_CONTEXTMENU {
                tray_menu(hwnd);
            } else if action == WM_LBUTTONUP_W || action == WM_LBUTTONDBLCLK_W {
                show_window(hwnd);
            }
            0
        }
        hotkey::MSG_CREATE_FILE => {
            let lang = match app_of(hwnd) {
                Some(state) => state.lang,
                None => 0,
            };
            std::thread::spawn(move || unsafe {
                let _ = hotkey::create_text_file(lang);
            });
            0
        }
        hotkey::MSG_OPEN_NOTEPAD => {
            std::thread::spawn(|| unsafe {
                let _ = hotkey::open_notepad();
            });
            0
        }
        hotkey::MSG_TOGGLE_CREATE | hotkey::MSG_TOGGLE_NOTEPAD => {
            if let Some(state) = app_of(hwnd) {
                if msg == hotkey::MSG_TOGGLE_CREATE {
                    state.settings.enable_create = !state.settings.enable_create;
                } else {
                    state.settings.enable_notepad = !state.settings.enable_notepad;
                }
                state.settings.save();
                hotkey::update(state.settings);
                state.sync_toggles();
            }
            0
        }
        hotkey::MSG_CAPTURED => {
            if let Some(state) = app_of(hwnd) {
                state.take_capture();
            }
            0
        }
        WM_DESTROY => {
            remove_tray(hwnd);
            hotkey::uninstall();
            let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut App;
            if !pointer.is_null() {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let mut state = Box::from_raw(pointer);
                state.dispose();
            }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
}
