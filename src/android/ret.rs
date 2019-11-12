use crate::Return;
use jni::sys::{jboolean, jstring, JNI_FALSE};
use jni::JNIEnv;

impl<'a> Return<'a> for () {
    type Ext = jboolean;
    type Env = JNIEnv<'a>;
    fn convert(_: &Self::Env, _val: Self) -> Self::Ext {
        JNI_FALSE
    }
}

impl<'a> Return<'a> for bool {
    type Ext = jboolean;
    type Env = JNIEnv<'a>;
    fn convert(_: &Self::Env, val: Self) -> Self::Ext {
        val as jboolean
    }
}

impl<'a> Return<'a> for String {
    type Ext = jstring;
    type Env = JNIEnv<'a>;
    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        env.new_string(val)
            .expect("Could not create java string")
            .into_inner()
    }
}

impl<'a, Inner: Return<'a, Env = JNIEnv<'a>> + Default> Return<'a> for Option<Inner> {
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
                let class = env
                    .find_class("java/lang/Exception")
                    .expect("Must have the Exception class; qed");
                let exception: JThrowable<'a> = env
                    .new_object(class, "()V", &[])
                    .expect("Must be able to instantiate the Exception; qed")
                    .into();
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                // !!!!                                                        !!!!
                // !!!! WE CAN NO LONGER INTERACT WITH JNIENV AFTER THIS POINT !!!!
                // !!!!                                                        !!!!
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                env.throw(exception)
                    .expect("Must be able to throw the Exception; qed");
                ret
            }
        }
    }

    fn convert_without_exception(env: &Self::Env, val: Self) -> Self::Ext {
        match val {
            Some(inner) => Return::convert_without_exception(env, inner),
            None => Return::convert_without_exception(env, Inner::default()),
        }
    }
}

impl<'a, Inner: Return<'a, Env = JNIEnv<'a>> + Default> Return<'a> for Result<Inner, String> {
    type Ext = Inner::Ext;
    type Env = Inner::Env;

    fn convert(env: &Self::Env, val: Self) -> Self::Ext {
        match val {
            Ok(inner) => Return::convert(env, inner),
            Err(exception) => {
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                // !!!!                                                              !!!!
                // !!!! RETURN VALUE HAS TO BE CREATED BEFORE THROWING THE EXCEPTION !!!!
                // !!!!                                                              !!!!
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                let ret = Return::convert_without_exception(env, Inner::default());
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                // !!!!                                                        !!!!
                // !!!! WE CAN NO LONGER INTERACT WITH JNIENV AFTER THIS POINT !!!!
                // !!!!                                                        !!!!
                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                env.throw(exception)
                    .expect("Must be able to throw the Exception; qed");
                ret
            }
        }
    }

    fn convert_without_exception(env: &Self::Env, val: Self) -> Self::Ext {
        match val {
            Ok(inner) => Return::convert(env, inner),
            Err(_) => Return::convert_without_exception(env, Inner::default()),
        }
    }
}
