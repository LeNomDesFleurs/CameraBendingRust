// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;

mod buffer;
pub use buffer::DelayLine;

use crossterm::execute;
use crossterm::{
    cursor, event, event::Event, event::KeyCode, event::KeyEvent, event::KeyModifiers,
    style::Stylize, terminal, terminal::disable_raw_mode, terminal::enable_raw_mode,
};

#[derive(Copy)]

enum ColorMode{
    Bayer,
    Interleaved,
    Composite,
}

#[derive(Copy)]
enum AlphaMode{
    Preserve,
    Delete,
    Interleave, //does nothing in bayer mode
}

#[derive(Copy)]

enum OrderMode{
    Column,
    Row,
}



//TODO add transparency Layer ?
//might create some really cool things
enum Signal<T>{
    InterleavedArray(Vec<Vec<T>>),
    InterleavedVector(Vec<T>),
    CompositeArray([Vec<Vec<T>>; 3]),
    CompositeVector([Vec<T>; 3]),
}

#[derive(Copy)]
struct Parameters{
 // signal params
    alpha_mode: AlphaMode,
    color_mode: ColorMode,
    order_mode: OrderMode,
    // wether everything is flushed between columns / rows
    continous: bool,
   
    // delay Param
    delay_size: i32,
    delay_feedback: f32,
number_of_parameters: u32,

}

impl Parameters{
    pub fn new()->Self{
        Self { 
             alpha_mode: AlphaMode::Delete, 
            color_mode: ColorMode::Composite, 
            order_mode: OrderMode::Column, 
            continous: true, 
            delay_size: 0, 
            delay_feedback: 0.0, 
            number_of_parameters: 6 }
    }
    pub fn get_number_of_params(&self)->u32{
        self.number_of_parameters
    }
}


struct ui{
parameters: Parameters,
position: u32,
// TODO increment value
}


impl ui{

    pub fn new()->Self{
        Self { parameters: Parameters::new(), position: 0 }
    }

    pub fn increment_value(&mut self){


        self.update_display();
    }

    pub fn decrement_value(&mut self){

        self.update_display();

    }

    pub fn increment_position(&mut self){
        self.position = (self.position + 1)%self.parameters.get_number_of_params();
        self.update_display();

    }

    pub fn decrement_position(&mut self){
        self.position = (self.position - 1)%self.parameters.get_number_of_params();
        self.update_display();
    }

    pub fn get_parameters(&self)->Parameters{
        self.parameters
    }

    pub fn update_display(&self){

    }
}

struct Processor{
   

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

    image_buffer:  

    bayer_matrix: [[i32; 2]; 2],


    //filter params
    filter_freq:f32,
    filter_resonance: f32,


   
    path: String,
}

impl Processor{

    pub fn new(in_path: String)->Self{
    let dynimg = image::open(in_path).unwrap();
    let mut bufimg = dynimg.into_rgb8();
        let height = bufimg.dimensions().1;
        let width = bufimg.dimensions().0;
       let new = Self { 
           
            quantization: , 
            signal: Signal::InterleavedVector(vec![0.0 as f32, 0.0 as f32]), 
            filter: Biquad::new(FilterType::LPF), 
            delay: DelayLine::new(1000.0, buffer::DelayMode::Comb), 
            feedback: 0.0,
            bayer_matrix: [[1, 2], [0, 1]],
            filter_freq: (), 
            filter_resonance: (), 
            delay_size: 0,
            image_buffer: bufimg, 
            width: width,
            height: height,
            size: width * height,
            String: () };
            new.init()
    }

    pub fn build_signal(&mut self){
        
        for pixel in self.image_buffer.enumerate_pixels_mut() {
            match color_mode()

            match self.ColorMode{
                //update the buffer for the dematricing
                let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];
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

    pub fn set_filters(&mut self, freq, resonance){
        if filter[0].get_frequence_and_Q
        for filter in self.filters.iterate(){
            filter.set_frequence_and_resonance()
        }
    }

    pub fn bayer_dematricing(){

    }

    pub fn process_image(&mut self, new_params: Parameters){
        

        for column in slices.iter_mut() {
        let mut prev_sample = 0.0;

        for sample in slice.iter_mut() {
            let temp = prev_sample * self.feedback + *sample;
            *sample = temp;
            prev_sample = temp;
        }
        if 
        filter.flush;
        for idx in 0..1000 {
            bufr.process(0.0);
        }
    }
    }

    pub fn reconstruct_image(&mut self){

    for pixel in self.image_buffer.enumerate_pixels_mut() {
        let color = self.bayer_matrix[(pixel.0 % 2) as usize][(pixel.1 % 2) as usize];

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

    }

    pub fn make_file(){
    // Write the contents of this image to the Writer in PNG format.

        self.image_buffer.save(self.path).unwrap();
    }

}





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
   
    let mut processor = Processor::new(in_path.to_string());

    init_terminal();

    loop {
        if let Event::Key(KeyEvent { code, .. }) =
            event::read().unwrap_or(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
        {
            //got to refresh at the end, will be modified if the event involve more modification
            ui_event = UiEvent::Refresh;
            parameters_modified = None;
            match code {
                KeyCode::Esc => {
                    disable_raw_mode().unwrap();
                    execute!(std::io::stdout(), cursor::Show)?;
                    println!("{}", terminal::Clear(terminal::ClearType::All));
                    println!("{}", cursor::MoveTo(0, 0));
                    break;
                }
                KeyCode::Down => {
                    if selected < number_of_params as i32 - 1 { 
                        selected += 1;
                    }
                    ui_event = UiEvent::UpdateSelection(selected);
                }
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                    ui_event = UiEvent::UpdateSelection(selected);
                }
                KeyCode::Right => parameters_modified = Some(ParameterModified::Increment),
                KeyCode::Left => parameters_modified = Some(ParameterModified::Decrement),
                KeyCode::Char(char) => {
                    if char == '<' {
                        let mut midi_chan = midi_channel.lock().unwrap();
                        //min channel 0
                        if *midi_chan > 0 {
                            *midi_chan -= 1;
                            ui_event = UiEvent::UpdateMidiChannel(*midi_chan);
                        }
                    } else if char == '>' {
                        let mut midi_chan = midi_channel.lock().unwrap();
                        //max channel 15
                        if *midi_chan < 15 {
                            *midi_chan += 1;
                            ui_event = UiEvent::UpdateMidiChannel(*midi_chan);
                        }
                    } else {
                        parameters_modified = Some(ParameterModified::SetValue(char))
                    }
                }
                KeyCode::Tab => {
                    _midi_connection = match connect_midi(
                        midi_sender.clone(),
                        parameters.clone(),
                        param_sender.clone(),
                        gui_sender.clone(),
                        midi_channel.clone(),
                    ) {
                        Ok((midi_connection, port_name)) => {
                            ui_event = UiEvent::UpdateMidiportName(port_name);
                            *midi_channel.lock().unwrap() = 0;
                            midi_connection
                        }
                        Err(error) => panic!("can't connect to midi: {:?}", error),
                    };
                }
                // Key::Ctrl('q') => self.should_quit = true,
                _ => {}
            }

            //update Audio Thread if a parameter is modified
            parameters_modified.map(|modification| {
                //the first unwrap is in the case where a mutex fucks up, the second is for get_mut and only return None if there is no parameter in the Hash map, which cannot happen
                let mut parameters_binding = parameters.lock().unwrap();
                let capsule_binding = parameters_binding
                    .capsules
                    .get_mut(selected as usize)
                    .unwrap();
                let parameter = &mut capsule_binding.parameter;
                let id = capsule_binding.id;
                //apply modification
                match modification {
                    ParameterModified::Increment => parameter.increment(),
                    ParameterModified::Decrement => parameter.decrement(),
                    ParameterModified::SetValue(char) => parameter.set_value(char),
                }
                //get a copy of the parameter and send it to the audio thread
                param_sender.send((id, parameter.get_raw_value())).unwrap();
            });
        }
    }

    

    Ok(())
}


pub fn init_terminal(){
     enable_raw_mode().unwrap();
    execute!(std::io::stdout(), cursor::Hide).unwrap();
}