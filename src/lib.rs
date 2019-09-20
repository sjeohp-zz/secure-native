#![feature(trace_macros)]
#![feature(option_flattening)]
#![allow(non_snake_case, non_upper_case_globals)]

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;

/// Trait for converting Rust types into FFI return values
pub trait Return<'a> {
    type Ext;
    type Env;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext;
}

/// Trait for converting FFI arguments into Rust types
pub trait Argument<'a> {
    type Ext;
    type Env;
    fn convert(env: &Self::Env, val: Self::Ext) -> Self;
}

#[macro_export]
macro_rules! expect {
    ( $x:expr ) => {
        $x.expect(&format!("{}:{}", file!(), line!()))
    };
}

#[macro_export]
macro_rules! double_expect {
    ( $x:expr ) => {
        $x.expect(&format!("{}:{}", file!(), line!())).expect(&format!("{}:{}", file!(), line!()))
    };
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

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, service: JString, account: JString, value: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let success = $crate::android::put(&env, activity, service, account, value);
                    let ret = super::$name(success, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_put {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::{ffi::CString, cell::Cell};
            use libc::c_uint;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, service: CString, account: CString, value: CString, $( $a: <$t as Argument<'static>>::Ext ),*) -> <$ret as Return<'static>>::Ext {
                    let error = Cell::new(0);
                    let success = $crate::ios::put(Argument::convert(&error, service), Argument::convert(&error, account), Argument::convert(&error, value));
                    let ret = super::$name(success, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[macro_export]
macro_rules! export_get {
    ($( @$jname:ident fn $name:ident($s:ident : Result<Option<String>, String>$(, $a:ident : $t:ty )*) -> $ret:ty $code:block )*) => {
        $(
            pub fn $name($s: Result<Option<String>, String>, $( $a: $t ),*) -> $ret $code
        )*

        #[cfg(target_os = "android")]
        pub mod android_get {
            use $crate::android::*;
            use $crate::{Return, Argument};

            use jni::JNIEnv;
            use jni::objects::{JClass, JObject, JString};

            $(
                #[no_mangle]
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, service: JString, account: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::get(&env, activity, service, account);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_get {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::{ffi::CString, cell::Cell};
            use libc::c_uint;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, service: CString, account: CString, $( $a: <$t as Argument<'static>>::Ext ),*) -> <$ret as Return<'static>>::Ext {
                    let error = Cell::new(0);
                    let res = $crate::ios::get(Argument::convert(&error, service), Argument::convert(&error, account));
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert(&error, ret);
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
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, service: JString, account: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::contains(&env, activity, service, account);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_contains {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::{ffi::CString, cell::Cell};
            use libc::c_uint;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, service: CString, account: CString, $( $a: <$t as Argument<'static>>::Ext ),*) -> <$ret as Return<'static>>::Ext {
                    let error = Cell::new(0);
                    let res = $crate::ios::contains(Argument::convert(&error, service), Argument::convert(&error, account));
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert(&error, ret);
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
                pub extern fn $jname<'jni>(env: JNIEnv<'jni>, _: JClass, activity: JObject, service: JString, account: JString, $( $a: <$t as Argument<'jni>>::Ext ),*) -> <$ret as Return<'jni>>::Ext {
                    let res = $crate::android::delete(&env, activity, service, account);
                    let ret = super::$name(res, $( Argument::convert(&env, $a) ),*);
                    Return::convert(&env, ret)
                }
            )*
        }

        #[cfg(target_os = "ios")]
        pub mod ios_delete {
            use $crate::ios::*;
            use $crate::{Return, Argument};

            use std::{ffi::CString, cell::Cell};
            use libc::c_uint;

            $(
                #[no_mangle]
                pub extern fn $name(err: *mut c_uint, service: CString, account: CString, $( $a: <$t as Argument<'static>>::Ext ),*) -> <$ret as Return<'static>>::Ext {
                    let error = Cell::new(0);
                    let res = $crate::ios::delete(Argument::convert(&error, service), Argument::convert(&error, account));
                    let ret = super::$name(res, $(Argument::convert(&error, $a)),*);
                    let ret = Return::convert(&error, ret);
                    unsafe { *err |= error.get() as c_uint };
                    ret
                }
            )*
        }
    }
}

#[cfg(test)]
mod tests {
    trace_macros!(true);

    export_put! {
        @Java_io_parity_secure_native_test_put
        fn test_put(success: Result<(), String>, other: u32) -> Option<()> {
            success.ok()
        }
    }

    export_get! {
        @Java_io_parity_secure_native_test_get
        fn test_get(seed: Result<Option<String>, String>, other: u32) -> Option<()> {
            seed.unwrap().map(|_| ())
        }
    }

    export_contains! {
        @Java_io_parity_secure_native_test_contains
        fn test_contains(contained: Result<bool, String>, other: u32) -> Option<bool> {
            contained.ok()
        }
    }

    export_delete! {
        @Java_io_parity_secure_native_test_delete
        fn test_delete(success: Result<(), String>, other: u32) -> Option<()> {
            success.ok()
        }
    }

    #[test]
    fn test_exports() {
    }
}
