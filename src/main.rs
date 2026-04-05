// use time::now;

use std::env;
mod filter;
mod outils;
pub use filter::Biquad;
pub use filter::FilterType;

mod parameters;
pub use parameters::Parameters;
mod ui;
// mod processor;
// pub use processor::Processor;
pub use ui::Ui;

mod buffer;
pub use buffer::DelayLine;

use crossterm::execute;
use crossterm::{
    cursor, event, event::Event, event::KeyCode, event::KeyEvent, style::Stylize,
    event::KeyModifiers, terminal, terminal::disable_raw_mode,
    terminal::enable_raw_mode,
};

fn main() -> anyhow::Result<()> {
    // Collect all arguments
    let args: Vec<String> = env::args().collect();
    // Path of input file is the first argument
    // let in_path = &args[1];
    // let out_path = &args[2];
    // let in_path: &String = &"assets/rose.jpg".to_string();
    // let out_path: &String = &"test6.jpg".to_string();
    // let feedback = args[3].parse::<f32>()?;
    // let delay = args[4].parse::<f32>()?;

    let mut parameters = Parameters::new();
    // let mut processor = Processor::new(in_path.to_string(), &parameters);
    let mut ui = Ui::new(&mut parameters);

    init_terminal();
    ui.update_display(&mut parameters);

    loop {
        if let Event::Key(KeyEvent { code, .. }) =
            event::read().unwrap_or(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
        {
            //got to refresh at the end, will be modified if the event involve more modification
            match code {
                KeyCode::Esc => {
                    clean_terminal();
                    break;
                }
                KeyCode::Down => ui.increment_position(),
                KeyCode::Up => ui.decrement_position(),
                KeyCode::Right => ui.increment_parameter(&mut parameters),
                KeyCode::Left => ui.decrement_parameter(&mut parameters),
                // KeyCode::Char(char) => parameters_modified = Some(ParameterModified::SetValue(char)),
                // KeyCode::Enter => processor.process_image(),
                _ => {}
            }
            ui.update_display(&mut parameters);
        }
    }

    Ok(())
}

pub fn init_terminal() {
    enable_raw_mode().unwrap();
    execute!(std::io::stdout(), cursor::Hide).unwrap();
    println!("{}", terminal::Clear(terminal::ClearType::All));
    println!("{}", cursor::MoveTo(0, 0));
}

pub fn clean_terminal() {
    disable_raw_mode().unwrap();
    execute!(std::io::stdout(), cursor::Show).unwrap();
    println!("{}", terminal::Clear(terminal::ClearType::All));
    println!("{}", cursor::MoveTo(0, 0));
}
