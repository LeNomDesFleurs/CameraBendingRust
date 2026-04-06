
use crossterm::style::Color;
use image::ImageBuffer;
use image::Rgb;
use image::Rgba;

use crate::buffer;
pub use crate::parameters::{Parameters, OrderMode, AlphaMode, ColorMode};
pub use crate::filter::Biquad;
pub use crate::filter::FilterType;
pub use crate::buffer::DelayLine;


// pub use crate::outils;


//TODO add transparency Layer ?
//might create some really cool things
enum Signal<T>{
    // slice 1 (RGBRGBRGB) slice 2 (RGBRGBRGB) ...
    InterleavedArray(Vec<Vec<T>>),
    // one line RGB RGB RGB RGB
    InterleavedVector(Vec<T>),
    // slice 1 (RRRRRR) slice 2 (RRRRRR) .... slice 1 (GGGGGG) slice 2 (GGGGG) ...
    CompositeArray([Vec<Vec<T>>; 4]),
    // 1 slice for each color channel
    CompositeVector([Vec<T>; 4]),
}

pub struct Processor{
   

    parameters: Parameters,

    //-------------processing-----------------

    quantization: f32,

    ordered_picture: Vec<[u8; 4]>,
    processed_picture: Vec<[u8; 4]>,
    slices: Vec<Vec<f32>>,
    signal: Signal<f32>,


    filter: Biquad,
    delay: DelayLine,

      // ---------------file----------
    width : u32,
    height : u32,
    size : u32,

    source_image_buffer:ImageBuffer<Rgba<u8>, Vec<u8>>,

    destination_image_buffer:ImageBuffer<Rgba<u8>, Vec<u8>>,

    bayer_matrix: [[i32; 2]; 2],
   

   
    path: String,
}

impl Processor{

    pub fn new(in_path: &str, parameters: &Parameters)->Self{
    let dynimg = image::open(in_path).unwrap();
    let mut bufimg = dynimg.into_rgba8();
        let height = bufimg.dimensions().1;
        let width = bufimg.dimensions().0;
       let mut new = Self { 
            parameters: Parameters::new(),
            quantization: 0.0, 
            signal: Signal::InterleavedVector(vec![0.0 as f32, 0.0 as f32]), 
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

    pub fn init(&mut self){
        self.filter.init(44000.0);
        self.delay.init_delay(1.0, 0.0);
    }
    
    
        pub fn set_filters(&mut self)
        {   
            self.filter.set_frequence_and_resonance(self.parameters.filter_cutoff.get() as f32, self.parameters.filter_resonance.value as f32)
        }

        pub fn set_delay(&mut self){
            self.delay.set_delay_time(self.parameters.delay_time.get() as f32);
            self.delay.set_feedback(self.parameters.delay_time.get() as f32);
        }


    fn order_signal(&mut self){
        self.ordered_picture.clear();
        match self.parameters.order_mode.get(){
            OrderMode::Row=>{
        
        for pixel in self.source_image_buffer.enumerate_pixels_mut() {
            self.ordered_picture.push([pixel.2[0], pixel.2[1], pixel.2[2], pixel.2[3]]);
        }
            }
            OrderMode::Column=>{}
            OrderMode::ReverseRow=>{}
            OrderMode::ReverseColumn=>{}
        }
    }


    pub fn bayer_dematricing(){

    }

    fn process_signal(&mut self){

        self.processed_picture.clear();

        match self.parameters.color_mode.get(){
            ColorMode::Interleaved=>{
                let mut r;
                let mut g;
                let mut b;
                let mut a;

                for pixel in <Vec<[u8; 4]> as Clone>::clone(&self.ordered_picture).into_iter(){

                        r = self.delay.process(self.filter.process(pixel[0] as f32));
                        g = self.delay.process(self.filter.process(pixel[1] as f32));
                        b = self.delay.process(self.filter.process(pixel[2] as f32));
                        a = self.delay.process(self.filter.process(pixel[3] as f32));
                    self.processed_picture.push([r as u8, g as u8, b as u8, a as u8]);
                }
                // reset on counter = width
            }
            ColorMode::Bayer=>{}
            ColorMode::Composite=>{

            }
        }
       
    }

    pub fn process_image(&mut self, parameters: &Parameters){
        self.parameters = *parameters;

        self.set_delay();
        self.set_filters();
        self.order_signal();
        // self.split_colors();
        self.process_signal();
        self.reconstruct_image();
        self.make_file();

    }
    

    pub fn reconstruct_image(&mut self){
let mut count = 0;
        match self.parameters.order_mode.get(){
            OrderMode::Row=>{

                for pixel in self.destination_image_buffer.enumerate_pixels_mut() {
                    let y = pixel.1.saturating_sub(1) as usize;
                    let x = pixel.0.saturating_sub(1) as usize;
                    
                    match self.parameters.color_mode.get(){
                        ColorMode::Interleaved=>{
                            
                            // Bayer reconstruction
                            let mut r = self.processed_picture[count][0];
                            let mut g = self.processed_picture[count][1];
                            let mut b = self.processed_picture[count][2];
                            let mut a = 127;
                            
                            match self.parameters.alpha_mode.get(){
                                AlphaMode::Delete=>{}
                                AlphaMode::Interleave=>{
                                    a = self.processed_picture[count][3];
                                }
                                AlphaMode::Preserve=>{
                                    a = self.source_image_buffer[(x as u32, y as u32)].0[3];
                                }
                                
                                
                            }
                           
                            
                            *pixel.2 = image::Rgba([r, g, b, a]);
                            count = count+1;
                        }
                        ColorMode::Bayer=>{
                            let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];

                        }
                        ColorMode::Composite=>{
                            let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];

                        }
                    }

            }},
            OrderMode::Column=>{},
            _=>{}




    
    }
    
    }



    pub fn make_file(&self){
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