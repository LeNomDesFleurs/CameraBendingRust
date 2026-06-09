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

    number_of_channels: usize, //usize because used as index

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
            number_of_channels: 3,
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

    /// Main function orchestrating everything else
    pub fn process_image(&mut self, parameters: &Parameters) {
        self.parameters = *parameters;

        self.set_delay();
        self.set_filters();

        self.number_of_channels = match *self.parameters.alpha_mode.get() {
            AlphaMode::Interleave => 4,
            _ => 3,
        };
        // make an signal out of the picture, from 2d to 1d
        self.order_signal(); // Vec<Vec<rgb>>
                             // organize colors in multiple signals, or interleave them all in one signal
        self.split_colors(); // Vec<f32>
        self.process_signal(); // Vec<f32>
        self.reconstruct_image();
        self.make_file();
    }

    /// Make 3 lanes with all pixels depending on the ordering modes
    fn order_signal(&mut self) {
        self.ordered_picture.clear();
        match self.parameters.order_mode.get() {
            OrderMode::Row => {
                for pixel in self.source_image_buffer.enumerate_pixels_mut() {
                    self.ordered_picture
                        .push([pixel.2[0], pixel.2[1], pixel.2[2], pixel.2[3]]);
                }
            }
            OrderMode::Column => {
                for x in 0..self.source_image_buffer.width() {
                    for y in 0..self.source_image_buffer.height() {
                        let pixel = self.source_image_buffer.get_pixel(x, y);
                        self.ordered_picture
                            .push([pixel[0], pixel[1], pixel[2], pixel[3]]);
                    }
                }
            }
            OrderMode::ReverseRow => {}
            OrderMode::ReverseColumn => {}
        }
    }

    pub fn init(&mut self) {
        self.filter.init(44000.0);
        self.delay.init_delay(1.0, 0.0);
    }

    pub fn set_filters(&mut self) {
        self.filter.set_frequence_and_resonance(
            self.parameters.filter_cutoff.get() as f32,
            self.parameters.filter_resonance.value as f32 / 10.0,
        )
    }

    pub fn set_delay(&mut self) {
        self.delay
            .set_delay_time(self.parameters.delay_time.get() as f32);
        self.delay
            .set_feedback(self.parameters.delay_feedback.get() as f32 / 1000.0);
    }

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
        let mut modulo = 0;

        match self.parameters.order_mode.get() {
            OrderMode::Column => modulo = self.height,
            OrderMode::Row => modulo = self.width,
            _ => {}
        }

        match self.parameters.color_mode.get() {
            ColorMode::Bayer => {
                let mut bayer_row = false;
                let mut bayer_column = false;
                // alternate between GR and BG, allow for more modularity instead of building the bayer at the ordering stage with a classic array indexing
                for pixel in self.ordered_picture.clone() {
                    // match color {
                    let mut flag = Flag::Continue;
                    if count > modulo {
                        count = 0;
                        bayer_row = !bayer_row;
                    }

                    if self.parameters.continuous.value == false && count % modulo == 0 {
                        flag = Flag::Reset
                    }

                    let color_index =
                        self.bayer_matrix[bayer_row as usize][bayer_column as usize] as usize;
                    let pixel_value = pixel[color_index];

                    self.signal.push((pixel_value as f32, flag));

                    count = count + 1;

                    bayer_column = !bayer_column;
                }
            }
            ColorMode::Interleaved => {
                for pixel in self.ordered_picture.clone() {
                    let mut flag = Flag::Continue;
                    if self.parameters.continuous.value == false && count % modulo == 0 {
                        flag = Flag::Reset
                    }
                    for i in 0..self.number_of_channels {
                        self.signal.push((pixel[i] as f32, flag));
                        flag = Flag::Continue //dirty
                    }

                    count = count + 1;
                }
            }
            ColorMode::Composite => {
                for i in 0..self.number_of_channels {
                    for pixel in self.ordered_picture.clone() {
                        let mut flag = Flag::Continue;
                        if self.parameters.continuous.value == false && count % modulo == 0 {
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
            if flag == Flag::Reset {
                self.reset_processing();
            }
            let temp = self.process_sample(pixel);
            self.processed_picture.push(temp);
        }
    }

    pub fn reconstruct_image(&mut self) {
        let mut count = 0;
        let len = self.processed_picture.len();
        let destination_len = self.destination_image_buffer.len();

        let offset = len / self.number_of_channels;
        match self.parameters.order_mode.get() {
            OrderMode::Row => {
                let mut dest: &ImageBuffer<Rgba<u8>, Vec<u8>> = &self.destination_image_buffer;
                for (xu, yu, pixel) in dest.enumerate_pixels_mut() {
                    let x = xu.saturating_sub(1) as usize;
                    let y = yu.saturating_sub(1) as usize;

                    match self.parameters.color_mode.get() {
                        ColorMode::Interleaved => {
                            let r = self.processed_picture[count] as u8;
                            let g = self.processed_picture[count + 1] as u8;
                            let b = self.processed_picture[count + 2] as u8;
                            let a = match self.parameters.alpha_mode.get() {
                                AlphaMode::Delete => 127 as u8,
                                AlphaMode::Interleave => self.processed_picture[count + 3] as u8,
                                AlphaMode::Preserve => {
                                    127
                                    // self.source_image_buffer.get_pixel(x, y)[3] as u8
                                }
                            };

                            *pixel = image::Rgba([r, g, b, a]);
                            count = count + self.number_of_channels;
                        }
                        ColorMode::Bayer => {
                            let color = self.bayer_matrix[x % 2][y % 2] as usize;
                            let (r, g, b) = self.bayer_dematricing(x, y, color);
                            *pixel = image::Rgba([r, g, b, 127]);
                            count = count + 1;
                        }
                        ColorMode::Composite => {
                            let r = self.processed_picture[count] as u8;
                            let g = self.processed_picture[count + offset] as u8;
                            let b = self.processed_picture[count + (offset * 2)] as u8;
                            let mut a = 255;

                            match self.parameters.alpha_mode.get() {
                                AlphaMode::Delete => {}
                                AlphaMode::Interleave => {
                                    a = self.processed_picture[count + (offset * 3)] as u8;
                                }
                                AlphaMode::Preserve => {
                                    a = self.source_image_buffer[(x as u32, y as u32)].0[3];
                                }
                            }

                            *pixel = image::Rgba([r, g, b, a]);
                            count = count + 1;
                        }
                    }
                }
            }
            OrderMode::Column => {}
            _ => {}
        }
    }

    pub fn coord_to_signal(&mut self, x: usize, y: usize) -> f32 {
        return self.signal[((x * self.width as usize) + y) as usize].0 as f32;
    }

    //   A
    // A X A average NESW
    //   A
    // this could be done by conbining the two horizontal and vertical dematricing
    pub fn straight_cross_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_signal(x.saturating_sub(1), y)
            + self.coord_to_signal(x + 1, y)
            + self.coord_to_signal(x, y.saturating_sub(1))
            + self.coord_to_signal(x, y + 1))
            / 4.0) as u8
    }

    //A   A
    //  X   average angles
    //A   A
    pub fn oblique_cross_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_signal(x.saturating_sub(1), y.saturating_sub(1))
            + self.coord_to_signal(x + 1, y + 1)
            + self.coord_to_signal(x + 1, y.saturating_sub(1))
            + self.coord_to_signal(x.saturating_sub(1), y + 1))
            / 4.0) as u8
    }

    // A X A

    pub fn horizontal_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_signal(x.saturating_sub(1), y) + self.coord_to_signal(x + 1, y)) / 2.0)
            as u8
    }

    //  A
    //  X
    //  A

    pub fn vertical_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_signal(x, y.saturating_sub(1)) + self.coord_to_signal(x, y + 1)) / 2.0)
            as u8
    }

    pub fn bayer_dematricing(&mut self, x: usize, y: usize, pixel_color: usize) -> (u8, u8, u8) {
        let (mut r, mut g, mut b) = (0, 0, 0);

        match pixel_color {
            // Red
            // B G B    R G R
            // G R G or G B G
            // B G B    R G R
            0 => {
                r = self.coord_to_signal(x, y) as u8;
                g = self.straight_cross_matrix(x, y);
                b = self.oblique_cross_matrix(x, y);
            }
            1 => {
                g = self.coord_to_signal(x, y) as u8;
                if y % 2 == 0 {
                    b = self.vertical_matrix(x, y);
                    r = self.horizontal_matrix(x, y);
                } else {
                    b = self.horizontal_matrix(x, y);
                    r = self.vertical_matrix(x, y);
                }
            }
            // B G B    R G R
            // G R G or G B G
            // B G B    R G R
            2 => {
                r = self.oblique_cross_matrix(x, y);
                g = self.straight_cross_matrix(x, y);
                b = self.coord_to_signal(x, y) as u8;
            }
            _ => {}
        };

        return (r, g, b);
    }

    pub fn make_file(&self) {
        // Write the contents of this image to the Writer in PNG format.

        self.destination_image_buffer.save("test.png").unwrap();
    }
}
