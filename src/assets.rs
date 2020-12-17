use android_ndk;
use image::{self, DynamicImage, ImageResult};
use conrod_core::text::{FontCollection, Font};
use std::ffi::{CString, CStr};
use std::io::Read;
use ndk::asset::{AssetManager, Asset};
use ndk::native_activity::NativeActivity;
use std::error::Error;
//use android_ndk::asset::Asset;
//use android_ndk::native_activity::NativeActivity;

fn aset2data_raw(asset: &mut Asset) -> Vec<u8> {
    let mut data = vec![];
    asset.read_to_end(&mut data);
    data
}

fn str2cstring(s: &str) -> &CStr {
    CStr::from_bytes_with_nul(s.as_bytes()).unwrap()
}

pub fn load_font(native_activity: &NativeActivity, filename: &str) -> Font {
    eprintln!("Call to load_font(..)");
    let am = native_activity.asset_manager();
    match am.open(str2cstring(filename) ) {
        Some(mut asset) => FontCollection::from_bytes(aset2data_raw(&mut asset))
            .unwrap()
            .into_font()
            .expect(&format!("Unable to select single font from collection of '{}'", filename)),
        None => panic!("Can't load font.")
    }
}

pub fn load_image(native_activity: &NativeActivity, filename: &str) -> ImageResult<DynamicImage> {
    eprintln!("Call to load_image(..)");
    let am = native_activity.asset_manager();
    match am.open(str2cstring(filename)) {
        Some(mut asset) => image::load_from_memory(&aset2data_raw(&mut asset)),
        None => panic!("Can't load image.")
    }
}
