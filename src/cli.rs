use std::{ffi::OsString, path::PathBuf};

use lexopt::prelude::*;

pub enum Mode {
    Up,
    Down,
}

impl TryFrom<&OsString> for Mode {
    type Error = ();

    fn try_from(value: &OsString) -> Result<Self, Self::Error> {
        value
            .to_str()
            .and_then(|v| match v {
                "up" => Some(Self::Up),
                "down" => Some(Self::Down),
                _ => None,
            })
            .ok_or(())
    }
}

pub struct Opts {
    pub mode: Mode,
    pub start_path: PathBuf,
    pub num_steps: f64,
}

const DEFAULT_NUM_STEPS: u32 = 75;
fn default_path() -> &'static str {
    "/sys/class/backlight"
}

pub fn help() {
    println!(
        r#"Usage: lux up|down [-p|--path <PATH={}>] [-n|--num-steps <STEPS={:.0}]"#,
        default_path(),
        DEFAULT_NUM_STEPS
    );
    std::process::exit(0);
}

pub fn parse_opts() -> Result<Opts, lexopt::Error> {
    let def_path = PathBuf::from(default_path());
    let mut mode = None;
    let mut start_path = None;
    let mut num_steps = None;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Value(val) => mode = Mode::try_from(&val).ok(),
            Short('p') | Long("path") => {
                start_path = Some(PathBuf::from(parser.value()?));
            }
            Short('n') | Long("num-steps") => {
                num_steps = Some(parser.value()?.parse::<u32>()? as f64);
            }
            Short('h') | Long("help") => help(),
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Opts {
        mode: mode.ok_or("missing mode")?,
        start_path: start_path.unwrap_or(def_path),
        num_steps: num_steps.unwrap_or(DEFAULT_NUM_STEPS as f64),
    })
}
