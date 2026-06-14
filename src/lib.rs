// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;
mod reverb;
use image::ImageBuffer;
use image::Rgba;
pub use reverb::Reverb;

mod parameters;
pub use parameters::Parameters;
mod processor;
pub use processor::Processor;

mod buffer;
pub use buffer::DelayLine;

#[cfg_attr(feature = "enable_wasm", wasm_bindgen)]
fn process_picture(input_image : ImageBuffer<Rgba<u8>, Vec<u8>> ) -> ImageBuffer<Rgba<u8>, Vec<u8>> 
{
    // Collect all arguments
    let args: Vec<String> = env::args().collect();

    let mut parameters = Parameters::new();
    let mut processor = Processor::new(input_image, &parameters);


    return processor.process_image(&mut parameters);
}