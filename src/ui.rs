use std::cmp::min;

pub use crate::parameters::Parameters;
use crossterm::{
    cursor, event, event::Event, event::KeyCode, event::KeyEvent, event::KeyModifiers,
    style::Stylize, terminal, terminal::disable_raw_mode, terminal::enable_raw_mode,
};

pub struct Ui {
    position: u32,
    parameter_amount: u32,
    // TODO increment value
}

impl Ui {
    pub fn new(parameters: &mut Parameters) -> Self {
        Self {
            parameter_amount: parameters.parameter_amount,
            position: 0,
        }
    }

    pub fn increment_parameter(&mut self, parameters: &mut Parameters) {
        let mut count = 0;
        parameters.each_mut(|param| {
            if count == self.position {
                param.increment()
            }
            count = count + 1;
        });
    }

    pub fn decrement_parameter(&mut self, parameters: &mut Parameters) {
        let mut count = 0;
        parameters.each_mut(|param| {
            if count == self.position {
                param.decrement()
            }
            count = count + 1;
        });
    }

    pub fn increment_position(&mut self) {
        self.position = min(self.position + 1, self.parameter_amount-1);
        
    }

    pub fn decrement_position(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    pub fn update_display(&mut self, parameters: &mut Parameters) {
        let mut count = 0;
        println!("{}", terminal::Clear(terminal::ClearType::All));
        println!("{}", cursor::MoveTo(0, 0));
        parameters.each_mut(|param| {
            if count == self.position {
                print!("{}", param.build_string().bold().italic());
            } else {
                print!("{}", param.build_string());
            }
            count = count + 1;
            print! {"\r\n"};
        });
    }
}
