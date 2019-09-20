use crate::Argument;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jclass, jint, jobject, jsize, jstring, JNI_FALSE};
use jni::JNIEnv;

impl<'jni> Argument<'jni> for u32 {
    type Ext = jint;
    type Env = JNIEnv<'jni>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val as u32
    }
}

impl<'jni> Argument<'jni> for u8 {
    type Ext = jint;
    type Env = JNIEnv<'jni>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val as u8
    }
}

impl<'jni> Argument<'jni> for bool {
    type Ext = jboolean;
    type Env = JNIEnv<'jni>;
    fn convert(_: &Self::Env, val: Self::Ext) -> Self {
        val != 0
    }
}

impl<'jni> Argument<'jni> for String {
    type Ext = JString<'jni>;
    type Env = JNIEnv<'jni>;
    fn convert(env: &Self::Env, val: Self::Ext) -> Self {
        env.get_string(val).expect("Invalid java string").into()
    }
}
