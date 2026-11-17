use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::GdiPlus::*;
use windows_sys::Win32::UI::Controls::Dialogs::*;

use crate::config::Config;
use crate::gfx::{load_image, Image, PIXEL_FORMAT_32BPP_ARGB};
use crate::util::{from_w, profiles_dir, w};

const SIZE: i32 = 96;
const MAX_SOURCE_BYTES: u64 = 20 * 1024 * 1024;
const PNG_ENCODER: GUID = GUID::from_u128(0x557cf406_1a04_11d3_9a73_0000f81ef32e);

pub fn path(id: &str) -> PathBuf {
    profiles_dir().join(id).join("avatar.png")
}

pub fn load(id: &str) -> Option<Image> {
    let p = path(id);
    if p.is_file() {
        load_image(&p)
    } else {
        None
    }
}

pub fn load_all(cfg: &Config) -> HashMap<String, Image> {
    cfg.accounts.iter().filter_map(|a| load(&a.id).map(|img| (a.id.clone(), img))).collect()
}

pub fn remove(id: &str) {
    let _ = std::fs::remove_file(path(id));
}

pub fn pick_file(owner: HWND) -> Option<PathBuf> {
    let mut file = [0u16; 1024];
    let filter: Vec<u16> =
        "Ảnh (*.png;*.jpg;*.jpeg;*.bmp;*.gif)\0*.png;*.jpg;*.jpeg;*.bmp;*.gif\0\0".encode_utf16().collect();
    let title = w("Chọn ảnh đại diện");
    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = owner;
    ofn.lpstrFilter = filter.as_ptr();
    ofn.lpstrFile = file.as_mut_ptr();
    ofn.nMaxFile = file.len() as u32;
    ofn.lpstrTitle = title.as_ptr();
    ofn.Flags = OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    if unsafe { GetOpenFileNameW(&mut ofn) } == 0 {
        return None;
    }
    Some(PathBuf::from(from_w(&file)))
}

pub fn import(src: &Path, id: &str) -> Result<(), String> {
    let size = std::fs::metadata(src).map_err(|e| e.to_string())?.len();
    if size > MAX_SOURCE_BYTES {
        return Err("Ảnh quá lớn (tối đa 20 MB).".into());
    }
    let out_path = path(id);
    std::fs::create_dir_all(out_path.parent().unwrap()).map_err(|e| e.to_string())?;
    let tmp = out_path.with_extension("tmp");

    unsafe {
        let src_w = w(&src.to_string_lossy());
        let mut img: *mut GpBitmap = null_mut();
        if GdipCreateBitmapFromFile(src_w.as_ptr(), &mut img) != 0 || img.is_null() {
            return Err("Không đọc được ảnh này.".into());
        }
        let (mut iw, mut ih) = (0u32, 0u32);
        GdipGetImageWidth(img as *mut GpImage, &mut iw);
        GdipGetImageHeight(img as *mut GpImage, &mut ih);
        if iw == 0 || ih == 0 {
            GdipDisposeImage(img as *mut GpImage);
            return Err("Không đọc được ảnh này.".into());
        }

        let mut out: *mut GpBitmap = null_mut();
        GdipCreateBitmapFromScan0(SIZE, SIZE, 0, PIXEL_FORMAT_32BPP_ARGB, null_mut(), &mut out);
        let mut g: *mut GpGraphics = null_mut();
        GdipGetImageGraphicsContext(out as *mut GpImage, &mut g);
        GdipSetInterpolationMode(g, InterpolationModeHighQualityBicubic);
        GdipSetPixelOffsetMode(g, PixelOffsetModeHalf);

        let side = iw.min(ih) as f32;
        let sx = (iw as f32 - side) / 2.0;
        let sy = (ih as f32 - side) / 2.0;
        GdipDrawImageRectRect(
            g,
            img as *mut GpImage,
            0.0,
            0.0,
            SIZE as f32,
            SIZE as f32,
            sx,
            sy,
            side,
            side,
            UnitPixel,
            null_mut(),
            0,
            null_mut(),
        );
        GdipDeleteGraphics(g);
        GdipDisposeImage(img as *mut GpImage);

        let tmp_w = w(&tmp.to_string_lossy());
        let st = GdipSaveImageToFile(out as *mut GpImage, tmp_w.as_ptr(), &PNG_ENCODER, null_mut());
        GdipDisposeImage(out as *mut GpImage);
        if st != 0 {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("Không lưu được ảnh (mã lỗi GDI+ {st})."));
        }
    }
    std::fs::rename(&tmp, &out_path).map_err(|e| e.to_string())
}
