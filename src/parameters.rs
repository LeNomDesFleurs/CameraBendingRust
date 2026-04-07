use std::cmp::{max, min};




pub trait Parameter{
    fn build_string(&self) -> String;
    fn increment(&mut self);
    fn decrement(&mut self);
}

#[derive(Clone, Copy)]

pub struct Mode<T: 'static + Copy + Clone> {
    pub name: &'static str,
    pub options_text: &'static [&'static str],
    pub options_enum: &'static [T],
    pub options_quantity: usize,
    pub value: usize,
}

impl<T: Copy + Clone> Mode<T> {
    pub fn new(name: &'static str, options_text: &'static [&'static str], options_enum: &'static [T] ) -> Self {
        assert_eq!(options_enum.len(), options_text.len(), "must have one option per text");
        Self {
            name,
            options_text,
            options_enum,
            options_quantity : options_enum.len(),
            value: 0,
        }
    }

    pub fn get(&self)->&T{
        & self.options_enum[self.value]
    }
}

impl<T: Copy + Clone> Parameter for Mode<T> {
    fn build_string(&self) -> String {
        self.name.to_string() +" - "+ self.options_text[self.value]
    }
    
    fn decrement(&mut self) {
        self.value = self.value.saturating_sub(1);
    }
    fn increment(&mut self) {
        self.value = self.value + 1;
        self.value = min(self.value, self.options_quantity-1);
    }
}

#[derive(Clone, Copy)]

pub struct Slider {
    pub name: &'static str,
    pub min: i32,
    pub max: i32,
    pub value: i32,
    pub step: i32,
}

impl Slider {
    pub fn new(name: &'static str, min: i32, max: i32, default: i32, step: i32) -> Self {
        Self {
            name,
            min,
            max,
            value: default,
            step,
        }
    }

    pub fn get(&self)->i32{
        self.value
    }
}

impl Parameter for Slider {
    fn build_string(&self) -> String {
        self.name.to_string()+ " - " + &self.value.to_string()
    }
    fn decrement(&mut self) {
        self.value = max(self.value - self.step, self.min);
        
    }
    fn increment(&mut self) {
        self.value = min(self.value + self.step, self.max);
    }

}


#[derive(Clone, Copy)]
pub struct Toggle {
    pub name: &'static str,
    pub value: bool,
}

impl Toggle {
    pub fn new(name: &'static str, default: bool) -> Self {
        Self {
            name,
            value: default,
        }
    }

    pub fn get(&self)->bool{
        self.value
    }
}

impl Parameter for Toggle {
    fn build_string(&self) -> String {
        self.name.to_string() + " - " + &self.value.to_string()
    }
    fn decrement(&mut self) {
        self.value = !self.value;
    }
    fn increment(&mut self) {
        self.value = !self.value;
    }
}



#[derive(Copy, Clone)]
pub enum ColorMode{
    Bayer,
    Interleaved,
    Composite,
}

#[derive(Copy, Clone, PartialEq)]
pub enum AlphaMode{
    Preserve,
    Delete,
    Interleave, //does nothing in bayer mode
}

#[derive(Copy, Clone)]
pub enum OrderMode{
    Column,
    Row,
    ReverseRow,
    ReverseColumn,
}

#[derive(Copy, Clone)]
pub struct Parameters {
    // signal params
    pub alpha_mode: Mode<AlphaMode>,
    pub color_mode: Mode<ColorMode>,
    pub order_mode: Mode<OrderMode>,
    pub delay_time : Slider,
    pub delay_feedback : Slider,
    pub filter_cutoff : Slider,
    pub filter_resonance : Slider,
    pub continuous : Toggle,

    pub parameter_amount: u32,
}

impl Parameters {
    pub fn new() -> Self {
        let mut temp =  Self {
                alpha_mode: Mode::new(
                    "Alpha Mode",
                    &[
                        "Delete",
                        "Preserve",
                        "Interleave",
                    ],
                    &[
                        AlphaMode::Delete,
                        AlphaMode::Preserve,
                        AlphaMode::Interleave,
                    ]
                ),
                color_mode : Mode::new(
                    "Color Mode",
                    &[
                        "Composite",
                        "Interleaved",
                        "Bayer",
                    ],
                    &[
                        ColorMode::Composite,
                        ColorMode::Interleaved,
                        ColorMode::Bayer,
                    ],
                ),
                order_mode : Mode::new(
                    "Order Mode",
                    &[
                        "Row",
                        "Column", 
                        ],
                    &[
                        OrderMode::Row,
                        OrderMode::Column, 
                        ]
                ),
                delay_time : Slider::new("Delay", 0, 5000, 0, 1),
                delay_feedback : Slider::new("Feedback", 0, 1000, 0, 1),
                continuous : Toggle::new("Continuous", false),
                filter_cutoff : Slider::new("Cutoff", 20, 100000, 100000, 1000),
                filter_resonance : Slider::new("Resonance", 0, 2000, 1, 1),
                parameter_amount: 0,

        };
        temp.count_param();
        temp
    }

    pub fn count_param(&mut self){
        let mut count=0;
        self.each_mut(|f|{
            count = count+1;
        });
        self.parameter_amount = count;
    }

    pub fn each_mut(&mut self , mut f : impl FnMut(&mut dyn Parameter)){
        f(&mut self.alpha_mode);
        f(&mut self.color_mode);
        f(&mut self.order_mode);
        f(&mut self.delay_time);
        f(&mut self.delay_feedback);
        f(&mut self.filter_cutoff);
        f(&mut self.filter_resonance );
        f(&mut self.continuous );
    }
}
