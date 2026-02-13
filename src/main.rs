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
    let in_path = &args[1];
    let out_path = &args[2];
    // let in_path: &String = &"assets/rose.jpg".to_string();
    // let out_path: &String = &"test6.jpg".to_string();
    let feedback = args[3].parse::<f32>()?;
    let delay = args[4].parse::<f32>()?;
    // let feedback = 0.9;
    // let delay = 1.0;

    let dynimg = image::open(in_path)?;

    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", dynimg.dimensions());
    let mut filter_r = Biquad::new(FilterType::LPF);
    let mut filter_g = Biquad::new(FilterType::LPF);
    let mut filter_b = Biquad::new(FilterType::LPF);

    let mut bufr = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let mut bufg = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let mut bufb = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    bufr.init_delay(1.0, delay);

    filter_r.init(44000.0);
    filter_r.set_frequence_and_resonance(6000.0, 00.);

    // bufr.set_feedback(0.0);

    let mut bufimg = dynimg.into_rgb8();

    let width = bufimg.dimensions().0;
    let height = bufimg.dimensions().1;
    let size = width * height;

    let mut progress: u32 = 0;
    let mut norm_progress: f32 = 0.0;
    let mut modulation: f32 = 0.0;

    let bayer_matrix = [[1, 2], [0, 1]];
    let mut signal = vec![0.0, 0.0];
    let mut columns: Vec<Vec<f32>> = vec![vec![0.0, 0.0]; width as usize];

    for pixel in bufimg.enumerate_pixels_mut() {
        //update the buffer for the dematricing
        let color = bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];
        // apply filter to the new pixel and push it on the buffer
        columns[pixel.0 as usize].push(pixel.2[color] as f32);
        // signal.push(pixel.2[color] as f32);
    }

    // for (idx, sample) in signal.iter_mut().enumerate() {
    //     *sample = filter_r.process(*sample);
    // }

    for column in columns.iter_mut() {
        let mut prev_sample = 0.0;
        for sample in column.iter_mut() {
            let temp = prev_sample * feedback + *sample;
            *sample = temp;
            prev_sample = temp;
        }
        for idx in 0..1000 {
            bufr.process(0.0);
        }
    }

    for pixel in bufimg.enumerate_pixels_mut() {
        let color = bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];

        // Bayer reconstruction
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let y = pixel.1.saturating_sub(1);
        let x = pixel.0.saturating_sub(1);
        // match color {
        //     0 => {
        //         r = columns[x as usize][y as usize] as u8;
        //         g = 0;
        //         b = 0;
        //     }
        //     1 => {
        //         r = 0;
        //         g = columns[x as usize][y as usize] as u8;
        //         b = 0;
        //     }
        //     2 => {
        //         r = 0;
        //         g = 0;
        //         b = columns[x as usize][y as usize] as u8;
        //     }
        //     _ => {}
        // }

        // r = columns[x as usize][y as usize] as u8;
        // g = columns[x as usize][y as usize] as u8;
        // b = columns[x as usize][y as usize] as u8;

        match color {
            0 => {
                r = columns[x as usize][y as usize] as u8;
                g = ((columns[x.saturating_sub(1) as usize][y as usize]
                    + columns[(x + 1) as usize][y as usize]
                    + columns[x as usize][y.saturating_sub(1) as usize]
                    + columns[x as usize][(y + 1) as usize])
                    / 4.0) as u8;
                b = ((columns[x.saturating_sub(1) as usize][y.saturating_sub(1) as usize]
                    + columns[(x + 1) as usize][(y + 1) as usize]
                    + columns[(x + 1) as usize][(y.saturating_sub(1)) as usize]
                    + columns[x.saturating_sub(1) as usize][(y + 1) as usize])
                    / 4.0) as u8;
            }
            1 => {
                g = columns[x as usize][y as usize] as u8;
                if y % 2 == 0 {
                    b = ((columns[x.saturating_sub(1) as usize][(y) as usize]
                        + columns[(x + 1) as usize][(y) as usize])
                        / 2.0) as u8;
                    r = ((columns[(x) as usize][(y.saturating_sub(1)) as usize]
                        + columns[(x) as usize][(y + 1) as usize])
                        / 2.0) as u8;
                } else {
                    r = ((columns[x.saturating_sub(1) as usize][(y) as usize]
                        + columns[(x + 1) as usize][(y) as usize])
                        / 2.0) as u8;
                    b = ((columns[(x) as usize][y.saturating_sub(1) as usize]
                        + columns[(x) as usize][(y + 1) as usize])
                        / 2.0) as u8;
                }
            }
            // B G B    R G R
            // G R G or G B G
            // B G B    R G R
            2 => {
                r = ((columns[x.saturating_sub(1) as usize][y.saturating_sub(1) as usize]
                    + columns[(x + 1) as usize][(y + 1) as usize]
                    + columns[(x + 1) as usize][(y.saturating_sub(1)) as usize]
                    + columns[x.saturating_sub(1) as usize][(y + 1) as usize])
                    / 4.0) as u8;
                g = ((columns[x.saturating_sub(1) as usize][(y) as usize]
                    + columns[(x + 1) as usize][(y) as usize]
                    + columns[(x) as usize][y.saturating_sub(1) as usize]
                    + columns[(x) as usize][(y + 1) as usize])
                    / 4.0) as u8;
                b = columns[x as usize][y as usize] as u8;
            }
            _ => {}
        }

        // match color {
        //     // Red
        //     0 => {
        //         r = signal[((x * width) + y) as usize] as u8;
        //         g = ((signal[((x.saturating_sub(1)) * width + (y)) as usize]
        //             + signal[((x + 1) * width + (y)) as usize]
        //             + signal[((x) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x) * width + (y + 1)) as usize])
        //             / 4.0) as u8;
        //         b = ((signal[((x.saturating_sub(1)) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x + 1) * width + (y + 1)) as usize]
        //             + signal[((x + 1) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x.saturating_sub(1)) * width + (y + 1)) as usize])
        //             / 4.0) as u8;
        //     }
        //     1 => {
        //         g = signal[((x * width) + y) as usize] as u8;
        //         if y % 2 == 0 {
        //             b = ((signal[((x.saturating_sub(1)) * width + (y)) as usize]
        //                 + signal[((x + 1) * width + (y)) as usize])
        //                 / 2.0) as u8;
        //             r = ((signal[((x) * width + (y.saturating_sub(1))) as usize]
        //                 + signal[((x) * width + (y + 1)) as usize])
        //                 / 2.0) as u8;
        //         } else {
        //             r = ((signal[((x.saturating_sub(1)) * width + (y)) as usize]
        //                 + signal[((x + 1) * width + (y)) as usize])
        //                 / 2.0) as u8;
        //             b = ((signal[((x) * width + (y.saturating_sub(1))) as usize]
        //                 + signal[((x) * width + (y + 1)) as usize])
        //                 / 2.0) as u8;
        //         }
        //     }
        //     // B G B    R G R
        //     // G R G or G B G
        //     // B G B    R G R
        //     2 => {
        //         r = ((signal[((x.saturating_sub(1)) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x + 1) * width + (y + 1)) as usize]
        //             + signal[((x + 1) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x.saturating_sub(1)) * width + (y + 1)) as usize])
        //             / 4.0) as u8;
        //         g = ((signal[((x.saturating_sub(1)) * width + (y)) as usize]
        //             + signal[((x + 1) * width + (y)) as usize]
        //             + signal[((x) * width + (y.saturating_sub(1))) as usize]
        //             + signal[((x) * width + (y + 1)) as usize])
        //             / 4.0) as u8;
        //         b = signal[((x * width) + y) as usize] as u8;
        //     }
        //     _ => {}
        // }

        *pixel.2 = image::Rgb([r, g, b]);
    }

    // Write the contents of this image to the Writer in PNG format.
    bufimg.save(out_path).unwrap();
    Ok(())
}
