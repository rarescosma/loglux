use std::{ptr, str};

use crate::BUFFER_SIZE;

pub struct GhettoConcat {
    buffer: [u8; BUFFER_SIZE],
    len: usize,
}

impl GhettoConcat {
    pub fn new(s1: &[u8], s2: &[u8]) -> Self {
        let mut buffer = [0u8; BUFFER_SIZE];
        if s1.len() + s2.len() > BUFFER_SIZE {
            panic!("no")
        }
        unsafe {
            ptr::copy_nonoverlapping(s1.as_ptr(), buffer.as_mut_ptr(), s1.len());
            ptr::copy_nonoverlapping(s2.as_ptr(), buffer.as_mut_ptr().add(s1.len()), s2.len());
        };
        Self { buffer, len: s1.len() + s2.len() }
    }

    fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.len]
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_slice()) }
    }
}
