use std::{
    fs::{read_dir, File},
    io::{Result as IoResult, Write},
    os::unix::fs::FileExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    str,
};

use crate::{cli::Opts, stepper::Bounded, LuxRes};

const BUFFER_SIZE: usize = 32;

pub struct Controller<'a> {
    path: &'a PathBuf,
    max_brightness: u64,
    brightness: u64,
    num_steps: u64,
}

impl Bounded for Controller<'_> {
    fn current(&self) -> u64 { self.brightness }
    fn max(&self) -> u64 { self.max_brightness }
    fn num_steps(&self) -> u64 { self.num_steps }
    fn with_current(&self, brightness: u64) -> Self { Self { brightness, ..*self } }
}

impl<'a> Controller<'a> {
    pub fn from_opts(opts: &'a mut Opts) -> Option<Self> {
        if let (Some(max_brightness), Some(brightness)) = (
            val_from_file(opts.start_path.join("max_brightness")),
            val_from_file(opts.start_path.join("brightness")),
        ) {
            return Some(Controller {
                path: &opts.start_path,
                max_brightness,
                brightness,
                num_steps: opts.num_steps,
            });
        }

        let mut max_brightness = 0;
        let mut found = false;

        for entry in read_dir(&opts.start_path).ok()?.flatten() {
            let c_path = entry.path();
            if let Some(max_b) = val_from_file(c_path.join("max_brightness")) {
                if max_b > max_brightness {
                    max_brightness = max_b;
                    opts.start_path = c_path;
                    found = true;
                }
            }
        }

        if found {
            let brightness = val_from_file(opts.start_path.join("brightness"))?;
            return Some(Controller {
                path: &opts.start_path,
                max_brightness,
                brightness,
                num_steps: opts.num_steps,
            });
        }
        None
    }

    pub fn set_brightness(&self, new_b: u64) -> LuxRes<()> {
        let mut tee = Command::new("sudo")
            .arg("tee")
            .arg(self.path.join("brightness"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        {
            let mut stdin = tee.stdin.take().ok_or("could not capture tee's stdin")?;
            stdin.write_all(format!("{}", new_b).as_bytes())?;
        }

        cmd_result("tee", tee.wait_with_output())
    }

    pub fn notify(&self, new_b: u64) -> LuxRes<()> {
        let output = Command::new("notify-send")
            .args([
                self.name()?,
                "-h",
                &format!("int:value:{}", new_b * 100 / self.max_brightness),
                "-h",
                "string:synchronous:volume",
            ])
            .output();

        cmd_result("notify-send", output)
    }

    fn name(&self) -> LuxRes<&str> {
        self.path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or("could not determine controller name".into())
    }
}

fn cmd_result(cmd_name: &str, output: IoResult<Output>) -> LuxRes<()> {
    match output {
        Ok(out) if out.status.success() => Ok(()),
        Ok(out) => {
            let stderr = str::from_utf8(&out.stderr).unwrap_or_default().trim();
            Err(format!("{} failed: {}", cmd_name, stderr).into())
        }
        Err(e) => Err(format!("{} failed: {e}", cmd_name).into()),
    }
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
