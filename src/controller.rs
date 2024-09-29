use std::{
    fs::{read_dir, File},
    io::{Error as IoError, ErrorKind, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    process, str,
};

use crate::{stepper::Bounded, Res};

const BUFFER_SIZE: usize = 32;

#[derive(Clone)]
pub struct Controller {
    path: PathBuf,
    max_brightness: u64,
    brightness: u64,
}

impl Bounded for Controller {
    fn current(&self) -> u64 { self.brightness }
    fn max(&self) -> u64 { self.max_brightness }
    fn with_current(&self, brightness: u64) -> Self { Self { brightness, ..self.clone() } }
}

impl Controller {
    pub fn new(path: PathBuf, max_brightness: u64, brightness: u64) -> Self {
        Self { path, max_brightness, brightness }
    }

    pub fn set_brightness(&self, new_b: u64) -> Res<()> {
        let mut tee = process::Command::new("sudo")
            .arg("tee")
            .arg(self.path.join("brightness"))
            .stdin(process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = tee.stdin.as_mut() {
            stdin.write_all(format!("{}", new_b).as_bytes())?;
        }

        tee.wait()?;
        Ok(())
    }

    pub fn notify(&self, new_b: u64) -> Res<()> {
        process::Command::new("notify-send")
            .arg(self.name()?)
            .args([
                "-h",
                &format!("int:value:{}", new_b * 100 / self.max_brightness),
                "-h",
                "string:synchronous:volume",
            ])
            .output()?;
        Ok(())
    }

    fn name(&self) -> Result<&str, IoError> {
        self.path.file_name().and_then(|f| f.to_str()).ok_or(IoError::from(ErrorKind::Other))
    }
}

pub fn best_controller(start_path: &PathBuf) -> Option<Controller> {
    let mut path: Option<PathBuf> = None;
    let mut max_brightness = 0;

    if let (Some(max_brightness), Some(brightness)) = (
        val_from_file(start_path.join("max_brightness")),
        val_from_file(start_path.join("brightness")),
    ) {
        return Some(Controller::new(start_path.to_owned(), max_brightness, brightness));
    }

    for entry in read_dir(start_path).ok()?.flatten() {
        let c_path = entry.path();
        if let Some(max_b) = val_from_file(c_path.join("max_brightness")) {
            if max_b > max_brightness {
                max_brightness = max_b;
                path = Some(c_path);
            }
        }
    }

    path.and_then(|path| {
        let brightness = val_from_file(path.join("brightness"))?;
        Some(Controller::new(path, max_brightness, brightness))
    })
}

fn val_from_file<V, P>(file: P) -> Option<V>
where
    V: str::FromStr,
    P: AsRef<Path>,
{
    let mut b_buf = [b' '; BUFFER_SIZE];
    let _ = File::open(file).ok().and_then(|f| f.read_at(&mut b_buf, 0).ok());

    str::from_utf8(&b_buf).ok().and_then(|x| x.trim().parse::<V>().ok())
}
