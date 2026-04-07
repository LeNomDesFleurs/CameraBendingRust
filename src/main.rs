// use time::now;

use std::env;
mod filter;
mod outils;
use crossterm::terminal::is_raw_mode_enabled;
pub use filter::Biquad;
pub use filter::FilterType;

mod parameters;
pub use parameters::Parameters;
mod ui;
mod processor;
pub use processor::Processor;
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
    let in_path: &str = &"assets/colors.png".to_string();
    // let out_path: &String = &"test6.jpg".to_string();
    // let feedback = args[3].parse::<f32>()?;
    // let delay = args[4].parse::<f32>()?;

    let mut parameters = Parameters::new();
    let mut processor = Processor::new(in_path, &parameters);
    let mut ui = Ui::new(&mut parameters);

    std::panic::set_hook(Box::new(|info| {
        clean_terminal();
        eprintln!("{info}");
    }));


    init_terminal();
    ui.update_display(&mut parameters);

    loop {
        if let Event::Key(KeyEvent { code, .. }) =
            event::read().unwrap_or(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
        {
            match code {
                KeyCode::Esc => {
                    break;
                }
                KeyCode::Down => ui.increment_position(),
                KeyCode::Up => ui.decrement_position(),
                KeyCode::Right => ui.increment_parameter(&mut parameters),
                KeyCode::Left => ui.decrement_parameter(&mut parameters),
                // KeyCode::Char(char) => parameters_modified = Some(ParameterModified::SetValue(char)),
                KeyCode::Enter => processor.process_image(&mut parameters),
                _ => {}
            }
            ui.update_display(&mut parameters);
        }
    }

    clean_terminal();

    Ok(())
}

pub fn init_terminal() {
    enable_raw_mode().unwrap();
    execute!(std::io::stdout(),
        terminal::EnterAlternateScreen,
        cursor::Hide,
    ).unwrap();
}

pub fn clean_terminal() {
    execute!(std::io::stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show,
    ).unwrap();
    disable_raw_mode().unwrap();
}