#![feature(trace_macros, option_flattening)]
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

#[cfg(target_os = "ios")]
pub use ffi_support;

#[macro_export]
macro_rules! define_cresult_destructor {
    ( $x:ident, $y:ty ) => {
        #[cfg(target_os = "ios")]
        #[no_mangle]
        pub extern "C" fn $x(x: *mut $crate::ios::CResult<<$y as $crate::Return<'static>>::Ext>) {
            unsafe {
                Box::from_raw(x);
            }
        }
    };
}

/// Trait for converting Rust types into FFI return values
pub trait Return<'a>: Sized {
    type Ext;
    type Env;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext;
    fn convert_without_exception(env: &Self::Env, val: Self) -> Self::Ext {
        Return::convert(env, val)
    }
    #[cfg(target_os = "ios")]
    fn convert_cresult(env: &Self::Env, val: Self) -> *mut ios::CResult<Self::Ext> {
        panic!()
    }
}

/// Trait for converting FFI arguments into Rust types
pub trait Argument<'a> {
    type Ext;
    type Env;
    fn convert(env: &Self::Env, val: Self::Ext) -> Self;
}

#[macro_export]
macro_rules! export_put {
    ($( @$jname:ident fn $name:ident($s:ident : Result<(), String>$(, $a:ident : $t:ty )*) -> $ret:ty $code:block )*) => {
        $(
            pub fn $name($s: Result<(), String>, $( $a: $t ),*) -> $ret $code
        )*

        #[cfg(target_os = "android")]
        pub mod android_put {
            use $crate::android::*;
            use $crate::{Return, Argument};

            use jni::JNIEnv;
            use jni::objects::{JClass, JObject, JString};
            use jni::sys::jboolean;

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, app: JString, key: JString, value: JString, with_biometry: jboolean, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let success = $crate::android::put(&env, activity, app, key, value, with_biometry);
                    let ret = super::$name(success, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_put {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::cell::Cell;
            use libc::{c_uint, c_uchar};
            use $crate::ffi_support::FfiStr;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, app: FfiStr, key: FfiStr, value: FfiStr, with_biometry: c_uchar, $( $a: <$t as Argument<'static>>::Ext ),*) -> *mut CResult<<$ret as Return<'static>>::Ext> {
                    let error = Cell::new(0);
                    let success = $crate::ios::put(app.as_str(), key.as_str(), value.as_str(), with_biometry != 0);
                    let ret = super::$name(success, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert_cresult(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! export_get {
    ($( @$jname:ident fn $name:ident($s:ident : Result<String, String>$(, $a:ident : $t:ty )*) -> $ret:ty $code:block )*) => {
        $(
            pub fn $name($s: Result<String, String>, $( $a: $t ),*) -> $ret $code
        )*

        #[cfg(target_os = "android")]
        pub mod android_get {
            use $crate::android::*;
            use $crate::{Return, Argument};

            use jni::JNIEnv;
            use jni::objects::{JClass, JObject, JString};

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, app: JString, key: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::get(&env, activity, app, key);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_get {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::cell::Cell;
            use libc::c_uint;
            use $crate::ffi_support::FfiStr;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, app: FfiStr, key: FfiStr, $( $a: <$t as Argument<'static>>::Ext ),*) -> *mut CResult<<$ret as Return<'static>>::Ext> {
                    let error = Cell::new(0);
                    let res = $crate::ios::get(app.as_str(), key.as_str());
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert_cresult(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! export_contains {
    ($( @$jname:ident fn $name:ident($s:ident : Result<bool, String>$(, $a:ident : $t:ty )*) -> $ret:ty $code:block )*) => {
        $(
            pub fn $name($s: Result<bool, String>, $( $a: $t ),*) -> $ret $code
        )*

        #[cfg(target_os = "android")]
        pub mod android_contains {
            use $crate::android::*;
            use $crate::{Return, Argument};

            use jni::JNIEnv;
            use jni::objects::{JClass, JObject, JString};

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, app: JString, key: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::contains(&env, activity, app, key);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_contains {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::cell::Cell;
            use libc::c_uint;
            use $crate::ffi_support::FfiStr;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, app: FfiStr, key: FfiStr, $( $a: <$t as Argument<'static>>::Ext ),*) -> *mut CResult<<$ret as Return<'static>>::Ext> {
                    let error = Cell::new(0);
                    let res = $crate::ios::contains(app.as_str(), key.as_str());
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert_cresult(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! export_delete {
    ($( @$jname:ident fn $name:ident($s:ident : Result<(), String>$(, $a:ident : $t:ty )*) -> $ret:ty $code:block )*) => {
        $(
            pub fn $name($s: Result<(), String>, $( $a: $t ),*) -> $ret $code
        )*

        #[cfg(target_os = "android")]
        pub mod android_delete {
            use $crate::android::*;
            use $crate::{Return, Argument};

            use jni::JNIEnv;
            use jni::objects::{JClass, JObject, JString};

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, app: JString, key: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::delete(&env, activity, app, key);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_delete {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::cell::Cell;
            use libc::c_uint;
            use $crate::ffi_support::FfiStr;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, app: FfiStr, key: FfiStr, $( $a: <$t as Argument<'static>>::Ext ),*) -> *mut CResult<<$ret as Return<'static>>::Ext> {
                    let error = Cell::new(0);
                    let res = $crate::ios::delete(app.as_str(), key.as_str());
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert_cresult(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[cfg(test)]
mod tests {
    //    trace_macros!(true);

    export_put! {
        @Java_io_parity_secure_native_test_put
        fn test_put(success: Result<(), String>, other: u32) -> Result<(), String> {
            success
        }
    }

    export_get! {
        @Java_io_parity_secure_native_test_get
        fn test_get(seed: Result<String, String>, other: u32) -> Result<String, String> {
            seed
        }
    }

    export_contains! {
        @Java_io_parity_secure_native_test_contains
        fn test_contains(contained: Result<bool, String>, other: u32) -> Result<bool, String> {
            contained
        }
    }

    export_delete! {
        @Java_io_parity_secure_native_test_delete
        fn test_delete(success: Result<(), String>, other: u32) -> Result<(), String> {
            success
        }
    }

    #[test]
    fn test_exports() {}
}
