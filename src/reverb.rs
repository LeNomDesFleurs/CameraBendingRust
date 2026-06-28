use crate::{buffer::{DelayLine, SimpleRingBuffer}, outils};


const NUMBER_OF_ALLPASS:usize = 6;
pub struct Reverb{
    allpasses: Vec<SimpleRingBuffer>,
    pub dry_wet: f32,
}

impl Reverb{

    pub fn new(max_delay_line_size: usize)->Self{
        let mut allpasses: Vec<SimpleRingBuffer> = vec![];
        let time = [10, 20, 11, 50, 33, 27];
        for i in 0..NUMBER_OF_ALLPASS{
            allpasses.push(SimpleRingBuffer::new(max_delay_line_size, time[i], 0.0));
        }
        Reverb{
        allpasses: allpasses,
        dry_wet: 0.5,
        }
    }

    pub fn flush(&mut self){
        for i in 0..self.allpasses.len(){
            self.allpasses[i].flush();
        }
    }

    /// size as a multiplier of default time, range [0; 2] recommended
    pub fn set_size(&mut self, size: f32){
        let time = [10, 20, 11, 50, 33, 27];
        for i in 0..NUMBER_OF_ALLPASS{
            let delay_time = (time[i] as f32 * size) as usize;
            self.allpasses[i].set_delay(delay_time);
        }
    }

    pub fn set_reverb_time(&mut self, feedback:f32){
        for allpass in self.allpasses.iter_mut(){
            allpass.set_feedback(feedback);
        }
    }

    pub fn process(&mut self, input_sample:f32)->f32{
        let mut output_sample = input_sample;
        for allpass in self.allpasses.iter_mut(){
            output_sample = allpass.process(output_sample);
            // output_sample;
        }
        return outils::equal_power_crossfade(input_sample, output_sample, self.dry_wet);
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverb_set_size() {
        let mut reverb = Reverb::new(100);
        reverb.set_reverb_time(0.9);
        reverb.dry_wet = 0.8;
        reverb.flush();
        reverb.set_size(2.0);
        reverb.set_size(0.5);
        reverb.set_size(1.0);
        for i in 0..255{
            reverb.process(i as f32);
        }

    }

    #[test]
     fn reverb_dry_wet() {
        let mut reverb = Reverb::new(100);
        reverb.set_reverb_time(0.9);
        reverb.dry_wet = 0.8;
        reverb.flush();
        reverb.set_size(2.0);
        reverb.set_size(0.5);
        reverb.set_size(1.0);
        reverb.dry_wet = 0.0;
        for i in 0..255{
            assert_eq!(reverb.process(i as f32), i as f32)
        }

    }
}