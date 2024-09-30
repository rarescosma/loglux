mod cli;
mod controller;
mod stepper;

use std::{
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener},
    },
    process,
};

use cli::*;
use controller::Controller;
use stepper::{Bounded, Stepper};

type LuxErr = Box<dyn std::error::Error + Send + Sync + 'static>;
type LuxRes<T> = Result<T, LuxErr>;

pub fn main() -> LuxRes<()> {
    // there can be only one
    let s = SocketAddr::from_abstract_name("loglux_lock".as_bytes())?;
    UnixListener::bind_addr(&s).unwrap_or_else(|_| {
        process::exit(2);
    });

    let mut opts = parse_opts().unwrap_or_else(|e| {
        eprintln!("error parsing arguments: {}", e);
        help();
        process::exit(1);
    });

    let mode = opts.mode;

    let controller = match Controller::from_opts(&mut opts) {
        Some(c) => c,
        None => {
            eprintln!("could not find any controller under {}", &opts.start_path.display());
            process::exit(1)
        }
    };

    let new_brightness = match mode {
        Mode::Up => controller.step_up(),
        Mode::Down => controller.step_down(),
    };
    if new_brightness != controller.current() {
        controller
            .set_brightness(new_brightness)
            .and_then(|_| controller.notify(new_brightness))?;
    }

    Ok(())
}
