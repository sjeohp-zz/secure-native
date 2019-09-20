use crate::Return;
use jni::objects::{JClass, JObject, JString};
use jni::sys::{jboolean, jclass, jobject, jsize, jstring, JNI_FALSE};
use jni::JNIEnv;

impl<'jni> Return<'jni> for () {
    type Ext = jboolean;
    type Env = JNIEnv<'jni>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        JNI_FALSE
    }
}

impl<'jni> Return<'jni> for bool {
    type Ext = jboolean;
    type Env = JNIEnv<'jni>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        val as jboolean
    }
}

impl<'jni> Return<'jni> for String {
    type Ext = jstring;
    type Env = JNIEnv<'jni>;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        env.new_string(val).expect("Could not create java string").into_inner()
    }
}

impl<'jni, Inner: Return<'jni, Env = JNIEnv<'jni>> + Default> Return<'jni> for Option<Inner> {
    type Ext = Inner::Ext;
    type Env = Inner::Env;

    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        use jni::objects::JThrowable;

        match val {
            Some(inner) => Return::convert(env, inner),
            None => {
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                // !!!!                                                              !!!!
                // !!!! RETURN VALUE HAS TO BE CREATED BEFORE THROWING THE EXCEPTION !!!!
                // !!!!                                                              !!!!
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                let ret = Return::convert(env, Inner::default());

                let class = env.find_class("java/lang/Exception").expect("Must have the Exception class; qed");
                let exception: JThrowable<'jni> = env.new_object(class, "()V", &[]).expect("Must be able to instantiate the Exception; qed").into();

                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                // !!!!                                                        !!!!
                // !!!! WE CAN NO LONGER INTERACT WITH JNIENV AFTER THIS POINT !!!!
                // !!!!                                                        !!!!
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                env.throw(exception).expect("Must be able to throw the Exception; qed");

                ret
            }
        }
    }
}
