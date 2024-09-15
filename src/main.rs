mod cli;

use std::{
    fs,
    fs::File,
    io::{Error as IoError, ErrorKind, Write},
    os::{
        linux::net::SocketAddrExt,
        unix::{
            fs::FileExt,
            net::{SocketAddr, UnixListener},
        },
    },
    path::PathBuf,
    process, str,
};

use cli::{help, parse_opts, Mode};

type Res<T> = Result<T, IoError>;
type Brightness = u32;

const BUFFER_SIZE: usize = 30;

struct Controller {
    path: PathBuf,
    max_b: Brightness,
    b: Brightness,
}

fn int_from_file(p: PathBuf) -> Option<u32> {
    let mut b_buf = [b' '; BUFFER_SIZE];
    let _ = File::open(p).ok().and_then(|f| f.read_at(&mut b_buf, 0).ok());

    str::from_utf8(&b_buf).ok().and_then(|x| x.trim().parse::<u32>().ok())
}

impl Controller {
    fn set_brightness(&mut self, new_b: Brightness) -> Res<()> {
        let mut tee = process::Command::new("sudo")
            .arg("tee")
            .arg(self.path.join("brightness"))
            .stdin(process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = tee.stdin.as_mut() {
            stdin.write_all(format!("{}", new_b).as_bytes())?;
        }

        tee.wait()?;
        self.b = new_b;
        Ok(())
    }

    fn notify(&self) -> Res<()> {
        process::Command::new("notify-send")
            .arg(self.name()?)
            .args([
                "-h",
                &format!("int:value:{}", self.b * 100 / self.max_b),
                "-h",
                "string:synchronous:volume",
            ])
            .output()?;
        Ok(())
    }

    fn name(&self) -> Result<&str, IoError> {
        self.path.file_name().and_then(|f| f.to_str()).ok_or(IoError::from(ErrorKind::Other))
    }

    fn current_step(&self, num_steps: f64) -> isize {
        (num_steps * (self.b.max(1) as f64).log(self.max_b as f64)).round() as _
    }

    fn b_from_step(&self, step_no: isize, num_steps: f64) -> Brightness {
        (self.max_b as f64).powf(step_no as f64 / num_steps) as _
    }

    fn step_up(&self, num_steps: f64) -> Brightness {
        let mut step = self.current_step(num_steps);
        let mut new_b = self.b;

        while new_b <= self.b {
            step += 1;
            new_b = self.b_from_step(step, num_steps);
        }
        new_b.min(self.max_b)
    }

    fn step_down(&self, num_steps: f64) -> Brightness {
        let mut step = self.current_step(num_steps);
        let mut new_b = self.b;

        while new_b >= self.b && step >= 0 {
            step -= 1;
            new_b = self.b_from_step(step, num_steps);
        }

        new_b
    }
}

fn best_controller(start_path: &PathBuf) -> Option<Controller> {
    let mut path: Option<PathBuf> = None;
    let mut best_max = 0;

    for entry in fs::read_dir(start_path).ok()?.flatten() {
        let c_path = entry.path();
        if let Some(max_b) = int_from_file(c_path.join("max_brightness")) {
            if max_b > best_max {
                best_max = max_b;
                path = Some(c_path);
            }
        }
    }

    path.and_then(|path| {
        let b_path = path.join("brightness");
        Some(Controller { path, max_b: best_max, b: int_from_file(b_path)? })
    })
}

pub fn main() -> Res<()> {
    let opts = {
        let i_opts = parse_opts();
        if i_opts.is_err() {
            help()
        }
        i_opts.unwrap()
    };

    // there can be only one
    let s = SocketAddr::from_abstract_name("lux_lock".as_bytes())?;
    if UnixListener::bind_addr(&s).is_err() {
        process::exit(2);
    }

    if let Some(mut controller) = best_controller(&opts.start_path) {
        let _ = match opts.mode {
            Mode::Up => Some(controller.step_up(opts.num_steps)),
            Mode::Down => Some(controller.step_down(opts.num_steps)),
        }
        .and_then(|b| if controller.b != b { controller.set_brightness(b).ok() } else { None })
        .and_then(|_| controller.notify().ok());
    }

    Ok(())
}
