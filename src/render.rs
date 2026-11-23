// 功能：基于 GDI 的离屏双缓冲绘图接口
use crate::ffi::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Align {
    Near,
    Center,
    Far,
}

fn colorref(color: u32) -> u32 {
    ((color & 0x0000FF) << 16) | (color & 0x00FF00) | ((color >> 16) & 0x0000FF)
}

fn blend(back: u32, front: u32) -> u32 {
    let a = (front >> 24) & 0xFF;
    if a == 0xFF || a == 0 {
        return if a == 0xFF { front & 0xFFFFFF } else { back & 0xFFFFFF };
    }
    let t = a as u32;
    let inv = 255 - t;
    let mix = |shift: u32| -> u32 {
        let b = (back >> shift) & 0xFF;
        let f = (front >> shift) & 0xFF;
        ((b * inv + f * t) / 255) & 0xFF
    };
    (mix(16) << 16) | (mix(8) << 8) | mix(0)
}

pub struct Canvas {
    pub w: i32,
    pub h: i32,
    dc: HDC,
    old: HGDIOBJ,
    bmp: HBITMAP,
}

impl Canvas {
    pub unsafe fn new(hwnd: HWND, w: i32, h: i32) -> Option<Canvas> {
        if w <= 0 || h <= 0 {
            return None;
        }
        let screen = GetDC(hwnd);
        let reference = if screen == 0 { GetDC(0) } else { screen };
        let dc = CreateCompatibleDC(reference);
        let bmp = CreateCompatibleBitmap(reference, w, h);
        if screen == 0 {
            ReleaseDC(0, reference);
        } else {
            ReleaseDC(hwnd, screen);
        }
        if dc == 0 || bmp == 0 {
            if dc != 0 {
                DeleteDC(dc);
            }
            if bmp != 0 {
                DeleteObject(bmp);
            }
            return None;
        }
        let old = SelectObject(dc, bmp);
        Some(Canvas { w, h, dc, old, bmp })
    }

    pub unsafe fn reinit(&mut self) {}

    pub unsafe fn present(&self, hdc: HDC) {
        BitBlt(hdc, 0, 0, self.w, self.h, self.dc, 0, 0, SRCCOPY);
    }

    pub unsafe fn free(&mut self) {
        SelectObject(self.dc, self.old);
        DeleteObject(self.bmp);
        DeleteDC(self.dc);
        self.bmp = 0;
    }
}

pub struct Gfx {
    dc: HDC,
    s: f32,
    back: u32,
    fonts: Vec<(u8, bool, HFONT)>,
    clips: i32,
}

impl Gfx {
    pub unsafe fn new(canvas: &Canvas) -> Option<Gfx> {
        Some(Gfx { dc: canvas.dc, s: 1.0, back: 0xFFFFFF, fonts: Vec::with_capacity(8), clips: 0 })
    }

    pub unsafe fn from_handle(handle: isize) -> Option<Gfx> {
        Some(Gfx { dc: handle, s: 1.0, back: 0xFFFFFF, fonts: Vec::with_capacity(8), clips: 0 })
    }

    pub unsafe fn set_scale(&mut self, s: f32) {
        self.s = s;
    }

    pub unsafe fn set_backdrop(&mut self, color: u32) {
        self.back = color;
    }

    pub unsafe fn free(&mut self) {
        for (_, _, font) in self.fonts.drain(..) {
            DeleteObject(font);
        }
        while self.clips > 0 {
            RestoreDC(self.dc, -1);
            self.clips -= 1;
        }
    }

    fn px(&self, value: f32) -> i32 {
        (value * self.s).round() as i32
    }

    unsafe fn font(&mut self, size: u8, bold: bool) -> HFONT {
        for (s, b, f) in self.fonts.iter() {
            if *s == size && *b == bold {
                return *f;
            }
        }
        let face: Vec<u16> = "Microsoft YaHei UI\0".encode_utf16().collect();
        let height = -self.px(size as f32);
        let weight = if bold { 600 } else { 400 };
        let font = CreateFontW(height, 0, 0, 0, weight, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0, face.as_ptr());
        self.fonts.push((size, bold, font));
        font
    }

    pub unsafe fn text(&mut self, s: &str, x: f32, y: f32, wd: f32, ht: f32, size: u8, bold: bool, color: u32, align: Align) {
        if s.is_empty() || wd <= 1.0 {
            return;
        }
        let font = self.font(size, bold);
        if font == 0 {
            return;
        }
        let units: Vec<u16> = s.encode_utf16().collect();
        let mut rect = RECT { left: self.px(x), top: self.px(y), right: self.px(x + wd), bottom: self.px(y + ht) };
        let old = SelectObject(self.dc, font);
        SetBkMode(self.dc, 1);
        SetTextColor(self.dc, colorref(blend(self.back, color)));
        let flags = DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX | DT_END_ELLIPSIS
            | match align {
                Align::Near => 0,
                Align::Center => DT_CENTER,
                Align::Far => DT_RIGHT,
            };
        DrawTextW(self.dc, units.as_ptr(), units.len() as i32, &mut rect, flags);
        SelectObject(self.dc, old);
    }

    pub unsafe fn text_width(&mut self, s: &str, size: u8, bold: bool) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        let font = self.font(size, bold);
        let units: Vec<u16> = s.encode_utf16().collect();
        let old = SelectObject(self.dc, font);
        let mut extent = SIZE { cx: 0, cy: 0 };
        GetTextExtentPoint32W(self.dc, units.as_ptr(), units.len() as i32, &mut extent);
        SelectObject(self.dc, old);
        extent.cx as f32 / self.s
    }

    unsafe fn fill(&self, l: i32, t: i32, r: i32, b: i32, color: u32) {
        let brush = CreateSolidBrush(colorref(blend(self.back, color)));
        let pen = GetStockObject(NULL_PEN_STOCK);
        let old_brush = SelectObject(self.dc, brush);
        let old_pen = SelectObject(self.dc, pen);
        Rectangle(self.dc, l, t, r, b);
        SelectObject(self.dc, old_pen);
        SelectObject(self.dc, old_brush);
        DeleteObject(brush);
    }

    pub unsafe fn rect(&mut self, x: f32, y: f32, wd: f32, ht: f32, color: u32) {
        self.fill(self.px(x), self.px(y), self.px(x + wd), self.px(y + ht), color);
    }

    unsafe fn shape(&self, x: f32, y: f32, wd: f32, ht: f32, r: f32, fill: u32, line: u32, width: f32) {
        let brush = if fill == 0 { GetStockObject(NULL_BRUSH_STOCK) } else { CreateSolidBrush(colorref(blend(self.back, fill))) };
        let pen = if line == 0 {
            GetStockObject(NULL_PEN_STOCK)
        } else {
            CreatePen(PS_SOLID_STYLE, self.px(width).max(1), colorref(blend(self.back, line)))
        };
        let old_brush = SelectObject(self.dc, brush);
        let old_pen = SelectObject(self.dc, pen);
        let radius = if r * 2.0 > wd.min(ht) { wd.min(ht) * 0.5 } else { r };
        let d = self.px(radius) * 2;
        RoundRect(self.dc, self.px(x), self.px(y), self.px(x + wd), self.px(y + ht), d, d);
        SelectObject(self.dc, old_pen);
        SelectObject(self.dc, old_brush);
        if fill != 0 {
            DeleteObject(brush);
        }
        if line != 0 {
            DeleteObject(pen);
        }
    }

    pub unsafe fn round(&mut self, x: f32, y: f32, wd: f32, ht: f32, r: f32, color: u32) {
        self.shape(x, y, wd, ht, r, color, 0, 0.0);
    }

    pub unsafe fn round_grad(&mut self, x: f32, y: f32, wd: f32, ht: f32, r: f32, c1: u32, c2: u32) {
        let saved = SaveDC(self.dc);
        if BeginPath(self.dc) != 0 {
            let radius = if r * 2.0 > wd.min(ht) { wd.min(ht) * 0.5 } else { r };
            let d = self.px(radius) * 2;
            RoundRect(self.dc, self.px(x), self.px(y), self.px(x + wd), self.px(y + ht), d, d);
            if EndPath(self.dc) != 0 && SelectClipPath(self.dc, RGN_COPY_MODE) != 0 {
                let steps = 24;
                let l = self.px(x);
                let rr = self.px(x + wd);
                for i in 0..steps {
                    let t0 = i as f32 / steps as f32;
                    let t1 = (i + 1) as f32 / steps as f32;
                    let top = self.px(y + ht * t0);
                    let bottom = self.px(y + ht * t1).max(top + 1);
                    let mixed = {
                        let a = (c1 >> 24) & 0xFF;
                        let cr = ((c1 >> 16) & 0xFF) as f32 * (1.0 - t0) + ((c2 >> 16) & 0xFF) as f32 * t0;
                        let cg = ((c1 >> 8) & 0xFF) as f32 * (1.0 - t0) + ((c2 >> 8) & 0xFF) as f32 * t0;
                        let cb = (c1 & 0xFF) as f32 * (1.0 - t0) + (c2 & 0xFF) as f32 * t0;
                        (a << 24) | ((cr as u32) << 16) | ((cg as u32) << 8) | cb as u32
                    };
                    self.fill(l, top, rr, bottom, mixed);
                }
            }
        }
        RestoreDC(self.dc, saved);
    }

    pub unsafe fn stroke(&mut self, x: f32, y: f32, wd: f32, ht: f32, r: f32, color: u32, width: f32) {
        self.shape(x, y, wd, ht, r, 0, color, width);
    }

    pub unsafe fn circle(&mut self, cx: f32, cy: f32, r: f32, color: u32) {
        let brush = CreateSolidBrush(colorref(blend(self.back, color)));
        let pen = GetStockObject(NULL_PEN_STOCK);
        let old_brush = SelectObject(self.dc, brush);
        let old_pen = SelectObject(self.dc, pen);
        Ellipse(self.dc, self.px(cx - r), self.px(cy - r), self.px(cx + r), self.px(cy + r));
        SelectObject(self.dc, old_pen);
        SelectObject(self.dc, old_brush);
        DeleteObject(brush);
    }

    pub unsafe fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, width: f32) {
        let pen = CreatePen(PS_SOLID_STYLE, self.px(width).max(1), colorref(blend(self.back, color)));
        let old_pen = SelectObject(self.dc, pen);
        MoveToEx(self.dc, self.px(x1), self.px(y1), core::ptr::null_mut());
        LineTo(self.dc, self.px(x2), self.px(y2));
        SelectObject(self.dc, old_pen);
        DeleteObject(pen);
    }

    pub unsafe fn shadow(&mut self, x: f32, y: f32, wd: f32, ht: f32, r: f32, color: u32) {
        let layers = 4;
        for i in 0..layers {
            let t = i as f32 / (layers - 1) as f32;
            let grow = 1.0 + t * 7.0;
            let alpha = (0.045 * (1.0 - t) + 0.012) * 255.0;
            self.round(x - grow, y - grow + 1.5, wd + grow * 2.0, ht + grow * 2.0, r + grow, with_alpha(color, alpha / 255.0));
        }
    }

    pub unsafe fn icon(&mut self, image: isize, x: i32, y: i32, size: i32) {
        if image == 0 {
            return;
        }
        DrawIconEx(self.dc, self.px(x as f32), self.px(y as f32), image, self.px(size as f32), self.px(size as f32), 0, 0, DI_NORMAL);
    }

    pub unsafe fn clip(&mut self, x: f32, y: f32, wd: f32, ht: f32) {
        SaveDC(self.dc);
        self.clips += 1;
        IntersectClipRect(self.dc, self.px(x), self.px(y), self.px(x + wd), self.px(y + ht));
    }

    pub unsafe fn unclip(&mut self) {
        if self.clips > 0 {
            RestoreDC(self.dc, -1);
            self.clips -= 1;
        }
    }
}

fn with_alpha(color: u32, alpha: f32) -> u32 {
    let a = (alpha.clamp(0.0, 1.0) * 255.0) as u32;
    (a << 24) | (color & 0xFFFFFF)
}

pub unsafe fn icon_image(inst: HINSTANCE, size: i32) -> isize {
    LoadImageW(inst, 1 as *const u16, IMAGE_ICON, size, size, LR_DEFAULTCOLOR)
}
