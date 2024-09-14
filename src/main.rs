#![no_main]

mod error;

use std::{
    error::Error,
    fs,
    io::Write,
    os::{
        linux::net::SocketAddrExt,
        unix::net::{SocketAddr, UnixListener},
    },
    path::PathBuf,
    process,
};
use std::ffi::CStr;
use crate::error::LuxError;

type Res<T> = Result<T, Box<dyn Error>>;
type Brightness = u32;

const NUM_STEPS: f64 = 100_f64;

#[derive(Debug)]
struct Controller {
    path: PathBuf,
    max_b: Brightness,
    b: Brightness,
}

impl Controller {
    fn set_brightness(&mut self, new_b: Brightness) -> Res<()> {
        let mut tee = process::Command::new("sudo")
            .arg("tee")
            .arg(self.path.join("brightness"))
            .stdin(process::Stdio::piped())
            .spawn()?;

        let mut buffer = itoa::Buffer::new();
        tee.stdin
            .as_mut()
            .ok_or(LuxError::boxed("no stdin :("))?
            .write_all(buffer.format(new_b).as_bytes())?;

        tee.wait()?;
        self.b = new_b;
        Ok(())
    }

    fn notify(&self) -> Res<()> {
        let perc = format!("int:value:{:.2}", (self.b * 100) as f64 / self.max_b as f64);
        process::Command::new("notify-send")
            .arg(self.path.file_name().unwrap().to_str().unwrap())
            .args(["-h", &perc, "-h", "string:synchronous:volume"])
            .output()?;
        Ok(())
    }

    fn current_step(&self) -> isize {
        (NUM_STEPS * (self.b.max(1) as f64).log(self.max_b as f64)).round() as _
    }

    fn b_from_step(&self, step_no: isize) -> Brightness {
        (self.max_b as f64).powf(step_no as f64 / NUM_STEPS) as _
    }

    fn step_up(&self) -> Brightness {
        let mut step = self.current_step();
        let mut new_b = self.b;

        while new_b <= self.b {
            step += 1;
            new_b = self.b_from_step(step);
        }
        new_b.min(self.max_b)
    }

    fn step_down(&self) -> Brightness {
        let mut step = self.current_step();
        let mut new_b = self.b;

        while new_b >= self.b && step >= 0 {
            step -= 1;
            new_b = self.b_from_step(step);
        }

        new_b
    }
}

fn best_controller(start_path: &PathBuf) -> Res<Controller> {
    let mut best_c: Option<PathBuf> = None;
    let mut best_max = 0;

    for entry in fs::read_dir(start_path)?.flatten() {
        let path = entry.path();
        if let Ok(contents) = fs::read_to_string(path.join("max_brightness")) {
            if let Ok(max_b) = contents.trim().parse::<u32>() {
                if max_b > best_max {
                    best_max = max_b;
                    best_c = Some(path);
                }
            }
        }
    }

    if let Some(best_c) = best_c {
        let b_path = best_c.join("brightness");
        let b = fs::read_to_string(&b_path)?.trim().parse::<u32>()?;
        return Ok(Controller { path: best_c, max_b: best_max, b });
    }
    Err(LuxError::boxed("could not find a suitable controller"))
}

fn bail() {
    process::exit(1);
}

#[no_mangle]
pub fn main(_argc: i32, _argv: *const *const i8) -> isize {
    if _argc < 2 {
        bail();
    }

    let _: Res<()> = (|| {
        // there can be only one
        let s = SocketAddr::from_abstract_name("lux_lock".as_bytes())?;
        if UnixListener::bind_addr(&s).is_err() {
            process::exit(2);
        }

        // Initialize a pointer to store the current argument
        let mut current_arg = _argv;

        // Skip the program name
        current_arg = unsafe { current_arg.offset(1) };
        if current_arg.is_null() {
            bail();
        }

        let mode =
            std::str::from_utf8(unsafe { CStr::from_ptr(*current_arg).to_bytes() }).expect("args");
        if mode != "up" && mode != "down" {
            bail();
        }

        let mut controller = best_controller(&PathBuf::from("/sys/class/backlight"))?;

        let _ = match mode {
            "up" => Some(controller.step_up()),
            "down" => Some(controller.step_down()),
            _ => None,
        }
        .and_then(|b| if controller.b != b { controller.set_brightness(b).ok() } else { None })
        .and_then(|_| controller.notify().ok());

        Ok(())
    })();
    0
}
