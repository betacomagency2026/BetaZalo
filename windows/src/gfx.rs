//! Vẽ hình khử răng cưa bằng GDI+ và chữ bằng GDI (ClearType).

use std::ptr::null_mut;

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::Graphics::GdiPlus::*;

use crate::util::w;

pub fn gdiplus_init() {
    unsafe {
        let mut token = 0usize;
        let input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: 0,
            SuppressExternalCodecs: 0,
        };
        GdiplusStartup(&mut token, &input, null_mut());
    }
}

/// 0xRRGGBB -> ARGB đục
pub fn argb(rgb: u32) -> u32 {
    0xFF00_0000 | rgb
}

/// 0xRRGGBB -> COLORREF (0x00BBGGRR)
pub fn cref(rgb: u32) -> COLORREF {
    ((rgb & 0xFF) << 16) | (rgb & 0xFF00) | ((rgb >> 16) & 0xFF)
}

pub struct Canvas {
    pub hdc: HDC,
    g: *mut GpGraphics,
    texts: Vec<(RECT, String, HFONT, u32, u32)>,
}

impl Canvas {
    pub fn new(hdc: HDC) -> Canvas {
        let mut g: *mut GpGraphics = null_mut();
        unsafe {
            GdipCreateFromHDC(hdc, &mut g);
            GdipSetSmoothingMode(g, SmoothingModeAntiAlias);
            GdipSetPixelOffsetMode(g, PixelOffsetModeHalf);
        }
        Canvas { hdc, g, texts: vec![] }
    }

    pub fn fill_rect(&mut self, r: RECT, rgb: u32) {
        unsafe {
            let b = CreateSolidBrush(cref(rgb));
            FillRect(self.hdc, &r, b);
            DeleteObject(b);
        }
    }

    fn round_path(x: i32, y: i32, wd: i32, ht: i32, r: i32) -> *mut GpPath {
        let mut p: *mut GpPath = null_mut();
        unsafe {
            GdipCreatePath(FillModeAlternate, &mut p);
            let d = (r * 2).min(wd).min(ht).max(1) as f32;
            let (x, y, wd, ht) = (x as f32, y as f32, wd as f32, ht as f32);
            GdipAddPathArc(p, x, y, d, d, 180.0, 90.0);
            GdipAddPathArc(p, x + wd - d, y, d, d, 270.0, 90.0);
            GdipAddPathArc(p, x + wd - d, y + ht - d, d, d, 0.0, 90.0);
            GdipAddPathArc(p, x, y + ht - d, d, d, 90.0, 90.0);
            GdipClosePathFigure(p);
        }
        p
    }

    pub fn fill_round(&mut self, x: i32, y: i32, wd: i32, ht: i32, r: i32, rgb: u32) {
        unsafe {
            let p = Self::round_path(x, y, wd, ht, r);
            let mut b: *mut GpSolidFill = null_mut();
            GdipCreateSolidFill(argb(rgb), &mut b);
            GdipFillPath(self.g, b as *mut GpBrush, p);
            GdipDeleteBrush(b as *mut GpBrush);
            GdipDeletePath(p);
        }
    }

    pub fn stroke_round(&mut self, x: i32, y: i32, wd: i32, ht: i32, r: i32, rgb: u32, width: f32) {
        unsafe {
            let p = Self::round_path(x, y, wd - 1, ht - 1, r);
            let mut pen: *mut GpPen = null_mut();
            GdipCreatePen1(argb(rgb), width, UnitPixel, &mut pen);
            GdipDrawPath(self.g, pen, p);
            GdipDeletePen(pen);
            GdipDeletePath(p);
        }
    }

    pub fn fill_circle(&mut self, x: i32, y: i32, d: i32, rgb: u32) {
        unsafe {
            let mut b: *mut GpSolidFill = null_mut();
            GdipCreateSolidFill(argb(rgb), &mut b);
            GdipFillEllipse(self.g, b as *mut GpBrush, x as f32, y as f32, d as f32, d as f32);
            GdipDeleteBrush(b as *mut GpBrush);
        }
    }

    /// Chữ được vẽ sau cùng (đè lên hình).
    pub fn text(&mut self, r: RECT, s: &str, font: HFONT, rgb: u32, flags: u32) {
        self.texts.push((r, s.to_string(), font, rgb, flags));
    }

    pub fn finish(mut self) {
        unsafe {
            GdipDeleteGraphics(self.g);
            self.g = null_mut();
            SetBkMode(self.hdc, TRANSPARENT as _);
            for (mut r, s, f, c, fl) in self.texts.drain(..) {
                let old = SelectObject(self.hdc, f as _);
                SetTextColor(self.hdc, cref(c));
                let ws = w(&s);
                DrawTextW(self.hdc, ws.as_ptr(), -1, &mut r, fl | DT_NOPREFIX);
                SelectObject(self.hdc, old);
            }
        }
    }
}

pub fn text_width(hdc: HDC, s: &str, font: HFONT) -> i32 {
    unsafe {
        let old = SelectObject(hdc, font as _);
        let ws: Vec<u16> = s.encode_utf16().collect();
        let mut sz: SIZE = std::mem::zeroed();
        GetTextExtentPoint32W(hdc, ws.as_ptr(), ws.len() as i32, &mut sz);
        SelectObject(hdc, old);
        sz.cx
    }
}

pub fn make_font(px: i32, weight: i32, face: &str) -> HFONT {
    let f = w(face);
    unsafe {
        CreateFontW(
            -px,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            CLEARTYPE_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            f.as_ptr(),
        )
    }
}

pub fn rc(l: i32, t: i32, r: i32, b: i32) -> RECT {
    RECT { left: l, top: t, right: r, bottom: b }
}

pub fn contains(r: &RECT, x: i32, y: i32) -> bool {
    x >= r.left && x < r.right && y >= r.top && y < r.bottom
}

pub struct Image {
    pub bmp: *mut GpBitmap,
    pub w: u32,
    pub h: u32,
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe { GdipDisposeImage(self.bmp as *mut GpImage) };
    }
}

pub fn load_image(path: &std::path::Path) -> Option<Image> {
    unsafe {
        let p = w(&path.to_string_lossy());
        let mut bmp: *mut GpBitmap = null_mut();
        if GdipCreateBitmapFromFile(p.as_ptr(), &mut bmp) != 0 || bmp.is_null() {
            return None;
        }
        let (mut iw, mut ih) = (0u32, 0u32);
        GdipGetImageWidth(bmp as *mut GpImage, &mut iw);
        GdipGetImageHeight(bmp as *mut GpImage, &mut ih);
        if iw == 0 || ih == 0 {
            GdipDisposeImage(bmp as *mut GpImage);
            return None;
        }
        Some(Image { bmp, w: iw, h: ih })
    }
}

impl Canvas {
    /// Vẽ ảnh phủ kín khung (cắt bớt nếu khác tỉ lệ), bo góc.
    pub fn image_round(&mut self, img: &Image, x: i32, y: i32, wd: i32, ht: i32, r: i32) {
        unsafe {
            let p = Self::round_path(x, y, wd, ht, r);
            GdipSetClipPath(self.g, p, CombineModeReplace);
            GdipSetInterpolationMode(self.g, InterpolationModeHighQualityBicubic);
            let (iw, ih) = (img.w as f32, img.h as f32);
            let scale = (wd as f32 / iw).max(ht as f32 / ih);
            let (sw, sh) = (wd as f32 / scale, ht as f32 / scale);
            let (sx, sy) = ((iw - sw) / 2.0, (ih - sh) / 2.0);
            GdipDrawImageRectRect(
                self.g,
                img.bmp as *mut GpImage,
                x as f32,
                y as f32,
                wd as f32,
                ht as f32,
                sx,
                sy,
                sw,
                sh,
                UnitPixel,
                null_mut(),
                0,
                null_mut(),
            );
            GdipResetClip(self.g);
            GdipDeletePath(p);
        }
    }
}
