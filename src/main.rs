// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;

mod buffer;
pub use buffer::DelayLine;

#[derive(Clone, Copy, PartialEq, Eq)]


fn main() -> anyhow::Result<()> {
    // Collect all arguments
    let args: Vec<String> = env::args().collect();
    // Path of input file is the first argument
    let in_path = &args[1];
    let out_path = &args[2];
    // let in_path:&String = &"assets/rose.jpg".to_string();
    // let out_path: &String = &"test6.jpg".to_string();
    let feedback = args[3].parse::<f32>()?;
    let delay = args[4].parse::<f32>()?;
    // let feedback = 0.7;

    let dynimg = image::open(in_path)?;

    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", dynimg.dimensions());
    let mut filter_r = Biquad::new(FilterType::LPF);
    let mut filter_g = Biquad::new(FilterType::LPF);
    let mut filter_b = Biquad::new(FilterType::LPF);

    let mut bufr = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let mut bufg = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let mut bufb = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    bufr.init(1.0);
    bufr.set_delay_time(1.0);
    bufg.init(1.0);
    bufb.init(1.0);

    filter_r.init(44000.0);
    filter_r.set_frequence_and_resonance(6000.0, 200.);
    filter_g.init(30000.0);
    filter_b.init(40000.0);

    // let mut buf: Vec<u8> = Vec::new();
    bufr.set_feedback(0.0);
    bufg.set_feedback(0.0);
    bufb.set_feedback(0.0);

    let mut bufimg = dynimg.into_rgb8();

    let width = bufimg.dimensions().0;
    let height = bufimg.dimensions().1;
    let size = width * height;
    let mut progress: u32 = 0;
    let mut norm_progress: f32 = 0.0;
    let mut modulation: f32 = 0.0;


    let mut samples  = vec![0.0; 5];
    let mut colors = vec![0; 5];
    let bayer_matrix = [[1, 0], [2, 1]];


    for (pixel) in bufimg.enumerate_pixels_mut() {
       


        progress = progress + 1;
        norm_progress = (progress as f32) / (size as f32);
        modulation = (norm_progress * 500.0).sin().abs();
        let modulationr = ((norm_progress * 5.0) + std::f32::consts::FRAC_PI_4)
            .sin()
            .abs();
        let modulationg = ((norm_progress * 5.0) + std::f32::consts::FRAC_PI_2)
            .sin()
            .abs();
        let modulationb = (norm_progress * 5.0).sin().abs();

        bufr.set_delay_time(500.0 + (10.0 * modulation));
        bufg.set_delay_time(500.0 + (10.0 * modulation));
        bufb.set_delay_time(500.0 + (10.0 * modulation));

        // bufr.set_feedback(modulationr);
        bufg.set_feedback(modulationg);
        bufb.set_feedback(modulationb);

        filter_r.set_frequence_and_resonance(1000.+900.0 * modulationr, 300.+200.*modulationg);
        filter_g.set_frequence_and_resonance(3000.+900.0 * modulationr, 500.+200.*modulationg);
        filter_b.set_frequence_and_resonance(2000.+300.0 * modulationr, 100.+200.*modulationg);
        // let mut r = buffer.process(pixel[0]as f32) as u8;
        // let mut g = buffer.process(pixel[1]as f32) as u8;
        // let mut b = buffer.process(pixel[2] as f32) as u8;
        // let r = bufr.process(filter_r.process(pixel.2[0] as f32)) as u8;
        // let g = bufg.process(filter_r.process(pixel.2[1] as f32)) as u8;
        // let b = bufb.process(filter_r.process(pixel.2[2] as f32)) as u8;

        let r = bufr.process((pixel.2[0] as f32)) as u8;
        let g = bufg.process((pixel.2[1] as f32)) as u8;
        let b = bufb.process((pixel.2[2] as f32)) as u8;
        // r = filter_r.process(r as f32) as u8;
        // g = filter_g.process(g as f32) as u8;
        // b = filter_b.process(b as f32) as u8;
        *pixel.2 = image::Rgb([r, g, b]);
    }

    // Write the contents of this image to the Writer in PNG format.
    bufimg.save(out_path).unwrap();
    Ok(())
}
