use crate::Argument;
use jni::objects::JString;
use jni::sys::{jboolean, jint};
use jni::JNIEnv;

impl<'a> Argument<'a> for u32 {
    type Ext = jint;
    type Env = JNIEnv<'a>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val as u32
    }
}

impl<'a> Argument<'a> for u8 {
    type Ext = jint;
    type Env = JNIEnv<'a>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val as u8
    }
}

impl<'a> Argument<'a> for bool {
    type Ext = jboolean;
    type Env = JNIEnv<'a>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val != 0
    }
}

impl<'a> Argument<'a> for String {
    type Ext = JString<'a>;
    type Env = JNIEnv<'a>;
    fn convert(env: &Self::Env, val: Self::Ext) -> Self {
        env.get_string(val).expect("Invalid java string").into()
    }
}
