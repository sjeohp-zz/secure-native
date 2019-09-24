use crate::Argument;
use crate::{double_expect, expect};
use std::cell::Cell;
use ffi_support::FfiStr;
use libc::c_char;

impl Argument<'static> for u32 {
    type Ext = u32;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val
    }
}

impl Argument<'static> for u8 {
    type Ext = u32;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val as u8
    }
}

impl Argument<'static> for bool {
    type Ext = u8;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val != 0
    }
}

impl Argument<'static> for String {
    type Ext = *const c_char;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        unsafe { FfiStr::from_raw(val) }.into_string()
    }
}
