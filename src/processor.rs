use wasm_bindgen::prelude::wasm_bindgen;

use crate::buffer;
pub use crate::buffer::DelayLine;
pub use crate::filter::Biquad;
pub use crate::filter::FilterType;
use crate::outils;
pub use crate::reverb::Reverb;
#[derive(PartialEq)]
enum AlphaMode {
    Preserve,
    Delete,
    Interleave,
}
#[derive(PartialEq)]
enum ColorMode {
    Composite,
    Interleaved,
    Bayer,
}

#[derive(PartialEq)]
enum OrderMode {
    Row,
    Column,
    ReverseRow,
    ReverseColumn,
}

pub struct Parameters {
    // signal params
    pub alpha_mode: AlphaMode,
    pub color_mode: ColorMode,
    pub order_mode: OrderMode,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub reverb_dry_wet: f32,
    pub reverb_decay: f32,
    wavefolder_amount: f32,
    wavefolder_frequency: f32,
    pub continuous: bool,
}

impl Parameters {
    pub fn new(
        alpha_mode: u32,
        color_mode: u32,
        order_mode: u32,
        delay_time: f32,
        delay_feedback: f32,
        filter_cutoff: f32,
        filter_resonance: f32,
        reverb_dry_wet: f32,
        reverb_decay: f32,
        wavefolder_amount: f32,
        wavefolder_frequency: f32,
        continuous: bool,
    ) -> Self {
        let alpha_mode_enum: AlphaMode = match alpha_mode {
            0 => AlphaMode::Preserve,
            1 => AlphaMode::Interleave,
            2 | _ => AlphaMode::Delete,
        };

        let order_mode_enum: OrderMode = match order_mode {
            0 => OrderMode::Row,
            1 => OrderMode::Column,
            2 => OrderMode::ReverseRow,
            3 | _ => OrderMode::ReverseColumn,
        };

        let color_mode_enum: ColorMode = match color_mode {
            0 => ColorMode::Composite,
            1 => ColorMode::Interleaved,
            2 | _ => ColorMode::Bayer,
        };

        Self {
            alpha_mode: alpha_mode_enum,
            color_mode: color_mode_enum,
            order_mode: order_mode_enum,
            delay_time,
            delay_feedback,
            filter_cutoff,
            filter_resonance,
            reverb_dry_wet,
            reverb_decay,
            wavefolder_amount,
            wavefolder_frequency,
            continuous,
        }
    }
}

// pub use crate::outils;

#[derive(Clone, PartialEq)]
enum Flag {
    Reset,
    Continue,
}

#[cfg_attr(feature = "enable_wasm", wasm_bindgen)]
#[derive(Clone, Debug)]
pub struct Picture {
    data: Vec<u8>, // interleave rgba for wasm compatibility. processing in f32
    pub width: usize,
    pub height: usize,
}

impl Picture {
    pub fn new(raw_data: Vec<u8>, width: usize, height: usize) -> Self {
        Self {
            data: raw_data,
            width,
            height,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let index = self.get_index(x, y);
        return [
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ];
    }
    pub fn set_pixel(&mut self, x: usize, y: usize, value: [u8; 4]) {
        let index = self.get_index(x, y);

        for i in 0..4 {
            self.data[index + i] = value[i];
        }
    }
    pub fn get_pixel_color(&self, x: usize, y: usize, color_index: usize) -> u8 {
        let index = self.get_index(x, y) + color_index;
        self.data[index]
    }
    fn get_index(&self, x: usize, y: usize) -> usize {
        // wrap if too high, no security over under index, I don't want it to fail silently
        ((x as usize * 4) + (y * self.width * 4) as usize) as usize % self.data.len()
    }
    pub fn get_raw_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    pub fn get_lenght(&self) -> usize {
        return self.data.len();
    }
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

    source_image_buffer: Picture,
    ordered_picture: Vec<[u8; 4]>,
    signal: Vec<(f32, Flag)>,
    processed_picture: Vec<f32>,
    slices: Vec<Vec<f32>>,
    output_picture: Picture,

    // signal: Signal<f32>,
    filter: Biquad,
    delay: DelayLine,
    reverb: Reverb,

    // ---------------file----------
    width: usize,
    height: usize,
    size: usize,

    number_of_channels: usize, //usize because used as index
    bayer_matrix: [[i32; 2]; 2],
    signal_built: bool,
}

impl Processor {
    pub fn new(input_image: Picture, parameters: Parameters) -> Self {
        let bufimg = input_image;
        let height = bufimg.height;
        let width = bufimg.width;
        let mut new = Self {
            parameters: parameters,
            quantization: 0.0,
            signal: Vec::new(),
            filter: Biquad::new(FilterType::LPF),
            delay: DelayLine::new(1000.0, buffer::DelayMode::Comb),
            bayer_matrix: [[1, 0], [2, 1]],
            // [ G ; R ]
            // [ B ; G ]
            ordered_picture: Vec::new(),
            processed_picture: Vec::new(),
            number_of_channels: 3,
            slices: Vec::new(),
            source_image_buffer: bufimg.clone(),
            output_picture: bufimg.clone(),
            width: width,
            height: height,
            size: width * height,
            reverb: Reverb::new(),
            signal_built: false,
        };
        new.init();
        new
    }

    fn dumb_wavefolder(&self, input_sample: f32) -> f32 {
        let half = 255.0 / 2.0;
        let period = self.parameters.wavefolder_frequency;
        let output_sample = ((input_sample / period).sin() * half) + half;
        return outils::linear_crossfade(
            input_sample,
            output_sample,
            self.parameters.wavefolder_amount,
        );
    }

    /// Main function orchestrating everything else
    pub fn process_image(&mut self) -> Picture {
        self.set_delay();
        self.set_filters();
        self.set_reverb();

        self.number_of_channels = match self.parameters.alpha_mode {
            AlphaMode::Interleave => 4,
            _ => 3,
        };

        // make an signal out of the picture, from 2d to 1d
        self.order_signal(); // Vec<Vec<rgb>>
                             // organize colors in multiple signals, or interleave them all in one signal
        self.split_colors(); // Vec<f32>
        self.signal_built = true;
        self.process_signal(); // Vec<f32>
        self.reconstruct_image();
        return self.output_picture.clone();
    }

    /// Make 3 lanes with all pixels depending on the ordering modes
    fn order_signal(&mut self) {
        self.ordered_picture.clear();
        match self.parameters.order_mode {
            OrderMode::Row => {
                for y in 0..self.width {
                    for x in 0..self.height {
                        let pixel = self.source_image_buffer.get_pixel(x, y);
                        self.ordered_picture
                            .push([pixel[0], pixel[1], pixel[2], pixel[3]]);
                    }
                }
            }
            OrderMode::Column => {
                for x in 0..self.width {
                    for y in 0..self.height {
                        let pixel = self.source_image_buffer.get_pixel(x, y);
                        self.ordered_picture
                            .push([pixel[0], pixel[1], pixel[2], pixel[3]]);
                    }
                }
            }
            OrderMode::ReverseRow => {
                for y in self.width..0 {
                    for x in self.height..0 {
                        let pixel = self.source_image_buffer.get_pixel(x, y);
                        self.ordered_picture
                            .push([pixel[0], pixel[1], pixel[2], pixel[3]]);
                    }
                }
            }
            OrderMode::ReverseColumn => {
                for x in self.width..0 {
                    for y in self.height..0 {
                        let pixel = self.source_image_buffer.get_pixel(x, y);
                        self.ordered_picture
                            .push([pixel[0], pixel[1], pixel[2], pixel[3]]);
                    }
                }
            }
        }
    }

    pub fn init(&mut self) {
        self.filter.init(44000.0);
        self.delay.init_delay(1.0, 0.0);
        self.reverb.init(100.0);
    }

    pub fn set_filters(&mut self) {
        self.filter.set_frequence_and_resonance(
            self.parameters.filter_cutoff as f32,
            self.parameters.filter_resonance as f32,
        )
    }

    pub fn set_delay(&mut self) {
        self.delay.set_delay_time(self.parameters.delay_time as f32);
        self.delay
            .set_feedback(self.parameters.delay_feedback as f32);
    }

    pub fn set_reverb(&mut self) {
        self.reverb
            .set_reverb_time(self.parameters.reverb_decay as f32);
        self.reverb.dry_wet = self.parameters.reverb_dry_wet as f32;
    }

    fn reset_processing(&mut self) {
        self.filter.flush();
        self.delay
            .init_delay(1.0, self.parameters.delay_time as f32); // dirty, add a flush function
        self.reverb.init(100.0);
        self.set_delay();
        self.set_filters();
        self.set_reverb();
    }

    fn process_sample(&mut self, sample: f32) -> f32 {
        let mut temp = sample;
        // temp = self.filter.process(temp);
        temp = self.delay.process(temp);
        temp = self.reverb.process(temp);
        temp = self.dumb_wavefolder(temp);
        if temp > 255.0 {
            temp = 255.0
        }
        if temp < 0.0 {
            temp = 0.0
        }

        return temp;
    }

    fn split_colors(&mut self) {
        self.signal.clear();
        let mut count = 0;
        let mut modulo = 0;

        match self.parameters.order_mode {
            OrderMode::Column => modulo = self.height,
            OrderMode::Row => modulo = self.width,
            _ => {}
        }

        match self.parameters.color_mode {
            ColorMode::Bayer => {
                let mut bayer_row = false;
                let mut bayer_column = false;
                // alternate between GR and BG, allow for more modularity instead of building the bayer at the ordering stage with a classic array indexing
                // TODO should probably replace this cloning by a reference, optim
                for pixel in self.ordered_picture.clone() {
                    // match color {
                    let mut flag = Flag::Continue;
                    if count == modulo {
                        if self.parameters.continuous == false {
                            flag = Flag::Reset;
                        }
                        count = 0;
                        bayer_row = !bayer_row;
                    }

                    let x = bayer_row as usize;
                    let y = bayer_column as usize;
                    let color_index = self.bayer_matrix[y][x] as usize;
                    let pixel_value = pixel[color_index];

                    self.signal.push((pixel_value as f32, flag));

                    count = count + 1;

                    bayer_column = !bayer_column;
                }
            }
            ColorMode::Interleaved => {
                for pixel in self.ordered_picture.clone() {
                    let mut flag = Flag::Continue;
                    if self.parameters.continuous == false && count % modulo == 0 {
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
                        if self.parameters.continuous == false && count % modulo == 0 {
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
        let destination_len = self.output_picture.data.len();
        let offset = len / self.number_of_channels;

        match self.parameters.order_mode {
            OrderMode::Row => {
                for y in 0..self.height as usize {
                    for x in 0..self.width as usize {
                        match self.parameters.color_mode {
                            ColorMode::Interleaved => {
                                let r = self.processed_picture[count] as u8;
                                let g = self.processed_picture[count + 1] as u8;
                                let b = self.processed_picture[count + 2] as u8;
                                let a = match self.parameters.alpha_mode {
                                    AlphaMode::Delete => 255 as u8,
                                    AlphaMode::Interleave => {
                                        self.processed_picture[count + 3] as u8
                                    }
                                    AlphaMode::Preserve => {
                                        self.source_image_buffer.get_pixel(x, y)[3] as u8
                                    }
                                };

                                self.output_picture.set_pixel(x, y, [r, g, b, a]);
                                count = count + self.number_of_channels;
                            }
                            ColorMode::Bayer => {
                                let color = self.bayer_matrix[y % 2][x % 2] as usize;
                                let (r, g, b) = self.bayer_dematricing(x, y, color);
                                let a = match self.parameters.alpha_mode {
                                    AlphaMode::Delete => 255 as u8,
                                    AlphaMode::Interleave => 255, // To do, 4 color marticing ?
                                    AlphaMode::Preserve => {
                                        self.source_image_buffer.get_pixel(x, y)[3] as u8
                                    }
                                };
                                self.output_picture.set_pixel(x, y, [r, g, b, a]);
                                count = count + 1;
                            }
                            ColorMode::Composite => {
                                let r = self.processed_picture[count] as u8;
                                let g = self.processed_picture[count + offset] as u8;
                                let b = self.processed_picture[count + (offset * 2)] as u8;
                                let mut a = 255;

                                match self.parameters.alpha_mode {
                                    AlphaMode::Delete => {}
                                    AlphaMode::Interleave => {
                                        a = self.processed_picture[count + (offset * 3)] as u8;
                                    }
                                    AlphaMode::Preserve => {
                                        a = self.source_image_buffer.get_pixel_color(x, y, 3);
                                    }
                                }
                                self.output_picture.set_pixel(x, y, [r, g, b, a]);
                                count = count + 1;
                            }
                        }
                    }
                }
            }
            OrderMode::Column => {}
            _ => {}
        }
    }

    pub fn coord_to_processed_signal(&mut self, x: usize, y: usize) -> f32 {
        // pixels are stored left to right, top to bottom, thus the column offset is y times the number of pixel in a row
        let index = ((x as usize) + y * self.width as usize) as usize % self.signal.len();
        return self.processed_picture[index] as f32;
    }

    //   A
    // A X A average North East South West
    //   A
    // this could be done by conbining the two horizontal and vertical dematricing
    pub fn straight_cross_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_processed_signal(x.saturating_sub(1), y)
            + self.coord_to_processed_signal(x + 1, y)
            + self.coord_to_processed_signal(x, y.saturating_sub(1))
            + self.coord_to_processed_signal(x, y + 1))
            / 4.0) as u8
    }

    //A   A
    //  X   average angles
    //A   A
    pub fn oblique_cross_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_processed_signal(x.saturating_sub(1), y.saturating_sub(1))
            + self.coord_to_processed_signal(x + 1, y + 1)
            + self.coord_to_processed_signal(x + 1, y.saturating_sub(1))
            + self.coord_to_processed_signal(x.saturating_sub(1), y + 1))
            / 4.0) as u8
    }

    // A X A average East West

    pub fn horizontal_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_processed_signal(x.saturating_sub(1), y)
            + self.coord_to_processed_signal(x + 1, y))
            / 2.0) as u8
    }

    //  A
    //  X average North South
    //  A

    pub fn vertical_matrix(&mut self, x: usize, y: usize) -> u8 {
        ((self.coord_to_processed_signal(x, y.saturating_sub(1))
            + self.coord_to_processed_signal(x, y + 1))
            / 2.0) as u8
    }

    pub fn bayer_dematricing(&mut self, x: usize, y: usize, pixel_color: usize) -> (u8, u8, u8) {
        let (mut r, mut g, mut b) = (0, 0, 0);

        match pixel_color {
            // red and blue indexes are flipped
            // don't ask
            // I'm tired

            // Red
            // B G B  B is oblique
            // G R G  G is straight
            // B G B
            2 => {
                r = self.coord_to_processed_signal(x, y) as u8;
                g = self.straight_cross_matrix(x, y);
                b = self.oblique_cross_matrix(x, y);
            }

            // Green
            // G B G        G R G
            // R G R   or   B G B     thus the if statement
            // G B G        G R G
            1 => {
                g = self.coord_to_processed_signal(x, y) as u8;
                if y % 2 == 1 {
                    r = self.horizontal_matrix(x, y);
                    b = self.vertical_matrix(x, y);
                } else {
                    r = self.vertical_matrix(x, y);
                    b = self.horizontal_matrix(x, y);
                }
            }
            // R G R   R is oblique
            // G B G   G is straight
            // R G R
            0 => {
                r = self.oblique_cross_matrix(x, y);
                g = self.straight_cross_matrix(x, y);
                b = self.coord_to_processed_signal(x, y) as u8;
            }
            _ => {}
        };

        return (r, g, b);
    }

    pub fn make_file(&self) {
        // Write the contents of this image to the Writer in PNG format.

        // self.output_picture.save("test.png").unwrap();
    }
}
