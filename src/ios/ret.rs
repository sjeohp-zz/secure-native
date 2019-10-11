use crate::Return;
use libc::c_char;
use std::cell::Cell;

#[repr(C)]
pub struct CResult<T> {
    pub value: T,
    pub error_msg: *mut c_char,
}

impl Return<'static> for () {
    type Ext = *mut std::ffi::c_void;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        std::ptr::null_mut()
    }
}

impl Return<'static> for bool {
    type Ext = *mut u8;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        Box::into_raw(Box::new(val as u8))
    }
}

impl Return<'static> for String {
    type Ext = *mut c_char;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        ffi_support::rust_string_to_c(val)
    }
}

impl<Inner: Return<'static, Env = Cell<u32>> + Default> Return<'static> for Option<Inner> {
    type Ext = Inner::Ext;
    type Env = Inner::Env;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        let val = match val {
            Some(inner) => inner,
            None => {
                env.set(1);
                Inner::default()
            }
        };
        Return::convert(env, val)
    }
}

impl<Inner: Return<'static, Env = Cell<u32>> + Default> Return<'static> for Result<Inner, String> {
    type Ext = Inner::Ext;
    type Env = Inner::Env;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        let val = match val {
            Ok(inner) => inner,
            Err(e) => {
                env.set(1);
                Inner::default()
            }
        };
        Return::convert(env, val)
    }

    fn convert_cresult(env: &Self::Env, val: Self) -> *mut CResult<Self::Ext> {
        match val {
            Ok(inner) => {
                Box::into_raw(Box::new(CResult {
                    error_msg: Return::convert(env, String::default()),
                    value: Return::convert(env, inner),
                }))
            }
            Err(e) => {
                env.set(1);
                Box::into_raw(Box::new(CResult {
                    error_msg: Return::convert(env, e),
                    value: Return::convert(env, Inner::default()),
                }))
            }
        }
    }
}
