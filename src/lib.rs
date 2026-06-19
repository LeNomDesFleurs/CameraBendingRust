// use time::now;

mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;
mod reverb;
pub use reverb::Reverb;

mod processor;
pub use processor::Picture;
pub use processor::Processor;

mod buffer;
pub use buffer::DelayLine;
#[cfg(feature = "web-sys")]
#[cfg(feature = "enable_wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm-bindgen")]
use wasm_bindgen::Clamped;

#[cfg(feature = "web-sys")]
use web_sys::{Blob, CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement, ImageData};

use web_sys::console;


use crate::processor::Parameters;

#[cfg_attr(feature = "enable_wasm", wasm_bindgen)]
pub fn process_picture(
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    alpha_mode: u32,
    color_mode: u32,
    order_mode: u32,
    delay_time: f32,
    delay_feedback: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    reverb_dry_wet: f32,
    reverb_decay: f32,
    continuous: bool,
){
    let w = canvas.width();
    let h = canvas.height();
    let imgdata = ctx.get_image_data(0.0, 0.0, w as f64, h as f64).unwrap();
    let raw_pixels = imgdata.data().to_vec();
    let mut picture = Picture::new(raw_pixels, w as usize, h as usize);
    console::log_1(&format!("width: {}, height: {}", canvas.width(), canvas.height()).into());
    
    let parameters = Parameters::new(
        alpha_mode,
        color_mode,
        order_mode,
        delay_time,
        delay_feedback,
        filter_cutoff,
        filter_resonance,
        reverb_dry_wet,
        reverb_decay,
        continuous,
    );

    


    let mut processor = Processor::new(picture, parameters);
    let picture = processor.process_image();
    // console::log_1(&format!("output picture - width: {}, height: {}, len: {}", picture.width, picture.height, picture.get_lenght()).into());
    // let raw_pixels = picture.get_raw_data();
    putImageData(canvas, ctx, picture);

    }

/// Place a PhotonImage onto a 2D canvas.
#[cfg(all(feature = "web-sys", feature = "wasm-bindgen"))]
#[cfg_attr(feature = "enable_wasm", wasm_bindgen)]
#[allow(non_snake_case)]
#[allow(clippy::unnecessary_mut_passed)]
pub fn putImageData(canvas: HtmlCanvasElement, ctx: CanvasRenderingContext2d, new_image: Picture) {
    // Convert the raw pixels back to an ImageData object.
    // let mut test_picture = vec![255 as u8; 4000000];
    assert_eq!((canvas.width()* canvas.height()*4) as usize, new_image.get_lenght());
    let mut raw_pixels = new_image.get_raw_data();
    let new_img_data = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(&mut raw_pixels),
        // Clamped(&mut raw_pixels),
        canvas.width(),
        canvas.height(),
    );

    // Place the new imagedata onto the canvas
    ctx.put_image_data(&new_img_data.unwrap(), 0.0, 0.0)
        .expect("Should put image data on Canvas");
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_set_and_get() {
        let width = 1000;
        let height = 1000;
        let size = width + height *4;
        let raw_data = vec![255 as u8; size];
        let mut picture = Picture::new(raw_data, width, height);
        let pixel = [0, 0, 0, 0];
        picture.set_pixel(750, 750, pixel);
        let output_pixel = picture.get_pixel(750, 750);
        assert_eq!(pixel, output_pixel);
    }
}