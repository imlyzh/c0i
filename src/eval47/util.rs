use std::mem::transmute;

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
