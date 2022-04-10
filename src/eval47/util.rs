use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem::transmute;
use sexpr_ir::gast::symbol::Location;

use crate::eval47::commons::Signature;

#[cfg(target_pointer_width = "32")]
pub fn bitcast_usize_i64(src: usize) -> i64 {
    unsafe { transmute(src as u64) }
}

#[cfg(target_pointer_width = "32")]
pub fn bitcast_i64_usize(src: i64) -> usize {
    unsafe { transmute(src as u64 as u32) }
}

#[cfg(target_pointer_width = "64")]
pub fn bitcast_usize_i64(src: usize) -> i64 {
    unsafe { transmute(src) }
}

#[cfg(target_pointer_width = "64")]
pub fn bitcast_i64_usize(src: i64) -> usize {
    unsafe { transmute(src) }
}

pub fn read_to_string_trim_comments(path: &str) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut file = BufReader::new(file);
    let mut content = String::new();
    let mut line = String::new();

    while file.read_line(&mut line)? > 0 {
        let parts = line.split(";").collect::<Vec<_>>();
        content.push_str(&parts[0]);
        if parts.len() > 1 {
            content.push('\n');
        }

        line.clear();
    }

    Ok(content)
}

pub use xjbutil::either::Either as Mantis;

#[derive(Clone, Debug)]
pub enum MantisGod<T1, T2, T3> {
    Left(T1),
    Middle(T2),
    Right(T3),
}

impl<T1, T2, T3> MantisGod<T1, T2, T3> {
    pub fn is_left(&self) -> bool {
        match self {
            MantisGod::Left(_) => true,
            _ => false,
        }
    }

    pub fn is_middle(&self) -> bool {
        match self {
            MantisGod::Middle(_) => true,
            _ => false,
        }
    }

    pub fn is_right(&self) -> bool {
        match self {
            MantisGod::Right(_) => true,
            _ => false,
        }
    }

    pub fn left(self) -> Option<T1> {
        match self {
            MantisGod::Left(x) => Some(x),
            _ => None,
        }
    }

    pub fn middle(self) -> Option<T2> {
        match self {
            MantisGod::Middle(x) => Some(x),
            _ => None,
        }
    }

    pub fn right(self) -> Option<T3> {
        match self {
            MantisGod::Right(x) => Some(x),
            _ => None,
        }
    }
}

pub fn clone_signature(signature: &Signature) -> Signature {
    Signature {
        func_type: signature.func_type,
        param_options: signature.param_options.iter().map(|x| *x).collect(),
        ret_option: signature.ret_option.iter().map(|x| *x).collect()
    }
}

pub struct Guard {
    pos: Option<Location>,
    inner: String,
    cancelled: bool
}

impl Guard {
    pub fn new(pos: Option<Location>, inner: String) -> Self {
        Guard {
            pos,
            inner,
            cancelled: false
        }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if !self.cancelled {
            if let Some(pos) = self.pos.as_ref() {
                eprintln!(
                    ".. at file \"{}\", line {}, column {}: when performing: {}",
                    pos.path.as_str(),
                    pos.line,
                    pos.colum,
                    self.inner
                );
            } else {
                eprintln!(".. when performing: {}", self.inner);
            }
        }
    }
}

#[macro_export]
macro_rules! guard {
    ($text:expr) => {
        Guard::new(None, $name.into())
    };
    ($fmt:expr, $($arg:tt)*) => {
        Guard::new(None, format!($fmt, $($arg)*))
    }
}

#[macro_export]
macro_rules! guard2 {
    ($pos:expr, $text:expr) => {
        Guard::new(Some($pos.clone()), $name.into())
    };
    ($pos:expr, $fmt:expr, $($arg:tt)*) => {
        Guard::new(Some($pos.clone()), format!($fmt, $($arg)*))
    }
}
