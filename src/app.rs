/// 功能：设置窗口的布局、绘制与交互状态
use crate::anim::{Anim, clock};
use crate::config::*;
use crate::ffi::*;
use crate::hotkey;
use crate::i18n::{self, Lang};
use crate::render::{Align, Canvas, Gfx, icon_image};
use crate::sys;
use crate::theme::*;

pub const ID_MIN: u32 = 1;
pub const ID_CLOSE: u32 = 2;
pub const ID_LANG: u32 = 3;
pub const ID_ABOUT: u32 = 4;
pub const ID_TOGGLE: u32 = 10;
pub const ID_KEY: u32 = 20;
pub const ID_CLEAR: u32 = 30;
pub const ID_RESET: u32 = 40;
pub const ID_RESET_ALL: u32 = 50;
pub const ID_LANG_ITEM: u32 = 60;
pub const ID_THEME_ITEM: u32 = 80;
const IDS: usize = 96;

#[derive(Clone, Copy, Default)]
pub struct RectF {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl RectF {
    pub fn has(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }

    fn cx(&self) -> f32 {
        self.x + self.w * 0.5
    }

    fn cy(&self) -> f32 {
        self.y + self.h * 0.5
    }
}

pub struct Layout {
    pub s: f32,
    pub w: f32,
    pub h: f32,
    pub min: RectF,
    pub close: RectF,
    pub lang: RectF,
    pub about: RectF,
    pub theme_ctl: RectF,
    pub card_left: RectF,
    pub card_right: RectF,
    pub toggles: [RectF; 3],
    pub fields: [RectF; 4],
    pub clears: [RectF; 4],
    pub resets: [RectF; 4],
    pub reset_all: RectF,
    pub card_about: RectF,
    pub list: RectF,
    pub list_max_scroll: f32,
}

pub struct App {
    pub hwnd: HWND,
    pub inst: HINSTANCE,
    pub settings: Settings,
    pub lang: usize,
    pub dpi: u32,
    pub w: i32,
    pub h: i32,
    canvas: Option<Canvas>,
    logo: isize,
    theme: Theme,
    theme_from: Theme,
    theme_to: Theme,
    theme_anim: Anim,
    hover: u32,
    press: u32,
    anims: Vec<Anim>,
    toggles: [Anim; 3],
    dropdown_anim: Anim,
    lang_scroll: Anim,
    lang_scroll_target: f32,
    dropdown_open: bool,
    lang_focus: bool,
    about_mode: bool,
    scroll: f32,
    scroll_target: f32,
    max_scroll: f32,
    capture: Option<usize>,
    capture_anim: Anim,
}

impl App {
    pub unsafe fn new(hwnd: HWND, inst: HINSTANCE) -> App {
        let settings = Settings::load();
        if settings.language_auto {
            settings.save();
        }
        let mut app = App {
            hwnd,
            inst,
            lang: settings.language,
            settings,
            dpi: 96,
            w: 0,
            h: 0,
            canvas: None,
            logo: 0,
            theme: Theme::light(),
            theme_from: Theme::light(),
            theme_to: Theme::light(),
            theme_anim: Anim::fast(1.0, 30.0),
            hover: 0,
            press: 0,
            anims: vec![Anim::new(0.0); IDS],
            toggles: [Anim::new(0.0); 3],
            dropdown_anim: Anim::fast(0.0, 46.0),
            lang_scroll: Anim::fast(0.0, 26.0),
            lang_scroll_target: 0.0,
            dropdown_open: false,
            lang_focus: false,
            about_mode: false,
            scroll: 0.0,
            scroll_target: 0.0,
            max_scroll: 0.0,
            capture: None,
            capture_anim: Anim::new(0.0),
        };
        app.dpi = GetDpiForWindow(hwnd);
        if app.dpi == 0 {
            app.dpi = 96;
        }
        app.theme = app.preferred_theme();
        app.theme_from = app.theme;
        app.theme_to = app.theme;
        app.logo = icon_image(inst, (32.0 * app.scale()) as i32);
        app.toggles[0].snap(if app.settings.enable_create { 1.0 } else { 0.0 });
        app.toggles[1].snap(if app.settings.enable_notepad { 1.0 } else { 0.0 });
        app.toggles[2].snap(if app.settings.autostart { 1.0 } else { 0.0 });
        app
    }

    pub fn scale(&self) -> f32 {
        self.dpi as f32 / 96.0
    }

    pub fn t(&self) -> &'static Lang {
        i18n::lang(self.lang)
    }

    pub unsafe fn dispose(&mut self) {
        if let Some(canvas) = self.canvas.as_mut() {
            canvas.free();
        }
        self.canvas = None;
        if self.logo != 0 {
            DestroyIcon(self.logo);
            self.logo = 0;
        }
    }

    pub fn preferred_theme(&self) -> Theme {
        match self.settings.theme_mode {
            1 => Theme::light(),
            2 => Theme::dark(),
            _ => if system_prefers_dark() { Theme::dark() } else { Theme::light() },
        }
    }

    pub unsafe fn set_theme_mode(&mut self, mode: u32) {
        if self.settings.theme_mode == mode {
            return;
        }
        self.settings.theme_mode = mode;
        self.settings.save();
        self.start_theme_fade();
    }

    pub unsafe fn refresh_theme(&mut self) {
        if self.settings.theme_mode != 0 {
            return;
        }
        self.start_theme_fade();
    }

    unsafe fn start_theme_fade(&mut self) {
        let to = self.preferred_theme();
        if to.bg == self.theme_to.bg && to.dark == self.theme_to.dark {
            return;
        }
        self.theme_from = self.theme;
        self.theme_to = to;
        self.theme_anim = Anim::fast(0.0, 30.0);
        self.theme_anim.to(1.0);
        self.sync_frame();
        self.request_anim();
        self.invalidate();
    }

    pub unsafe fn sync_frame(&self) {
        let c = self.theme.bg;
        let border: u32 = ((c & 0x0000FF) << 16) | (c & 0x00FF00) | ((c >> 16) & 0x0000FF);
        DwmSetWindowAttribute(self.hwnd, DWMWA_BORDER_COLOR, &border as *const u32 as *const core::ffi::c_void, 4);
        let dark: i32 = if self.theme_to.dark { 1 } else { 0 };
        DwmSetWindowAttribute(self.hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, &dark as *const i32 as *const core::ffi::c_void, 4);
    }

    pub fn invalidate(&self) {
        unsafe {
            InvalidateRect(self.hwnd, core::ptr::null(), 0);
        }
    }

    pub fn resize(&mut self, w: i32, h: i32) {
        if self.canvas.is_some() && w == self.w && h == self.h {
            return;
        }
        self.w = w;
        self.h = h;
        unsafe {
            if let Some(canvas) = self.canvas.as_mut() {
                canvas.free();
            }
            self.canvas = Canvas::new(self.hwnd, w, h);
        }
    }

    pub unsafe fn update_dpi(&mut self, dpi: u32) {
        self.dpi = dpi;
        if self.logo != 0 {
            DestroyIcon(self.logo);
        }
        self.logo = icon_image(self.inst, (32.0 * self.scale()) as i32);
        let mut rect: RECT = core::mem::zeroed();
        GetClientRect(self.hwnd, &mut rect);
        self.resize(rect.right, rect.bottom);
    }

    pub fn layout(&self) -> Layout {
        let s = self.scale();
        let w = self.w as f32 / s;
        let h = self.h as f32 / s;
        let pad = 20.0f32;
        let title_h = 44.0f32;
        let header_y = title_h + 12.0;
        let content_top = header_y + 50.0;
        let left_w = (w * 0.30).clamp(244.0, 330.0);
        let card_h = (h - content_top - pad).max(120.0);
        let btn = 34.0f32;
        let mut layout = Layout {
            s,
            w,
            h,
            min: RectF { x: w - 96.0, y: 0.0, w: 46.0, h: title_h },
            close: RectF { x: w - 50.0, y: 0.0, w: 46.0, h: title_h },
            lang: RectF { x: pad + 90.0, y: header_y, w: 200.0, h: 36.0 },
            about: RectF { x: w - pad - 112.0, y: header_y, w: 112.0, h: 36.0 },
            theme_ctl: RectF { x: w - pad - 112.0 - 10.0 - 192.0, y: header_y, w: 192.0, h: 36.0 },
            card_left: RectF { x: pad, y: content_top, w: left_w, h: card_h },
            card_right: RectF { x: pad + left_w + 14.0, y: content_top, w: w - pad * 2.0 - left_w - 14.0, h: card_h },
            toggles: [RectF::default(); 3],
            fields: [RectF::default(); 4],
            clears: [RectF::default(); 4],
            resets: [RectF::default(); 4],
            reset_all: RectF::default(),
            card_about: RectF { x: pad, y: content_top, w: w - pad * 2.0, h: card_h },
            list: RectF::default(),
            list_max_scroll: 0.0,
        };
        for i in 0..3 {
            let row_y = layout.card_left.y + 54.0 + i as f32 * 54.0;
            layout.toggles[i] = RectF { x: layout.card_left.x + layout.card_left.w - 22.0 - 46.0, y: row_y + 14.0, w: 46.0, h: 26.0 };
        }
        let card = layout.card_right;
        let reset_x = card.x + card.w - 22.0 - 100.0;
        let clear_x = reset_x - 8.0 - 100.0;
        for i in 0..4 {
            let row_y = card.y + 54.0 + i as f32 * 54.0;
            let field_x = card.x + 194.0;
            let field_w = (clear_x - 12.0 - field_x).max(90.0);
            layout.fields[i] = RectF { x: field_x, y: row_y + (54.0 - 36.0) * 0.5, w: field_w, h: 36.0 };
            layout.clears[i] = RectF { x: clear_x, y: row_y + (54.0 - btn) * 0.5, w: 100.0, h: btn };
            layout.resets[i] = RectF { x: reset_x, y: row_y + (54.0 - btn) * 0.5, w: 100.0, h: btn };
        }
        layout.reset_all = RectF { x: card.x + card.w - 22.0 - 140.0, y: card.y + card.h - 20.0 - 36.0, w: 140.0, h: 36.0 };
        let list_top = layout.lang.y + 40.0;
        let avail = (layout.h - list_top - 12.0).max(120.0);
        let full = i18n::COUNT as f32 * 40.0 + 16.0;
        let panel = if full < avail { full } else { avail };
        layout.list = RectF { x: layout.lang.x, y: list_top, w: layout.lang.w, h: panel };
        layout.list_max_scroll = if full > panel { full - panel } else { 0.0 };
        layout
    }

    pub fn hit(&self, px: f32, py: f32) -> u32 {
        let s = self.scale();
        let x = px / s;
        let y = py / s;
        let l = self.layout();
        if self.dropdown_open && l.list.has(x, y) {
            let off = self.lang_scroll.value;
            let idx = ((y - l.list.y - 8.0 + off) / 40.0).floor();
            if idx >= 0.0 && (idx as usize) < i18n::COUNT {
                return ID_LANG_ITEM + idx as u32;
            }
        }
        if l.close.has(x, y) {
            return ID_CLOSE;
        }
        if l.min.has(x, y) {
            return ID_MIN;
        }
        if l.about.has(x, y) {
            return ID_ABOUT;
        }
        if l.theme_ctl.has(x, y) {
            let step = l.theme_ctl.w / 3.0;
            let index = ((x - l.theme_ctl.x) / step) as i32;
            return ID_THEME_ITEM + index.clamp(0, 2) as u32;
        }
        if l.lang.has(x, y) {
            return ID_LANG;
        }
        if self.about_mode {
            return 0;
        }
        for i in 0..3 {
            if l.toggles[i].has(x, y) {
                return ID_TOGGLE + i as u32;
            }
        }
        for i in 0..4 {
            if l.fields[i].has(x, y) {
                return ID_KEY + i as u32;
            }
            if l.clears[i].has(x, y) {
                return ID_CLEAR + i as u32;
            }
            if l.resets[i].has(x, y) {
                return ID_RESET + i as u32;
            }
        }
        if l.reset_all.has(x, y) {
            return ID_RESET_ALL;
        }
        0
    }

    pub fn hover_id(&self) -> u32 {
        self.hover
    }

    pub unsafe fn on_mouse_move(&mut self, x: f32, y: f32) {
        let id = self.hit(x, y);
        if id != self.hover {
            self.hover = id;
            self.animate_hover(id);
            self.invalidate();
        }
    }

    pub unsafe fn on_leave(&mut self) {
        if self.hover != 0 {
            self.hover = 0;
            self.animate_hover(0);
            self.invalidate();
        }
    }

    fn animate_hover(&mut self, id: u32) {
        for i in 0..IDS {
            let want = if i as u32 == id { 1.0 } else { 0.0 };
            let speed = if i == ID_MIN as usize || i == ID_CLOSE as usize { 34.0 } else { 14.0 };
            self.anims[i].set_speed(speed);
            if self.anims[i].to(want) {
                self.request_anim();
            }
        }
    }

    pub unsafe fn on_down(&mut self, x: f32, y: f32) {
        if self.dropdown_open {
            let s = self.scale();
            let px = x / s;
            let py = y / s;
            let l = self.layout();
            if !l.list.has(px, py) && !l.lang.has(px, py) {
                self.dropdown_open = false;
                self.dropdown_anim.to(0.0);
                self.request_anim();
                self.invalidate();
            }
        }
        let id = self.hit(x, y);
        if id != 0 && id != ID_LANG {
            self.lang_focus = false;
        }
        self.press = id;
        if id != 0 {
            self.request_anim();
            self.invalidate();
        }
        self.on_mouse_move(x, y);
    }

    pub unsafe fn on_up(&mut self, x: f32, y: f32) {
        let id = self.hit(x, y);
        let pressed = self.press;
        self.press = 0;
        if id != 0 && id == pressed {
            self.action(id);
        }
        self.invalidate();
    }

    pub unsafe fn on_wheel(&mut self, delta: i16, sx: f32, sy: f32) {
        let mut pt = POINT { x: sx as i32, y: sy as i32 };
        ScreenToClient(self.hwnd, &mut pt);
        let s = self.scale();
        let l = self.layout();
        let cx = pt.x as f32 / s;
        let cy = pt.y as f32 / s;
        if self.dropdown_open {
            if l.list.has(cx, cy) || l.lang.has(cx, cy) {
                let max = l.list_max_scroll;
                let mut target = self.lang_scroll_target - delta as f32 / 120.0 * 72.0;
                if target < 0.0 {
                    target = 0.0;
                }
                if target > max {
                    target = max;
                }
                self.lang_scroll_target = target;
                self.lang_scroll.to(target);
                self.request_anim();
                self.invalidate();
                return;
            }
        } else if l.lang.has(cx, cy) {
            self.lang_focus = true;
            self.step_language(if delta > 0 { -1 } else { 1 });
            return;
        }
        if !self.about_mode {
            return;
        }
        self.scroll_target -= delta as f32 / 120.0 * 84.0;
        if self.scroll_target < 0.0 {
            self.scroll_target = 0.0;
        }
        if self.scroll_target > self.max_scroll {
            self.scroll_target = self.max_scroll;
        }
        self.request_anim();
        self.invalidate();
    }

    pub unsafe fn on_key(&mut self, vk: u32, down: bool) -> bool {
        let key = vk & 0xFF;
        if !down {
            return false;
        }
        if let Some(index) = self.capture {
            if key == VK_ESCAPE {
                self.capture = None;
                hotkey::set_capturing(false);
                self.invalidate();
                return true;
            }
            if hotkey::is_modifier(key) {
                return true;
            }
            let mods = hotkey::current_mods();
            self.set_shortcut(index, key, mods);
            self.capture = None;
            hotkey::set_capturing(false);
            self.settings.save();
            hotkey::update(self.settings);
            return true;
        }
        if self.lang_focus {
            if key == VK_UP {
                self.step_language(-1);
                return true;
            }
            if key == VK_DOWN {
                self.step_language(1);
                return true;
            }
            if key == VK_SPACE || key == VK_RETURN {
                self.dropdown_open = !self.dropdown_open;
                self.dropdown_anim.to(if self.dropdown_open { 1.0 } else { 0.0 });
                self.request_anim();
                self.invalidate();
                return true;
            }
        }
        if key == VK_ESCAPE {
            if self.dropdown_open {
                self.dropdown_open = false;
                self.dropdown_anim.to(0.0);
                self.invalidate();
                return true;
            }
        }
        false
    }

    pub unsafe fn set_language(&mut self, index: usize) {
        if index == self.lang || index >= i18n::COUNT {
            return;
        }
        self.lang = index;
        self.settings.language = index;
        self.settings.language_auto = false;
        self.settings.save();
        hotkey::update(self.settings);
        let title = w(self.t().title);
        SetWindowTextW(self.hwnd, title.as_ptr());
    }

    pub unsafe fn step_language(&mut self, dir: i32) -> bool {
        let next = self.lang as i32 + dir;
        if next < 0 || next as usize >= i18n::COUNT {
            return false;
        }
        self.set_language(next as usize);
        self.request_anim();
        self.invalidate();
        true
    }

    pub fn shortcut_index(&self, i: usize) -> (u32, u32) {
        match i {
            0 => (self.settings.vk_create, self.settings.mod_create),
            1 => (self.settings.vk_notepad, self.settings.mod_notepad),
            2 => (self.settings.vk_switch_create, self.settings.mod_switch_create),
            _ => (self.settings.vk_switch_notepad, self.settings.mod_switch_notepad),
        }
    }

    pub fn default_shortcut(&self, i: usize) -> (u32, u32) {
        let base = Settings::defaults();
        match i {
            0 => (base.vk_create, base.mod_create),
            1 => (base.vk_notepad, base.mod_notepad),
            2 => (base.vk_switch_create, base.mod_switch_create),
            _ => (base.vk_switch_notepad, base.mod_switch_notepad),
        }
    }

    pub fn set_shortcut(&mut self, i: usize, vk: u32, mods: u32) {
        match i {
            0 => {
                self.settings.vk_create = vk;
                self.settings.mod_create = mods;
            }
            1 => {
                self.settings.vk_notepad = vk;
                self.settings.mod_notepad = mods;
            }
            2 => {
                self.settings.vk_switch_create = vk;
                self.settings.mod_switch_create = mods;
            }
            _ => {
                self.settings.vk_switch_notepad = vk;
                self.settings.mod_switch_notepad = mods;
            }
        }
    }

    pub unsafe fn action(&mut self, id: u32) {
        if id == ID_CLOSE {
            self.request_close();
            return;
        }
        if id == ID_MIN {
            ShowWindow(self.hwnd, SW_MINIMIZE);
            return;
        }
        if id == ID_LANG {
            self.lang_focus = true;
            self.dropdown_open = !self.dropdown_open;
            self.dropdown_anim.to(if self.dropdown_open { 1.0 } else { 0.0 });
            self.request_anim();
            self.invalidate();
            return;
        }
        if id >= ID_LANG_ITEM && id < ID_LANG_ITEM + i18n::COUNT as u32 {
            let index = (id - ID_LANG_ITEM) as usize;
            self.dropdown_open = false;
            self.dropdown_anim.to(0.0);
            self.set_language(index);
            self.request_anim();
            self.invalidate();
            return;
        }
        if id >= ID_THEME_ITEM && id < ID_THEME_ITEM + 3 {
            self.set_theme_mode(id - ID_THEME_ITEM);
            return;
        }
        if id == ID_ABOUT {
            self.about_mode = !self.about_mode;
            self.scroll = 0.0;
            self.scroll_target = 0.0;
            self.request_anim();
            self.invalidate();
            return;
        }
        if id >= ID_TOGGLE && id < ID_TOGGLE + 3 {
            let index = (id - ID_TOGGLE) as usize;
            let value = self.toggles[index].value < 0.5;
            self.toggles[index].to(if value { 1.0 } else { 0.0 });
            match index {
                0 => self.settings.enable_create = value,
                1 => self.settings.enable_notepad = value,
                _ => self.settings.autostart = value,
            }
            self.settings.save();
            if index == 2 {
                self.settings.apply_autostart();
            }
            hotkey::update(self.settings);
            self.request_anim();
            self.invalidate();
            return;
        }
        if id >= ID_KEY && id < ID_KEY + 4 {
            self.capture = Some((id - ID_KEY) as usize);
            hotkey::set_capturing(true);
            self.request_anim();
            self.invalidate();
            return;
        }
        if id >= ID_CLEAR && id < ID_CLEAR + 4 {
            let index = (id - ID_CLEAR) as usize;
            self.set_shortcut(index, 0, 0);
            self.capture = None;
            hotkey::set_capturing(false);
            self.settings.save();
            hotkey::update(self.settings);
            self.invalidate();
            return;
        }
        if id >= ID_RESET && id < ID_RESET + 4 {
            let index = (id - ID_RESET) as usize;
            let (vk, mods) = self.default_shortcut(index);
            self.set_shortcut(index, vk, mods);
            self.dismiss_capture();
            self.settings.save();
            hotkey::update(self.settings);
            self.invalidate();
            return;
        }
        if id == ID_RESET_ALL {
            for i in 0..4 {
                let (vk, mods) = self.default_shortcut(i);
                self.set_shortcut(i, vk, mods);
            }
            self.dismiss_capture();
            self.settings.save();
            hotkey::update(self.settings);
            self.invalidate();
        }
    }

    fn dismiss_capture(&mut self) {
        self.capture = None;
        hotkey::set_capturing(false);
    }

    fn toggle_label(&self, index: usize) -> &'static str {
        let t = self.t();
        match index {
            0 => t.enable_create,
            1 => t.enable_notepad,
            _ => t.autostart,
        }
    }

    pub fn request_close(&mut self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub unsafe fn sync_toggles(&mut self) {
        self.toggles[0].to(if self.settings.enable_create { 1.0 } else { 0.0 });
        self.toggles[1].to(if self.settings.enable_notepad { 1.0 } else { 0.0 });
        self.toggles[2].to(if self.settings.autostart { 1.0 } else { 0.0 });
        self.request_anim();
        self.invalidate();
    }

    pub fn request_anim(&self) {
        unsafe {
            SetTimer(self.hwnd, 1, 15, 0);
        }
    }

    pub unsafe fn on_timer(&mut self) -> bool {
        let dt = 1.0 / 64.0;
        let mut active = false;
        for i in 0..IDS {
            if self.anims[i].step(dt) {
                active = true;
            }
        }
        for i in 0..3 {
            if self.toggles[i].step(dt) {
                active = true;
            }
        }
        if self.dropdown_anim.step(dt) {
            active = true;
        }
        if self.lang_scroll.step(dt) {
            active = true;
        }
        if self.capture_anim.step(dt) {
            active = true;
        }
        if self.theme_anim.step(dt) {
            self.theme = lerp(self.theme_from, self.theme_to, self.theme_anim.value);
            self.sync_frame();
            active = true;
        } else if self.theme.bg != self.theme_to.bg {
            self.theme = self.theme_to;
            self.sync_frame();
        }
        if (self.scroll - self.scroll_target).abs() > 0.5 {
            self.scroll += (self.scroll_target - self.scroll) * (1.0 - (-18.0 * dt).exp());
            active = true;
        } else {
            self.scroll = self.scroll_target;
        }
        if !active {
            KillTimer(self.hwnd, 1);
            return false;
        }
        self.invalidate();
        true
    }

    pub unsafe fn paint(&mut self) {
        let mut rect: RECT = core::mem::zeroed();
        GetClientRect(self.hwnd, &mut rect);
        if rect.right != self.w || rect.bottom != self.h {
            self.resize(rect.right, rect.bottom);
        }
        let stale = match self.canvas.as_ref() {
            Some(c) => c.w != self.w || c.h != self.h,
            None => true,
        };
        if stale {
            self.resize(self.w, self.h);
        }
        let mut g = match self.canvas.as_ref() {
            Some(c) => match Gfx::new(c) {
                Some(g) => g,
                None => return,
            },
            None => return,
        };
        let s = self.scale();
        g.set_scale(s);
        let theme = self.theme;
        g.set_backdrop(theme.bg);
        let l = self.layout();
        let t = i18n::lang(self.lang);
        g.rect(0.0, 0.0, l.w, l.h, theme.bg);
        g.rect(0.0, 0.0, l.w, 44.0, theme.rail);
        g.rect(0.0, 43.5, l.w, 1.0, theme.border);
        if self.logo != 0 {
            g.icon(self.logo, 16, 12, 20);
        } else {
            g.round(16.0, 12.0, 20.0, 20.0, 6.0, theme.accent);
        }
        g.text(t.title, 44.0, 0.0, 200.0, 44.0, 13, true, theme.text, Align::Near);
        self.window_button(&mut g, l.min, ID_MIN, false, theme);
        self.window_button(&mut g, l.close, ID_CLOSE, true, theme);
        g.text(t.lang_label, 22.0, l.lang.y, 74.0, l.lang.h, 13, false, theme.dim, Align::Near);
        self.pill_button(&mut g, l.lang, ID_LANG, false, theme);
        if self.lang_focus && !self.dropdown_open {
            g.stroke(l.lang.x, l.lang.y, l.lang.w, l.lang.h, 10.0, theme.accent, 1.6);
        }
        let lang_hover = self.anims[ID_LANG as usize].value;
        g.text(i18n::NAMES[self.lang], l.lang.x + 16.0, l.lang.y, l.lang.w - 46.0, l.lang.h, 13, false, theme.text, Align::Near);
        let cx = l.lang.x + l.lang.w - 18.0;
        let cy = l.lang.cy();
        let chevron = mix(theme.dim, theme.text, lang_hover);
        let angle = core::f32::consts::PI * self.dropdown_anim.value;
        let (sa, ca) = (angle.sin(), angle.cos());
        let rot = |px: f32, py: f32| -> (f32, f32) { (cx + px * ca - py * sa, cy + px * sa + py * ca) };
        let (x1, y1) = rot(-5.0, -2.0);
        let (x2, y2) = rot(0.0, 3.0);
        let (x3, y3) = rot(5.0, -2.0);
        g.line(x1, y1, x2, y2, chevron, 1.6);
        g.line(x2, y2, x3, y3, chevron, 1.6);
        self.theme_switch(&mut g, &l, t, theme);
        let about_label = if self.about_mode { t.back } else { t.about };
        self.ghost_button(&mut g, l.about, ID_ABOUT, about_label, theme, true);
        if self.about_mode {
            self.paint_about(&mut g, l.card_about, t, theme);
        } else {
            self.paint_left(&mut g, l.card_left, t, theme);
            self.paint_right(&mut g, &l, t, theme);
        }
        if self.dropdown_open || self.dropdown_anim.value > 0.01 {
            self.paint_dropdown(&mut g, &l, theme);
        }
        g.free();
        let hdc = GetDC(self.hwnd);
        if let Some(mut canvas) = self.canvas.take() {
            canvas.present(hdc);
            canvas.free();
        }
        ReleaseDC(self.hwnd, hdc);
        let _ = s;
    }

    unsafe fn window_button(&mut self, g: &mut Gfx, r: RectF, id: u32, is_close: bool, theme: Theme) {
        let hv = self.anims[id as usize].value;
        let pr = if self.press == id { 1.0 } else { 0.0 };
        if hv > 0.01 {
            let color = if is_close { mix(theme.bg, theme.danger, 0.92 * hv) } else { mix(theme.bg, theme.border, hv) };
            g.round(r.x + 4.0, r.y + 5.0, r.w - 8.0, r.h - 10.0, 8.0, color);
        }
        let ink = if is_close { mix(theme.text, 0xFFFFFFFF, hv * 0.7) } else { mix(theme.dim, theme.text, hv) };
        let c = RectF { x: r.x, y: r.y, w: r.w, h: r.h };
        let shrink = pr * 0.6;
        if is_close {
            g.line(c.cx() - 5.0 + shrink, c.cy() - 5.0 + shrink, c.cx() + 5.0 - shrink, c.cy() + 5.0 - shrink, ink, 1.5);
            g.line(c.cx() + 5.0 - shrink, c.cy() - 5.0 + shrink, c.cx() - 5.0 + shrink, c.cy() + 5.0 - shrink, ink, 1.5);
        } else {
            g.line(c.cx() - 6.0, c.cy() + 1.0, c.cx() + 6.0, c.cy() + 1.0, ink, 1.5);
        }
    }

    unsafe fn pill_button(&mut self, g: &mut Gfx, r: RectF, id: u32, primary: bool, theme: Theme) {
        let hv = self.anims[id as usize].value;
        let pr = if self.press == id { 1.0 } else { 0.0 };
        if primary {
            let c1 = mix(theme.accent, theme.accent2, hv * 0.6);
            g.round_grad(r.x, r.y, r.w, r.h, 10.0, c1, lighten(c1, 0.12));
            if pr > 0.0 {
                g.round(r.x, r.y, r.w, r.h, 10.0, with_alpha(0xFF000000, 0.06 * pr));
            }
        } else {
            g.round(r.x, r.y, r.w, r.h, 10.0, mix(theme.surface, theme.surface2, hv));
            g.stroke(r.x, r.y, r.w, r.h, 10.0, mix(theme.border, theme.accent, hv * 0.5), 1.0);
        }
    }

    unsafe fn ghost_button(&mut self, g: &mut Gfx, r: RectF, id: u32, label: &str, theme: Theme, outlined: bool) {
        let hv = self.anims[id as usize].value;
        let pr = if self.press == id { 1.0 } else { 0.0 };
        let fill = mix(theme.surface, theme.surface2, hv);
        g.round(r.x, r.y, r.w, r.h, 10.0, fill);
        if outlined {
            g.stroke(r.x, r.y, r.w, r.h, 10.0, mix(theme.border, theme.accent, hv * 0.6), 1.0);
        }
        if pr > 0.0 {
            g.round(r.x, r.y, r.w, r.h, 10.0, mix(fill, theme.accent, 0.16 * pr));
        }
        let ink = mix(theme.text, theme.accent, hv * 0.7);
        let mut size = 13u8;
        while size > 9 && g.text_width(label, size, false) > r.w - 14.0 {
            size -= 1;
        }
        g.text(label, r.x, r.y, r.w, r.h, size, false, ink, Align::Center);
    }

    unsafe fn theme_switch(&mut self, g: &mut Gfx, l: &Layout, t: &'static Lang, theme: Theme) {
        let r = l.theme_ctl;
        let labels = [t.theme_auto, t.theme_light, t.theme_dark];
        let step = r.w / 3.0;
        let selected = self.settings.theme_mode as usize;
        let muted = mix(theme.text, theme.bg, 0.55);
        for i in 0..3 {
            let x = r.x + step * i as f32;
            let wd = if i == 2 { r.x + r.w - x } else { step };
            let hv = self.anims[(ID_THEME_ITEM + i as u32) as usize].value;
            if i == selected {
                g.round(x + 0.5, r.y + 0.5, wd - 1.0, r.h - 1.0, 9.0, theme.accent);
            } else if hv > 0.01 {
                g.round(x + 0.5, r.y + 0.5, wd - 1.0, r.h - 1.0, 9.0, mix(theme.bg, theme.text, 0.07 * hv));
            }
            let ink = if i == selected { 0xFFFFFFFF } else { mix(muted, theme.text, hv * 0.45) };
            let mut size = 12u8;
            while size > 9 && g.text_width(labels[i], size, i == selected) > wd - 10.0 {
                size -= 1;
            }
            g.text(labels[i], x, r.y, wd, r.h, size, i == selected, ink, Align::Center);
        }
        g.stroke(r.x, r.y, r.w, r.h, 10.0, theme.border, 1.0);
    }

    unsafe fn card(&mut self, g: &mut Gfx, r: RectF, theme: Theme) {
        g.set_backdrop(theme.bg);
        g.shadow(r.x, r.y, r.w, r.h, 16.0, theme.shadow);
        g.round(r.x, r.y, r.w, r.h, 16.0, theme.surface);
        g.stroke(r.x, r.y, r.w, r.h, 16.0, theme.border, 1.0);
        g.set_backdrop(theme.surface);
    }

    unsafe fn paint_left(&mut self, g: &mut Gfx, r: RectF, t: &'static Lang, theme: Theme) {
        self.card(g, r, theme);
        g.text(t.group_switch, r.x + 22.0, r.y + 16.0, r.w - 44.0, 26.0, 12, true, theme.dim, Align::Near);
        let labels = [t.enable_create, t.enable_notepad, t.autostart];
        for i in 0..3 {
            let row_y = r.y + 54.0 + i as f32 * 54.0;
            if i == 2 {
                g.rect(r.x + 22.0, row_y - 6.0, r.w - 44.0, 1.0, theme.border);
            }
            let avail = r.w - 100.0;
            let mut size = 14u8;
            while size > 10 && g.text_width(labels[i], size, false) > avail {
                size -= 1;
            }
            g.text(labels[i], r.x + 22.0, row_y, avail, 54.0, size, false, theme.text, Align::Near);
            let rect = RectF { x: r.x + r.w - 22.0 - 46.0, y: row_y + 14.0, w: 46.0, h: 26.0 };
            let v = self.toggles[i].value;
            let track = mix(theme.surface2, theme.accent, v);
            let border = mix(theme.border, theme.accent, v);
            g.round(rect.x, rect.y, rect.w, rect.h, 13.0, track);
            g.stroke(rect.x, rect.y, rect.w, rect.h, 13.0, border, 1.0);
            let knob_x = rect.x + 3.0 + (rect.w - 6.0 - 20.0) * v;
            g.circle(knob_x + 10.0, rect.y + rect.h * 0.5, 10.0, mix(theme.dim, 0xFFFFFFFF, v));
        }
    }

    unsafe fn paint_right(&mut self, g: &mut Gfx, l: &Layout, t: &'static Lang, theme: Theme) {
        let r = l.card_right;
        self.card(g, r, theme);
        g.text(t.group_shortcuts, r.x + 22.0, r.y + 16.0, r.w - 44.0, 26.0, 12, true, theme.dim, Align::Near);
        let labels = [t.row_create, t.row_notepad, t.row_switch_create, t.row_switch_notepad];
        for i in 0..4 {
            let row_y = r.y + 54.0 + i as f32 * 54.0;
            let mut size = 13u8;
            while size > 9 && g.text_width(labels[i], size, false) > 168.0 {
                size -= 1;
            }
            g.text(labels[i], r.x + 22.0, row_y, 168.0, 54.0, size, false, theme.text, Align::Near);
            let field = l.fields[i];
            let focused = self.capture == Some(i);
            let hv = self.anims[(ID_KEY + i as u32) as usize].value;
            let fill = mix(theme.surface2, theme.surface, hv * 0.0);
            g.round(field.x, field.y, field.w, field.h, 9.0, fill);
            let border = if focused { theme.accent } else { mix(theme.border, theme.accent, hv * 0.5) };
            g.stroke(field.x, field.y, field.w, field.h, 9.0, border, if focused { 1.6 } else { 1.0 });
            let (vk, mods) = self.shortcut_index(i);
            let text = if focused {
                t.capturing.to_string()
            } else if vk == 0 {
                t.empty.to_string()
            } else {
                hotkey::shortcut_text(vk, mods)
            };
            let ink = if focused { theme.accent } else if vk == 0 { theme.dim } else { theme.text };
            if focused {
                let phase = ((clock() * 1.6).sin() * 0.5 + 0.5) as f32;
                let width = g.text_width(&text, 13, false);
                g.rect(field.x + 14.0 + width + 4.0, field.y + 10.0, 1.6, field.h - 20.0, with_alpha(theme.accent, 0.25 + phase * 0.75));
            }
            g.text(&text, field.x + 14.0, field.y, field.w - 28.0, field.h, 13, false, ink, Align::Near);
            let clear = l.clears[i];
            let reset = l.resets[i];
            self.ghost_button(g, clear, ID_CLEAR + i as u32, t.clear, theme, true);
            self.ghost_button(g, reset, ID_RESET + i as u32, t.reset, theme, true);
        }
        let all = l.reset_all;
        let hv = self.anims[ID_RESET_ALL as usize].value;
        let pr = if self.press == ID_RESET_ALL { 1.0 } else { 0.0 };
        let c1 = mix(theme.accent, theme.accent2, 0.5 + hv * 0.5);
        g.round(all.x, all.y, all.w, all.h, 10.0, with_alpha(c1, 0.14 + hv * 0.10));
        if pr > 0.0 {
            g.round(all.x, all.y, all.w, all.h, 10.0, with_alpha(0xFF000000, 0.05 * pr));
        }
        g.stroke(all.x, all.y, all.w, all.h, 10.0, with_alpha(c1, 0.35 + hv * 0.35), 1.0);
        let mut size = 13u8;
        while size > 9 && g.text_width(t.reset_all, size, false) > all.w - 14.0 {
            size -= 1;
        }
        g.text(t.reset_all, all.x, all.y, all.w, all.h, size, false, mix(theme.accent, theme.accent2, hv), Align::Center);
    }

    unsafe fn paint_about(&mut self, g: &mut Gfx, r: RectF, t: &'static Lang, theme: Theme) {
        self.card(g, r, theme);
        let inner_x = r.x + 28.0;
        let inner_w = r.w - 56.0;
        let top = r.y + 24.0;
        let bottom = r.y + r.h - 24.0;
        let line_h = 24.0f32;
        let lines = wrap(g, t.about_text, inner_w, 14);
        let total = lines.len() as f32 * line_h;
        self.max_scroll = (total - (bottom - top)).max(0.0);
        if self.scroll > self.max_scroll {
            self.scroll = self.max_scroll;
            self.scroll_target = self.max_scroll;
        }
        g.clip(inner_x - 4.0, top - 4.0, inner_w + 8.0, bottom - top + 8.0);
        let offset = top - self.scroll;
        for (i, line) in lines.iter().enumerate() {
            let y = offset + i as f32 * line_h;
            if y > bottom || y + line_h < top {
                continue;
            }
            let ink = if line.starts_with('【') || line.starts_with('[') { theme.text } else { mix(theme.text, theme.dim, 0.35) };
            g.text(line, inner_x, y, inner_w, line_h, 14, line.starts_with('【') || line.starts_with('['), ink, Align::Near);
        }
        g.unclip();
        if self.max_scroll > 0.0 {
            let track_x = r.x + r.w - 14.0;
            let track_y = top;
            let track_h = bottom - top;
            let thumb_h = (track_h * track_h / total).max(48.0);
            let ratio = if self.max_scroll > 0.0 { self.scroll / self.max_scroll } else { 0.0 };
            let thumb_y = track_y + (track_h - thumb_h) * ratio;
            g.round(track_x, track_y, 4.0, track_h, 2.0, with_alpha(theme.border, 0.5));
            g.round(track_x, thumb_y, 4.0, thumb_h, 2.0, with_alpha(theme.dim, 0.55));
        }
    }

    unsafe fn paint_dropdown(&mut self, g: &mut Gfx, l: &Layout, theme: Theme) {
        let a = self.dropdown_anim.value;
        if a <= 0.01 {
            return;
        }
        g.set_backdrop(theme.surface);
        if self.lang_scroll.value > l.list_max_scroll || self.lang_scroll_target > l.list_max_scroll {
            self.lang_scroll.snap(l.list_max_scroll);
            self.lang_scroll_target = l.list_max_scroll;
        }
        let x = l.list.x;
        let y = l.list.y;
        let wd = l.list.w;
        let hgt = l.list.h * a;
        let off = self.lang_scroll.value;
        g.shadow(x, y, wd, hgt, 12.0, with_alpha(theme.shadow, 0.9 * a));
        g.round(x, y, wd, hgt, 12.0, theme.surface);
        g.stroke(x, y, wd, hgt, 12.0, theme.border, 1.0);
        g.clip(x, y, wd, hgt);
        for i in 0..i18n::COUNT {
            let iy = y + 8.0 + i as f32 * 40.0 - off;
            if iy + 40.0 < y || iy > y + hgt {
                continue;
            }
            let hv = self.anims[(ID_LANG_ITEM + i as u32) as usize].value;
            let selected = i == self.lang;
            if hv > 0.01 {
                g.round(x + 6.0, iy + 2.0, wd - 12.0, 36.0, 8.0, with_alpha(theme.accent, 0.10 * hv));
            }
            let ink = if selected { theme.accent } else { theme.text };
            let mut size = 13u8;
            while size > 10 && g.text_width(i18n::NAMES[i], size, selected) > wd - 28.0 {
                size -= 1;
            }
            g.text(i18n::NAMES[i], x + 14.0, iy, wd - 28.0, 40.0, size, selected, ink, Align::Near);
        }
        g.unclip();
        if l.list_max_scroll > 0.0 {
            let track_h = hgt - 16.0;
            let thumb_h = (track_h * (hgt / (hgt + l.list_max_scroll))).max(28.0);
            let pos = if l.list_max_scroll > 0.0 { off / l.list_max_scroll } else { 0.0 };
            let thumb_y = y + 8.0 + (track_h - thumb_h) * pos;
            g.round(x + wd - 8.0, thumb_y, 3.0, thumb_h, 1.5, with_alpha(theme.dim, 0.45));
        }
    }
}

fn wrap(g: &mut Gfx, text: &str, width: f32, size: u8) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    unsafe {
        for paragraph in text.split('\n') {
            if paragraph.is_empty() {
                out.push(String::new());
                continue;
            }
            let mut line = String::new();
            for ch in paragraph.chars() {
                let mut candidate = line.clone();
                candidate.push(ch);
                if !line.is_empty() && g.text_width(&candidate, size, false) > width {
                    out.push(line.clone());
                    line.clear();
                }
                line.push(ch);
            }
            if !line.is_empty() {
                out.push(line);
            }
        }
    }
    out
}
