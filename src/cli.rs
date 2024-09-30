use std::path::PathBuf;

use lexopt::{prelude::*, Error};

#[derive(Copy, Clone)]
pub enum Mode {
    Up,
    Down,
}

pub struct Opts {
    pub mode: Mode,
    pub start_path: PathBuf,
    pub num_steps: u64,
}

const DEFAULT_NUM_STEPS: u64 = 75;
fn default_path() -> &'static str { "/sys/class/backlight" }

pub fn help() {
    println!(
        r#"Usage: loglux up|down [-p|--path (default: {})] [-n|--num-steps (default: {:.0})]"#,
        default_path(),
        DEFAULT_NUM_STEPS
    );
    std::process::exit(0);
}

pub fn parse_opts() -> Result<Opts, Error> {
    let def_path = PathBuf::from(default_path());
    let mut mode = Err(Error::from("missing mode"));
    let mut start_path = Ok(def_path);
    let mut num_steps = Ok(DEFAULT_NUM_STEPS);

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) => {
                if val == "up" {
                    mode = Ok(Mode::Up);
                } else if val == "down" {
                    mode = Ok(Mode::Down)
                } else {
                    mode = Err(Error::from(format!("invalid mode: {:?}", val)))
                }
            }
            Short('p') | Long("path") => {
                start_path = parser.value().map(PathBuf::from);
            }
            Short('n') | Long("num-steps") => {
                num_steps = parser.value().and_then(|v| {
                    v.parse::<u64>()
                        .map_err(|e| Error::from(format!("invalid number of steps: {e}")))
                });
            }
            Short('h') | Long("help") => help(),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Opts { mode: mode?, start_path: start_path?, num_steps: num_steps? })
}
