
use crate::outils::{rt60_to_gain};
#[derive(Clone, Copy, PartialEq, Eq)]
enum InterpolationMode {
    None,
    Linear,
    Allpass,
}

// #[derive(Copy, Clone)]
pub struct RingBuffer {
    interpolation_mode: InterpolationMode,
    pub freezed: bool,
    reverse: bool,
    sample_rate: f32,
    buffer: Vec<f32>,
    read: f32,
    write: f32,
    i_read: i32,
    i_read_next: i32,
    step_size: f32,
    size_goal: i32,
    buffer_size: i32,
    actual_size: f32,
    size_on_freeze: f32,
    frac: f32,
    output_sample: f32,
    max_time: f32, //size in seconds
                   // self.buffer_size en base 0
}

impl RingBuffer {
    ///Buffer size in seconds
    pub fn new(max_time: f32) -> Self {
        RingBuffer {
            interpolation_mode: InterpolationMode::None,
            freezed: false,
            reverse: false,
            sample_rate: 0.0,
            buffer: vec![0.; 1],
            buffer_size: 0,
            write: 0.0,
            actual_size: 0.0,
            size_goal: 0,
            read: 0.,
            i_read_next: 1,
            i_read: 0,
            step_size: 1.,
            size_on_freeze: 0.,
            frac: 0.,
            output_sample: 0.,
            max_time,
        }
    }

    pub fn init(&mut self, sample_rate: f32) {
        let buffer_size: usize = (sample_rate * self.max_time) as usize;
        self.sample_rate = sample_rate;
        self.buffer = vec![0.; buffer_size];
        self.buffer_size = (buffer_size - 1) as i32;
        self.write = (buffer_size / 2) as f32;
        self.actual_size = (buffer_size / 2) as f32;
        self.size_goal = (buffer_size / 2) as i32;
    }

    //init the size to avoid resizing at begining
    pub fn init_delay(&mut self, sample_rate: f32, delay: f32) {
        let buffer_size: usize = (sample_rate * self.max_time) as usize;
        let delay_size = sample_rate * delay;
        self.sample_rate = sample_rate;
        self.buffer = vec![0.; buffer_size];
        self.buffer_size = (buffer_size - 1) as i32;
        self.write = delay_size as f32;
        self.read = 0.0;
        self.actual_size = delay_size as f32;
        self.size_goal = delay_size as i32;
    }

    /// @brief increment pointer and set its int, incremented int and frac value
    fn increment_read_pointer(&mut self) {
        self.read += self.step_size;
        self.check_for_read_index_overflow();
        if self.read > self.buffer_size as f32 {
            self.read -= self.buffer_size as f32
        }
        // in case of reverse read
        else if self.read < 0. {
            self.read += self.buffer_size as f32
        }
    }

    /// increment read pointer and return sample from interpolation
    pub fn read_sample(&mut self) -> f32 {
        if self.reverse {
            self.step_size = 0. - self.step_size;
        }

        if self.freezed {
            self.freeze_increment_read_pointer();
            self.freezed_update_step_size();
        } else {
            self.update_step_size();
            self.increment_read_pointer();
        }

        self.fractionalize_read_index();

        // those functions modify the self.output_sample value
        match self.interpolation_mode {
            InterpolationMode::None => self.no_interpolation(),
            InterpolationMode::Linear => self.linear_interpolation(),
            InterpolationMode::Allpass => self.allpass_interpolation(),
        }

        if self.freezed && self.step_size < 1.0 {
            self.output_sample /= self.step_size.powf(1.5);
        }

        return self.output_sample;
    }

    fn no_interpolation(&mut self) {
        self.output_sample = self.buffer[self.i_read as usize];
    }

    fn flush(&mut self) {
        self.buffer.iter_mut().map(|x| *x = 0.0).count();
    }

    /// Interpolation lineaire du buffer a un index flottant donne
    fn linear_interpolation(&mut self) {
        // S[n]=frac * Buf[i+1]+(1-frac)*Buf[i]
        self.output_sample = (self.frac * self.buffer[self.i_read_next as usize])
            + ((1. - self.frac) * self.buffer[self.i_read as usize]);
    }

    /// Interpolation passe-tout, recursion
    fn allpass_interpolation(&mut self) {
        // S[n]=Buf[i+1]+(1-frac)*Buf[i]-(1-frac)*S[n-1]
        self.output_sample = (self.buffer[(self.i_read) as usize])
            + ((1. - self.frac) * self.buffer[(self.i_read) as usize])
            - ((1. - self.frac) * self.output_sample);
    }

    /// increment write pointer and write input sample in buffer
    /// input_sample
    pub fn write_sample(&mut self, input_sample: f32) {
        if !self.freezed {
            if self.write > (self.buffer_size - 1) as f32 {
                self.write = 0.;
            } else {
                self.write += 1.
            };
            self.buffer[self.write as usize] = input_sample;
            // self.buffer[0] = input_sample;
        }
    }

    pub fn set_step_size(&mut self, step_size: f32) {
        self.step_size = step_size;
    }

    /// Triggered at each sample, update the step size and the self.actual_size
    /// to keep up with change of size goal
    fn update_step_size(&mut self) {
        let _correction_offset: f32 = 0.;
        if self.actual_size > (self.size_goal - 5) as f32
            && self.actual_size < (self.size_goal + 5) as f32
        {
            self.step_size = 1.0;
        } else if self.actual_size > self.size_goal as f32 {
            self.step_size = 1.5;
            self.actual_size -= 0.5;
            // update the step size but with slew for clean repitch
        } else if self.actual_size < self.size_goal as f32 {
            self.step_size = 0.5;
            self.actual_size += 0.5;
        }

        // self.step_size = noi::Outils::slewValue(correction_offset, self.step_size,
        // 0.999);

        // if (self.step_size > 0.999 && self.step_size < 1.0001) {
        //   self.step_size = 1.0;
        // }

        // if (!freezed){
        // if (self.step_size > 1) {
        //   self.actual_size -= self.step_size - 1;

        // } else if (self.step_size < 1) {
        //   self.actual_size += 1 - self.step_size;
        // }
        // update the step size and update the actual delay time
        // }
    }

    /// Take a delay time in milliseconds, clip it within the defined max
    /// buffer size and set the goal to reach.
    /// delay_time in milliseconds
    pub fn set_delay_time(&mut self, delay_time: f32) {
        let delay_in_samples: i32 = delay_time as i32;
        //   adding some 4 samples padding just to be sure.
        self.size_goal = (delay_in_samples.clamp(4, self.buffer_size as i32 - 4)) as i32;
        if self.interpolation_mode == InterpolationMode::None {
            self.read = self.write - self.size_goal as f32;
            if self.read < 0. {
                self.read = self.buffer_size as f32 - self.read;
            }
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_freezed(&mut self, freezed: bool) {
        // avoid updating the self.size_on_freeze
        if !self.freezed {
            self.size_on_freeze = self.actual_size;
        }
        self.freezed = freezed;
    }

    fn freezed_update_step_size(&mut self) {
        self.step_size = self.size_on_freeze / self.size_goal as f32;
    }

    fn check_for_read_index_overflow(&mut self) {
        if self.read < 0. {
            self.read += self.buffer_size as f32;
        }
        if self.read > self.buffer_size as f32 {
            self.read -= self.buffer_size as f32;
        }
    }

    fn fractionalize_read_index(&mut self) {
        // get sample
        self.i_read = self.read.floor() as i32;
        // get fraction
        self.frac = self.read - (self.i_read as f32);
        // Get next sample
        self.i_read_next = (self.i_read + 1) % (self.buffer_size - 1);
    }

    fn freeze_increment_read_pointer(&mut self) {
        self.read += self.step_size;
        // buffer over and under flow
        self.check_for_read_index_overflow();
        self.actual_size -= self.step_size;

        // In freezed case, self.read only iterate on the last buffer size,
        //  hence it's like a little ringBuffer in the bigger ringBuffer
        //  so more buffer over and under flow
        if self.actual_size < 0. {
            self.read -= self.write - self.size_on_freeze;
            self.check_for_read_index_overflow();
            self.actual_size = self.size_on_freeze;
        } else if self.actual_size > self.size_on_freeze {
            self.read = self.write;
            self.actual_size = 0.;
        }
    }
}

pub struct SimpleRingBuffer {
    buffer: Vec<f32>,
    read_index: usize,
    write_index: usize,
    size: usize,
    feedback: f32,
    max_output_value: f32,
}

impl SimpleRingBuffer {
    pub fn new(size: usize, delay: usize, feedback: f32) -> Self {
        assert!(delay < size, "delay should be inferior to maximum size");
        Self {
            buffer: vec![0.; size],
            read_index: 0,
            write_index: delay,
            size: size,
            feedback: feedback,
            max_output_value: 255.0,
        }
    }
    pub fn process(&mut self, input_sample: f32) -> f32 {
        let output = self.buffer[self.read_index];
        let feedback = output * self.feedback;
        // feedback = (feedback / self.max_output_value / 2.0).tanh() * self.max_output_value;
        let to_write = (input_sample as u8).wrapping_add(feedback as u8);
        self.buffer[self.write_index] = to_write as f32;
        self.increment();
        if self.read_index==self.write_index{ // need to read before writing for feedback, thus feedback 0 won't work
            return input_sample;
        }
        return output;
    }

    fn increment(&mut self) {
        self.read_index = (self.read_index + 1) % self.size;
        self.write_index = (self.write_index + 1) % self.size;
    }

    pub fn flush(&mut self) {
        self.buffer.iter_mut().map(|x| *x = 0.0).count();
    }

    pub fn set_delay(&mut self, mut delay: usize) {
        if delay >= self.size {delay = self.size-1}
        self.read_index = 0;
        self.write_index = delay;
        self.flush();
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }
}

pub static MAXIMUM_DELAY_TIME: f32 = 10.;
pub static MINIMUM_DELAY_TIME: f32 = 0.01;

pub enum DelayMode {
    //flat frequency feedback
    Allpass,
    //basic feedback
    Comb,
}
// #[derive(Clone, Copy)]
pub struct DelayLine {
    buffer: RingBuffer,
    feedback: f32,
    delay_time: f32,
    delay_mode: DelayMode,
}

impl DelayLine {
    pub fn set_rt60(&mut self, rt60: f32) {
        self.feedback = rt60_to_gain(rt60, self.delay_time)
    }
    //max_time in seconds
    pub fn new(max_time: f32, mode: DelayMode) -> Self {
        DelayLine {
            buffer: RingBuffer::new(max_time),
            feedback: 0.5,
            delay_mode: mode,
            delay_time: max_time,
        }
    }

    pub fn flush(&mut self) {
        self.buffer.flush();
    }

    pub fn init(&mut self, sample_rate: f32) {
        self.buffer.init(sample_rate);
    }

    pub fn init_delay(&mut self, sample_rate: f32, delay: f32) {
        self.buffer.init_delay(sample_rate, delay);
    }

    pub fn process(&mut self, input_sample: f32) -> f32 {
        let delay = self.buffer.read_sample();
        // delay = delay.clamp(-1.0, 1.0);
        let mut buf_in = 0.0;
        let mut buf_out = 0.0;
        // buffer.writeSample(buf_in);
        // return buf_out;
        match self.delay_mode {
            DelayMode::Allpass => {
                // float buf_in = (delay * m_gain) + input;
                // float buf_out = delay + (input * -m_gain);
                buf_in = (input_sample as u8).wrapping_add((delay * self.feedback) as u8) as f32;
                buf_out = (input_sample as u8).wrapping_add((delay * -self.feedback) as u8) as f32;
            }
            DelayMode::Comb => {
                //buf_in = input_sample + delay * feedback
                buf_out = delay;
                buf_in = (input_sample as u8).wrapping_add((delay * self.feedback) as u8) as f32;
            }
        }

        if self.delay_time == 0.0 {
            buf_out = buf_in;
        }

        self.buffer.write_sample(buf_in);
        buf_out
    }
    ///time in seconds
    pub fn set_delay_time(&mut self, delay_time: f32) {
        self.delay_time = delay_time;
        self.buffer.set_delay_time(delay_time);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    pub fn set_freeze(&mut self, freeze: bool) {
        self.buffer.set_freezed(freeze)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_sample_delay() {
        let mut delay_line = DelayLine::new(1000.0, DelayMode::Comb);
        delay_line.init_delay(1.0, 1.0);
        let output_sample_one = delay_line.process(255.0);
        let output_sample_two = delay_line.process(255.0);
        assert_eq!(0.0, output_sample_one);
        assert_eq!(255.0, output_sample_two);
    }

    #[test]
    fn simple_ringbuffer() {
        let mut ringbuffer = SimpleRingBuffer::new(300, 4, 0.0);
        let sample_one = ringbuffer.process(255.0);
        let sample_two = ringbuffer.process(0.0);
        let sample_three = ringbuffer.process(0.0);
        let sample_four = ringbuffer.process(0.0);
        let sample_five = ringbuffer.process(0.0);

        // 0.0 0.0 0.0 0.0 0.0
        assert_eq!(sample_one, 0.0);
        assert_eq!(sample_two, 0.0);
        assert_eq!(sample_three, 0.0);
        assert_eq!(sample_four, 0.0);
        assert_eq!(sample_five, 255.0);

        ringbuffer.set_delay(1);

        let sample_six = ringbuffer.process(255.0);
        let sample_seven = ringbuffer.process(0.0);

        assert_eq!(sample_six, 0.0);
        assert_eq!(sample_seven, 255.0);
    }

    #[test]
    fn set_delay() {
        let mut ringbuffer = SimpleRingBuffer::new(300, 4, 0.0);
        let mut sample_one = ringbuffer.process(255.0);
        let mut sample_two = ringbuffer.process(0.0);
        let mut sample_three = ringbuffer.process(0.0);
        let mut sample_four = ringbuffer.process(0.0);
        let mut sample_five = ringbuffer.process(0.0);

        ringbuffer.set_delay(3);
        sample_one = ringbuffer.process(255.0);
        sample_two = ringbuffer.process(0.0);
        sample_three = ringbuffer.process(0.0);
        sample_four = ringbuffer.process(0.0);
        sample_five = ringbuffer.process(0.0);

        // 0.0 0.0 0.0 0.0 0.0
        assert_eq!(sample_one, 0.0);
        assert_eq!(sample_two, 0.0);
        assert_eq!(sample_three, 0.0);
        assert_eq!(sample_four, 255.0);
        assert_eq!(sample_five, 0.0);

        ringbuffer.set_delay(1);

        let sample_six = ringbuffer.process(255.0);
        let sample_seven = ringbuffer.process(0.0);

        assert_eq!(sample_six, 0.0);
        assert_eq!(sample_seven, 255.0);
    }
}
