use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub struct LuxError<'a> {
    msg: &'a str,
}

impl<'a> Display for LuxError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.write_str(self.msg) }
}

impl<'a> Error for LuxError<'a> {}

impl<'a> LuxError<'a> {
    pub fn boxed(msg: &'a str) -> Box<Self> { Box::new(Self { msg }) }
}
