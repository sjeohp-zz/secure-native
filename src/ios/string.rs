#[repr(C)]
pub struct RustStringPtr {
    pub ptr: *const u8,
    pub len: size_t,
}

impl<'a> From<&'a str> for StringPtr {
    fn from(s: &'a str) -> Self {
        StringPtr {
            ptr: s.as_ptr(),
            len: s.len() as size_t,
        }
    }
}

impl StringPtr {
    pub fn as_str(&self) -> &str {
        use std::{slice, str};

        unsafe {
            let slice = slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8_unchecked(slice)
        }
    }
}

impl std::ops::Deref for StringPtr {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_string_ptr(s: *mut String) -> *mut StringPtr {
    Box::into_raw(Box::new(StringPtr::from(&**s)))
}

#[no_mangle]
pub unsafe extern "C" fn rust_string_destroy(s: *mut String) {
    let _ = Box::from_raw(s);
}

#[no_mangle]
pub unsafe extern "C" fn rust_string_ptr_destroy(s: *mut StringPtr) {
    let _ = Box::from_raw(s);
}
