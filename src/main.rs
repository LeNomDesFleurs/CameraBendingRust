// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;

mod buffer;
pub use buffer::DelayLine;

fn shaper(in_value: f32, min: f32, max: f32, curve: f32)->f32{
    //normalize to [0 ; 1]
    let mut value = (in_value-min)/(max-min);
    value = ((curve*value).exp()-1.0)/((curve).exp()-1.0);
    value = (value * max)+min;
    value
}

struct Random{
    seed:f32,
    min:f32,
    max:f32,
    shaped_value:f32,
    m:f32,
    smoothing:f32,
}

impl Random{

pub fn new()->Random
{
    Random { seed: 123456789.0, min: 0.0, max: 1.0 , shaped_value: 0.0, m: 126379272.0, smoothing: 0.0}
}

pub fn new_min_max(min:f32, max:f32, smoothing:f32)->Random
{
    Random { seed: 123456789.0, min, max, shaped_value: 0.0, m: 126379272.0, smoothing}
}

pub fn process(&mut self)->f32{
    self.seed = (1103515245.0 * self.seed + 5.0) % self.m;
    let prev_value = self.shaped_value;
    self.shape();
    self.shaped_value = (self.shaped_value * (1.0-self.smoothing)) + (prev_value * (self.smoothing));
    return self.shaped_value;
}

fn shape(&mut self){
    self.shaped_value = self.seed / self.m;
    self.shaped_value = self.shaped_value * self.max;
    self.shaped_value = self.shaped_value + self.min;
}

}


fn rand(seed:f32)->f32
{
  let seed = (1103515245.0 * seed + 5.0) % 126379272.0;
  return seed;
}

fn main() -> anyhow::Result<()> {
    // Collect all arguments
    let args: Vec<String> = env::args().collect();
    
    // Path of input file is the first argument
    let in_path = &args[1];
    let out_path = &args[2];
    let feedback = args[3].parse::<f32>()?;
    let delay = args[4].parse::<f32>()?;

    // debug values
    // let out_path: &String = &"test14.jpg".to_string();
    // let in_path: &String = &"assets/rose.jpg".to_string();
    // let feedback = 0.9;
    // let delay = 1.0;

    let dynimg = image::open(in_path)?;

    
    let mut bufr = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let _bufg = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    let _bufb = DelayLine::new(1000.0, buffer::DelayMode::Comb);
    bufr.init_delay(1.0, delay);
    // The dimensions method returns the images width and height.
    // println!("dimensions {:?}", dynimg.dimensions());
    let mut filter_r = Biquad::new(FilterType::LPF);
    let mut filter_g = Biquad::new(FilterType::PEAK);
    let filter_b = Biquad::new(FilterType::LPF);
    
    filter_r.init(44000.0);
    filter_r.set_frequence_and_resonance(3000.0, 7.0);
    
    filter_g.init(44000.0);
    filter_g.set_frequence_and_resonance(2000.0, 100.0);
    

    // bufr.set_feedback(0.0);

    let mut bufimg = dynimg.into_rgb8();

    let width = bufimg.dimensions().0;
    let height = bufimg.dimensions().1;
    let _size = width * height;

    let _progress: u32 = 0;
    let _norm_progress: f32 = 0.0;
    let _modulation: f32 = 0.0;

    let bayer_matrix = [[1, 0], [2, 1]];
    let _signal = vec![0.0, 0.0];
    let mut columns: Vec<Vec<f32>> = vec![vec![0.0, 0.0]; width as usize];

    let mut random_generator = Random::new_min_max(0.0, 1.0, 0.99);

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
        let local_feedback = 0.99 + random_generator.process()/100.0;
        
        for sample in column.iter_mut() {
            let mut temp = prev_sample * (local_feedback) + *sample * (1.0-local_feedback);
            // if prev_sample < 50.0 {
            //     temp = *sample;
            // }

            *sample = filter_r.process(*sample);
            // *sample = filter_g.process(*sample);
            prev_sample = temp; // shaper(temp, 0.0, 255.0, 0.001);
        }

        for idx in 0..100{
            filter_r.process(0.0);
        }
        for sample in column.iter_mut().rev() {
            let mut temp = prev_sample * (local_feedback) + *sample * (1.0-local_feedback);
            // if prev_sample < 50.0 {
            //     temp = *sample;
            // }

            *sample = filter_r.process(*sample);
            // *sample = filter_g.process(*sample);
            prev_sample = temp; // shaper(temp, 0.0, 255.0, 0.001);
        }

        for idx in 0..100{
            filter_r.process(0.0);
        }

        // douple pass in reverse
        // for sample in column.iter_mut().rev() {
        //     let mut temp = prev_sample * feedback + *sample * (1.0-feedback);
        //     if prev_sample < 50.0 {
        //         temp = *sample;
        //     }
        //     *sample = temp;
        //     prev_sample = temp;
        // }
        for _idx in 0..1000 {
            bufr.process(0.0);
        }
    }

    for pixel in bufimg.enumerate_pixels_mut() {
        let color = bayer_matrix[((pixel.0+1) % 2) as usize][((pixel.1+1) % 2) as usize];

        // Bayer reconstruction
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;
        let y = pixel.1.saturating_sub(1) as usize;
        let x = pixel.0.saturating_sub(1) as usize;
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
                r = columns[x][y] as u8;
                g = ((columns[x.saturating_sub(1)][y]
                    + columns[x + 1][y]
                    + columns[x][y.saturating_sub(1)]
                    + columns[x][y + 1])
                    / 4.0) as u8;
                b = ((columns[x.saturating_sub(1)][y.saturating_sub(1)]
                    + columns[x + 1][y + 1]
                    + columns[x + 1][y.saturating_sub(1)]
                    + columns[x.saturating_sub(1)][y + 1])
                    / 4.0) as u8;
            }
            1 => {
                g = columns[x][y] as u8;
                if y % 2 == 0 {
                    b = ((columns[x.saturating_sub(1)][y] + columns[x + 1][y]) / 2.0) as u8;
                    r = ((columns[x][y.saturating_sub(1)] + columns[x][y + 1]) / 2.0) as u8;
                } else {
                    r = ((columns[x.saturating_sub(1)][y] + columns[x + 1][y]) / 2.0) as u8;
                    b = ((columns[x][y.saturating_sub(1)] + columns[x][y + 1]) / 2.0) as u8;
                }
            }
            // B G B    R G R
            // G R G or G B G
            // B G B    R G R
            2 => {
                r = ((columns[x.saturating_sub(1)][y.saturating_sub(1)]
                    + columns[x + 1][y + 1]
                    + columns[x + 1][y.saturating_sub(1)]
                    + columns[x.saturating_sub(1)][y + 1])
                    / 4.0) as u8;
                g = ((columns[x.saturating_sub(1)][y]
                    + columns[x + 1][y]
                    + columns[x][y.saturating_sub(1)]
                    + columns[x][y + 1])
                    / 4.0) as u8;
                b = columns[x][y] as u8;
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
