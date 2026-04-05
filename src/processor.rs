
use image::ImageBuffer;
use image::Rgb;

use crate::buffer;
pub use crate::parameters::Parameters;
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
    CompositeArray([Vec<Vec<T>>; 3]),
    // 1 slice for each color channel
    CompositeVector([Vec<T>; 3]),
}

pub struct Processor{
   

    parameters: Parameters,

    //-------------processing-----------------

    quantization: f32,


    signal: Signal<f32>,

    filter: Biquad,
    delay: DelayLine,

      // ---------------file----------
    width : u32,
    height : u32,
    size : u32,

    image_buffer:ImageBuffer<Rgb<u8>, Vec<u8>>,


    bayer_matrix: [[i32; 2]; 2],
   

   
    path: String,
}

impl Processor{

    pub fn new(in_path: String, parameters: &Parameters)->Self{
    let dynimg = image::open(in_path).unwrap();
    let mut bufimg = dynimg.into_rgb8();
        let height = bufimg.dimensions().1;
        let width = bufimg.dimensions().0;
       let mut new = Self { 
            parameters: parameters::new(),
            quantization: 0.0, 
            signal: Signal::InterleavedVector(vec![0.0 as f32, 0.0 as f32]), 
            filter: Biquad::new(FilterType::LPF), 
            delay: DelayLine::new(1000.0, buffer::DelayMode::Comb), 
            bayer_matrix: [[1, 2], [0, 1]],
            image_buffer: bufimg, 
            width: width,
            height: height,
            size: width * height,
            path: in_path };
            new.init();
            new
    }

    pub fn build_signal(&mut self){

        match self.parameters.color_mode{
            ColorMode::Bayer=>{}
            ColorMode::Interleaved =>{
                make_composite_vector();
            }
            ColorMode::Composite=>{}
        }
        
        for pixel in self.image_buffer.enumerate_pixels_mut() {

            match self.parameters.colorMode{

                //update the buffer for the dematricing
                // let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];
            }
            // apply filter to the new pixel and push it on the buffer
            slice[pixel.0 as usize].push(pixel.2[color] as f32);
            // signal.push(pixel.2[color] as f32);
        }

    }

    pub fn init(&mut self){
        self.filter.init(44000.0);
        self.delay.init_delay(1.0, delay);
    }

    pub fn set_filters(&mut self, freq: f32, resonance: f32){
        // if filter[0].get_frequence_and_Q();
        for filter in self.filters.iterate(){
            filter.set_frequence_and_resonance()
        }
    }

    pub fn bayer_dematricing(){

    }

    pub fn make_composite_vector(&mut self)
    {
        let mut pixels = vec::new();
        for 
        signal = Signal::InterleavedVector(())
    }

    pub fn process_image(&mut self, parameters: &Parameters){
        self.parameters = *parameters;

        // for column in slices.iter_mut() {
        // let mut prev_sample = 0.0;

        // for sample in slice.iter_mut() {
        //     let temp = prev_sample * self.feedback + *sample;
        //     *sample = temp;
        //     prev_sample = temp;
        // }
        // if 
        // filter.flush;
        // for idx in 0..1000 {
        //     bufr.process(0.0);
        // }



    }
    }

    pub fn reconstruct_image(&mut self){

    for pixel in self.image_buffer.enumerate_pixels_mut() {
        let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];

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

        *pixel.2 = image::Rgb([r, g, b]);
    }

    }

    pub fn make_file(&self){
    // Write the contents of this image to the Writer in PNG format.

        self.image_buffer.save("test.png").unwrap();
    }

}
