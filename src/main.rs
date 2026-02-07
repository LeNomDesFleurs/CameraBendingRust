// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;

mod buffer;
pub use buffer::DelayLine;

fn main() -> anyhow::Result<()> {
    // Collect all arguments
    let args: Vec<String> = env::args().collect();
    // Path of input file is the first argument
    // let in_path = &args[1];
    // let out_path = &args[2];
    let in_path:&String = &"assets/rose.jpg".to_string();
    let out_path: &String = &"test6.jpg".to_string();
    let feedback = args[3].parse::<f32>()?;
    let delay = args[4].parse::<f32>()?;
    // let feedback = 0.7;
    let dynimg = image::open(in_path)?;

    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", dynimg.dimensions());
    let mut filter_r = Biquad::new(FilterType::LPF);
    let mut filter_g = Biquad::new(FilterType::LPF);
    let mut filter_b = Biquad::new(FilterType::LPF);

    let mut buf = DelayLine::new(1.0, buffer::DelayMode::Comb);
    buf.init(delay);
    buf.init(delay);
    buf.init(delay);
    filter_r.init(44000.0);
    filter_g.init(30000.0);
    filter_b.init(40000.0);

    // let mut buf: Vec<u8> = Vec::new();
    buf.set_feedback(feedback);

    let mut bufimg = dynimg.into_rgb8();

    for (pixel) in bufimg.enumerate_pixels_mut() {
        // let mut r = buffer.process(pixel[0]as f32) as u8;
        // let mut g = buffer.process(pixel[1]as f32) as u8;
        // let mut b = buffer.process(pixel[2] as f32) as u8;
        let r = buf.process(pixel.2[0] as f32) as u8;
        let g = buf.process(pixel.2[1] as f32) as u8;
        let b = buf.process(pixel.2[2] as f32) as u8;

        // r = filter_r.process(r as f32) as u8;
        // g = filter_g.process(g as f32) as u8;
        // b = filter_b.process(b as f32) as u8;
        *pixel.2 = image::Rgb([r, g, b]);
    }

    // Write the contents of this image to the Writer in PNG format.
    bufimg.save(out_path).unwrap();
    Ok(())
}
