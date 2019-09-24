use crate::Return;
use std::cell::Cell;
use libc::c_char;

impl Return<'static> for () {
    type Ext = u8;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        0
    }
}

impl Return<'static> for bool {
    type Ext = u8;
    type Env = Cell<u32>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        val as u8
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
