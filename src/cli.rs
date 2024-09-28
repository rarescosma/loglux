use std::path::PathBuf;

use lexopt::prelude::*;

pub enum Mode {
    Up,
    Down,
}

pub struct Opts {
    pub mode: Mode,
    pub start_path: PathBuf,
    pub num_steps: u32,
}

const DEFAULT_NUM_STEPS: u32 = 75;
fn default_path() -> &'static str { "/sys/class/backlight" }

pub fn help() {
    println!(
        r#"Usage: loglux up|down [-p|--path (default: {})] [-n|--num-steps (default: {:.0})]"#,
        default_path(),
        DEFAULT_NUM_STEPS
    );
    std::process::exit(0);
}

pub fn parse_opts() -> Result<Opts, lexopt::Error> {
    let def_path = PathBuf::from(default_path());
    let mut mode = None;
    let mut start_path = Some(def_path);
    let mut num_steps = Some(DEFAULT_NUM_STEPS);

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) => {
                if val == "up" {
                    mode = Some(Mode::Up);
                } else if val == "down" {
                    mode = Some(Mode::Down)
                }
            }
            Short('p') | Long("path") => {
                start_path = parser.value().ok().map(PathBuf::from);
            }
            Short('n') | Long("num-steps") => {
                num_steps = parser.value().ok().and_then(|v| v.parse::<u32>().ok());
            }
            Short('h') | Long("help") => help(),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Opts {
        mode: mode.ok_or("invalid mode")?,
        start_path: start_path.ok_or("invalid path")?,
        num_steps: num_steps.ok_or("invalid number of steps")?,
    })
}
