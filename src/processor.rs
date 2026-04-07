use crossterm::style::Color;
use image::ImageBuffer;
use image::Rgb;
use image::Rgba;

use crate::buffer;
pub use crate::buffer::DelayLine;
pub use crate::filter::Biquad;
pub use crate::filter::FilterType;
pub use crate::parameters::{AlphaMode, ColorMode, OrderMode, Parameters};

// pub use crate::outils;

#[derive(Clone, PartialEq)]
enum Flag {
    Reset,
    Continue,
}

//TODO add transparency Layer ?
//might create some really cool things
enum Signal<T> {
    // slice 1 (RGBRGBRGB) slice 2 (RGBRGBRGB) ...
    InterleavedArray(Vec<Vec<T>>),
    // one line RGB RGB RGB RGB
    InterleavedVector(Vec<T>),
    // slice 1 (RRRRRR) slice 2 (RRRRRR) .... slice 1 (GGGGGG) slice 2 (GGGGG) ...
    CompositeArray([Vec<Vec<T>>; 4]),
    // 1 slice for each color channel
    CompositeVector([Vec<T>; 4]),
}

pub struct Processor {
    parameters: Parameters,

    //-------------processing-----------------
    quantization: f32,

    ordered_picture: Vec<[u8; 4]>,
    signal: Vec<(f32, Flag)>,
    processed_picture: Vec<f32>,
    slices: Vec<Vec<f32>>,
    // signal: Signal<f32>,
    filter: Biquad,
    delay: DelayLine,

    // ---------------file----------
    width: u32,
    height: u32,
    size: u32,

    source_image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,

    destination_image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,

    bayer_matrix: [[i32; 2]; 2],

    path: String,
}

impl Processor {
    pub fn new(in_path: &str, parameters: &Parameters) -> Self {
        let dynimg = image::open(in_path).unwrap();
        let mut bufimg = dynimg.into_rgba8();
        let height = bufimg.dimensions().1;
        let width = bufimg.dimensions().0;
        let mut new = Self {
            parameters: Parameters::new(),
            quantization: 0.0,
            // signal: Signal::InterleavedVector(vec![0.0 as f32, 0.0 as f32]),
            signal: Vec::new(),
            filter: Biquad::new(FilterType::LPF),
            delay: DelayLine::new(1000.0, buffer::DelayMode::Comb),
            bayer_matrix: [[1, 2], [0, 1]],
            ordered_picture: Vec::new(),
            processed_picture: Vec::new(),
            slices: Vec::new(),
            source_image_buffer: bufimg.clone(),
            destination_image_buffer: bufimg.clone(),
            width: width,
            height: height,
            size: width * height,
            path: in_path.to_string(),
        };
        new.init();
        new
    }

    pub fn init(&mut self) {
        self.filter.init(44000.0);
        self.delay.init_delay(1.0, 0.0);
    }

    pub fn set_filters(&mut self) {
        self.filter.set_frequence_and_resonance(
            self.parameters.filter_cutoff.get() as f32,
            self.parameters.filter_resonance.value as f32,
        )
    }

    pub fn set_delay(&mut self) {
        self.delay
            .set_delay_time(self.parameters.delay_time.get() as f32);
        self.delay
            .set_feedback(self.parameters.delay_time.get() as f32 / 1000.0);
    }

    fn order_signal(&mut self) {
        self.ordered_picture.clear();
        match self.parameters.order_mode.get() {
            OrderMode::Row => {
                for pixel in self.source_image_buffer.enumerate_pixels_mut() {
                    self.ordered_picture
                        .push([pixel.2[0], pixel.2[1], pixel.2[2], pixel.2[3]]);
                }
            }
            OrderMode::Column => {}
            OrderMode::ReverseRow => {}
            OrderMode::ReverseColumn => {}
        }
    }

    pub fn bayer_dematricing() {}

    fn reset_processing(&mut self) {
        self.filter.flush();
        self.delay
            .init_delay(1.0, self.parameters.delay_time.get() as f32);
    }

    fn process_sample(&mut self, sample: f32) -> f32 {
        let mut temp = sample;
        temp = self.filter.process(temp);
        temp = self.delay.process(temp);
        return temp;
    }

    fn split_colors(&mut self) {
        self.signal.clear();
        let mut count = 0;
        let mut color_layer = 3;
        if *self.parameters.alpha_mode.get() == AlphaMode::Interleave {
            color_layer = 4
        };
        let mut modulo = 0;
        match self.parameters.order_mode.get(){
            OrderMode::Column=>{modulo = self.height}
            OrderMode::Row=>{modulo = self.width}
            _=>{}
        }
        match self.parameters.color_mode.get() {
            ColorMode::Bayer => {}
            ColorMode::Interleaved => {
                for pixel in self.ordered_picture.clone() {
                    let mut flag = Flag::Continue;
                    if self.parameters.continuous.value == false && count % modulo == 0{
                            flag = Flag::Reset
                        }
                    for i in 0..color_layer {
                        self.signal.push((pixel[i] as f32, flag));
                        flag = Flag::Continue //dirty
                    }

                    count = count + 1;
                }
            }
            ColorMode::Composite => {
                for i in 0..color_layer {
                    for pixel in self.ordered_picture.clone() {

                        let mut flag = Flag::Continue;
                        if self.parameters.continuous.value == false && count % modulo == 0{
                            flag = Flag::Reset
                        }
                        self.signal.push((pixel[i] as f32, flag));
                        count = count + 1;
                    }
                }
            }
        }
    }


    fn process_signal(&mut self) {
        self.processed_picture.clear();

        for (pixel, flag) in self.signal.clone() {
            if flag==Flag::Reset{
                self.reset_processing();
            }
            let temp = self.process_sample(pixel);
            self.processed_picture.push(temp);
        }
    }

    pub fn process_image(&mut self, parameters: &Parameters) {
        self.parameters = *parameters;

        self.set_delay();
        self.set_filters();

        self.order_signal(); // Vec<Vec<rgb>>
        self.split_colors(); // Vec<f32>
        self.process_signal(); // Vec<f32>
        self.reconstruct_image();
        self.make_file();
    }

    pub fn reconstruct_image(&mut self) {
        let mut count = 0;
        let len = self.processed_picture.len();
        let destination_len = self.destination_image_buffer.len();
        let mut number_of_channels = 3;
        if *self.parameters.alpha_mode.get() == AlphaMode::Interleave {
            number_of_channels = 4
        };
        let offset = len / number_of_channels;
        match self.parameters.order_mode.get() {
            OrderMode::Row => {
                for pixel in self.destination_image_buffer.enumerate_pixels_mut() {
                    let y = pixel.1.saturating_sub(1) as usize;
                    let x = pixel.0.saturating_sub(1) as usize;

                    match self.parameters.color_mode.get() {
                        ColorMode::Interleaved => {
                            let r = self.processed_picture[count] as u8;
                            let g = self.processed_picture[count + 1] as u8;
                            let b = self.processed_picture[count + 2] as u8;
                            let mut a = 127;

                            match self.parameters.alpha_mode.get() {
                                AlphaMode::Delete => {}
                                AlphaMode::Interleave => {
                                    a = self.processed_picture[count + 3] as u8;
                                }
                                AlphaMode::Preserve => {
                                    a = self.source_image_buffer[(x as u32, y as u32)].0[3];
                                }
                            }

                            *pixel.2 = image::Rgba([r, g, b, a]);
                            count = count + 1;
                        }
                        ColorMode::Bayer => {
                            let color =
                                self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];
                        }
                        ColorMode::Composite => {
                            let r = self.processed_picture[count] as u8;
                            let g = self.processed_picture[count + offset] as u8;
                            let b = self.processed_picture[count + (offset * 2)] as u8;
                            let mut a = 127;

                            match self.parameters.alpha_mode.get() {
                                AlphaMode::Delete => {}
                                AlphaMode::Interleave => {
                                    a = self.processed_picture[count + (offset * 3)] as u8;
                                }
                                AlphaMode::Preserve => {
                                    a = self.source_image_buffer[(x as u32, y as u32)].0[3];
                                }
                            }

                            *pixel.2 = image::Rgba([r, g, b, a]);
                            count = count + 1;
                        }
                    }
                }
            }
            OrderMode::Column => {}
            _ => {}
        }
    }

    pub fn make_file(&self) {
        // Write the contents of this image to the Writer in PNG format.

        self.destination_image_buffer.save("test.png").unwrap();
    }
}

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

// match color {
//     0 => {
//         r = columns[x][y] as u8;
//         g = ((columns[x.saturating_sub(1)][y]
//             + columns[x + 1][y]
//             + columns[x][y.saturating_sub(1)]
//             + columns[x][y + 1])
//             / 4.0) as u8;
//         b = ((columns[x.saturating_sub(1)][y.saturating_sub(1)]
//             + columns[x + 1][y + 1]
//             + columns[x + 1][y.saturating_sub(1)]
//             + columns[x.saturating_sub(1)][y + 1])
//             / 4.0) as u8;
//     }
//     1 => {
//         g = columns[x][y] as u8;
//         if y % 2 == 0 {
//             b = ((columns[x.saturating_sub(1)][y] + columns[x + 1][y]) / 2.0) as u8;
//             r = ((columns[x][y.saturating_sub(1)] + columns[x][y + 1]) / 2.0) as u8;
//         } else {
//             r = ((columns[x.saturating_sub(1)][y] + columns[x + 1][y]) / 2.0) as u8;
//             b = ((columns[x][y.saturating_sub(1)] + columns[x][y + 1]) / 2.0) as u8;
//         }
//     }
//     // B G B    R G R
//     // G R G or G B G
//     // B G B    R G R
//     2 => {
//         r = ((columns[x.saturating_sub(1)][y.saturating_sub(1)]
//             + columns[x + 1][y + 1]
//             + columns[x + 1][y.saturating_sub(1)]
//             + columns[x.saturating_sub(1)][y + 1])
//             / 4.0) as u8;
//         g = ((columns[x.saturating_sub(1)][y]
//             + columns[x + 1][y]
//             + columns[x][y.saturating_sub(1)]
//             + columns[x][y + 1])
//             / 4.0) as u8;
//         b = columns[x][y] as u8;
//     }
//     _ => {}
// }

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
