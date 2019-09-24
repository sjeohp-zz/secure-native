#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod arg;
mod ret;

pub use arg::*;
pub use ret::*;

use jni::objects::{JClass, JObject, JString};
use jni::strings::{JNIStr, JNIString};
use jni::sys::{jboolean, jclass, jobject, jsize, jstring, JNI_FALSE};
use jni::JNIEnv;
use jni_glue::{AsJValue, AsValidJObjectAndEnv, ByteArray, JniType, Local, ObjectArray};
use std::os::raw::c_char;
use std::ptr::null_mut;

use android::app::Activity;
use android::content::{Context, SharedPreferences};
use android::security::keystore::{KeyGenParameterSpec, KeyGenParameterSpec_Builder, KeyProperties};
use android::util::Base64;
use java::lang::Throwable;
use java::security::spec::AlgorithmParameterSpec;
use java::security::{AlgorithmParameters, Key, KeyStore, KeyStore_SecretKeyEntry, SecureRandom};
use javax::crypto::spec::{GCMParameterSpec, IvParameterSpec};
use javax::crypto::{Cipher, KeyGenerator, SecretKey};
use jni_android_sys::*;

use crate::{double_expect, expect};

const ANDROID_KEYSTORE_PROVIDER: &'static str = "AndroidKeyStore";

struct ResOpt<T> {
    inner: Result<Option<T>, String>,
}

impl<T> ResOpt<T> {
    fn and_then<'env, F, U>(self, f: F) -> ResOpt<U>
    where
        F: FnOnce(T) -> ResOpt<U>,
    {
        match self.inner {
            Ok(Some(y)) => f(y),
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt { inner: Err(e) },
        }
    }

    fn map_result<'env, F, U>(self, f: F) -> ResOpt<U>
    where
        F: FnOnce(T) -> Result<Option<U>, Local<'env, Throwable>>,
    {
        match self.inner {
            Ok(Some(y)) => ResOpt {
                inner: f(y).map_err(|e| format!("{:?}", e.toString().unwrap().unwrap())),
            },
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt { inner: Err(e) },
        }
    }

    fn map_option<'env, F, U>(self, f: F) -> ResOpt<U>
    where
        F: FnOnce(T) -> Option<U>,
    {
        match self.inner {
            Ok(Some(y)) => ResOpt { inner: Ok(f(y)) },
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt { inner: Err(e) },
        }
    }

    fn map_inner<'env, F, U>(self, f: F) -> ResOpt<U>
    where
        F: FnOnce(T) -> U,
    {
        match self.inner {
            Ok(Some(y)) => ResOpt { inner: Ok(Some(f(y))) },
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt { inner: Err(e) },
        }
    }
}

impl<'env, T> Into<Result<Option<T>, String>> for ResOpt<T> {
    fn into(self) -> Result<Option<T>, String> {
        self.inner
    }
}

impl<'env, T> From<Result<Option<T>, Local<'env, Throwable>>> for ResOpt<T> {
    fn from(val: Result<Option<T>, Local<'env, Throwable>>) -> Self {
        match val {
            Ok(Some(y)) => ResOpt { inner: Ok(Some(y)) },
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt {
                inner: Err(format!("{:?}", e.toString().unwrap().unwrap())),
            },
        }
    }
}

impl<'env, T> From<Result<Option<T>, String>> for ResOpt<T> {
    fn from(val: Result<Option<T>, String>) -> Self {
        match val {
            Ok(Some(y)) => ResOpt { inner: Ok(Some(y)) },
            Ok(None) => ResOpt { inner: Ok(None) },
            Err(e) => ResOpt { inner: Err(e) },
        }
    }
}

macro_rules! resopt {
    ( $x:expr ) => {
        ResOpt::from($x)
    };
}

#[macro_export]
macro_rules! double_expect {
    ( $x:expr ) => {
        $x.expect(&format!("{}:{}", file!(), line!())).expect(&format!("{}:{}", file!(), line!()))
    };
}

#[allow(non_snake_case)]
fn Local_String<'env>(env: &'env JNIEnv, s: String) -> Local<'env, java::lang::String> {
    unsafe { Local::from_env_object(env.get_native_interface(), env.new_string(s).expect("Couldn't create java string!").into_inner()) }
}

#[allow(non_snake_case)]
fn Local_ObjectArray<'env, S>(env: &'env JNIEnv, size: jsize, inner_class: S) -> Local<'env, ObjectArray<java::lang::String, Throwable>>
where
    S: Into<JNIString>,
{
    unsafe {
        let class = env.find_class(inner_class).unwrap();
        let object = env.new_object_array(size, class, JObject::null()).unwrap();
        let exception = env.exception_occurred().unwrap();
        assert!(exception.is_null()); // Only sane exception here is an OOM exception
        Local::from_env_object(env.get_native_interface(), object)
    }
}

#[allow(non_snake_case)]
fn Local_KeyGenerator<'env>(env: &'env JNIEnv, algorithm: &'env java::lang::String, provider: &'env java::lang::String) -> Local<'env, KeyGenerator> {
    double_expect!(KeyGenerator::getInstance_String_String(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        Some(algorithm),
        Some(provider)
    ))
}

#[allow(non_snake_case)]
fn Local_KeyGenerator_generateKey<'env>(env: &'env JNIEnv, keygen: &'env KeyGenerator) -> ResOpt<Local<'env, Key>> {
    resopt!(keygen.generateKey()).map_inner(|x| unsafe { std::mem::transmute::<Local<'_, SecretKey>, Local<'_, Key>>(x) })
}

#[allow(non_snake_case)]
fn Local_KeyGenerator_init<'env>(env: &'env JNIEnv, keygen: &'env KeyGenerator, spec: &'env AlgorithmParameterSpec) -> Result<(), Local<'env, Throwable>> {
    keygen.init_AlgorithmParameterSpec(Some(spec))
}

#[allow(non_snake_case)]
fn Local_AlgorithmParameterSpec1<'env>(
    env: &'env JNIEnv,
    alias: &'env java::lang::String,
    block_mode: &'env ObjectArray<java::lang::String, Throwable>,
    padding: &'env ObjectArray<java::lang::String, Throwable>,
    key_size: i32,
) -> Result<Option<Local<'env, AlgorithmParameterSpec>>, String> {
    resopt!(KeyGenParameterSpec_Builder::new(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        alias,
        KeyProperties::PURPOSE_ENCRYPT | KeyProperties::PURPOSE_DECRYPT,
    )
    .map(|x| Some(x)))
    .and_then(|x| {
        resopt!(x.setKeySize(key_size))
            .map_result(|_| x.setBlockModes(Some(&*block_mode)))
            .map_result(|_| x.setEncryptionPaddings(Some(&*padding)))
            .map_result(|_| x.setUserAuthenticationRequired(true))
            .map_result(|_| x.build())
            .map_result(|x| Ok(Some(unsafe { std::mem::transmute::<Local<'_, KeyGenParameterSpec>, Local<'_, AlgorithmParameterSpec>>(x) })))
    })
    .into()
}

#[allow(non_snake_case)]
fn Local_AlgorithmParameterSpec2<'env>(env: &'env JNIEnv, iv: &'env ByteArray) -> Local<'env, AlgorithmParameterSpec> {
    let spec = expect!(IvParameterSpec::new_byte_array(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, Some(iv)));
    unsafe { std::mem::transmute::<Local<'_, IvParameterSpec>, Local<'_, AlgorithmParameterSpec>>(spec) }
}

#[allow(non_snake_case)]
fn Local_Base64_encodeToString<'env>(env: &'env JNIEnv, bytes: &'env ByteArray) -> ResOpt<Local<'env, java::lang::String>> {
    resopt!(Base64::encodeToString_byte_array_int(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        Some(bytes),
        Base64::DEFAULT
    ))
}

#[allow(non_snake_case)]
fn Local_Base64_decode<'env>(env: &'env JNIEnv, s: &'env java::lang::String) -> Local<'env, ByteArray> {
    double_expect!(Base64::decode_String_int(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, Some(s), Base64::DEFAULT))
}

#[allow(non_snake_case)]
fn Local_Cipher<'env>(env: &'env JNIEnv, transform: &'env java::lang::String, mode: i32, secret_key: &'env Key, spec: Option<&'env AlgorithmParameterSpec>) -> ResOpt<Local<'env, Cipher>> {
    resopt!(Cipher::getInstance_String(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, transform)).map_inner(|x| {
        x.init_int_Key_AlgorithmParameterSpec(mode, Some(secret_key), spec);
        x
    })
}

#[allow(non_snake_case)]
fn Local_Cipher_doFinal<'env>(cipher: &'env Cipher, bytes: &'env ByteArray) -> Local<'env, ByteArray> {
    double_expect!(cipher.doFinal_byte_array(Some(bytes)))
}

#[allow(non_snake_case)]
fn Local_Context<'env>(env: &'env JNIEnv, activity: &'env JObject) -> Local<'env, Context> {
    unsafe { Local::from_env_object(env.get_native_interface(), activity.into_inner()) }
}

#[allow(non_snake_case)]
fn Local_Context_getSharedPreferences<'env>(context: &'env Context, name: &'env java::lang::String) -> Local<'env, SharedPreferences> {
    double_expect!(context.getSharedPreferences(Some(name), Context::MODE_PRIVATE))
}

#[allow(non_snake_case)]
fn Local_KeyStore<'env>(env: &'env JNIEnv, provider: &'env java::lang::String) -> Local<'env, KeyStore> {
    let ks = double_expect!(KeyStore::getInstance_String(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, provider));
    ks.load_LoadStoreParameter(None);
    ks
}

#[allow(non_snake_case)]
fn Local_KeyStore_getKey<'env>(keystore: &'env KeyStore, alias: &'env java::lang::String) -> Local<'env, Key> {
    double_expect!(keystore.getKey(Some(alias), None))
}

pub fn put(env: &JNIEnv, activity: JObject, service: JString, account: JString, value: JString) -> Result<Option<()>, String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();
    let value: String = env.get_string(value).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let key_size = 128;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(env, service);
    let local_iv_key = Local_String(env, format!("{}iv", &account));
    let local_key = Local_String(env, account);
    let local_value = Local_String(env, value);
    let local_alias = Local_String(env, alias.to_string());
    let local_algorithm = Local_String(env, algorithm.to_string());
    let local_provider = Local_String(env, provider.to_string());
    let local_block_mode = Local_String(env, block_mode.to_string());
    let local_padding = Local_String(env, padding.to_string());
    let local_transform = Local_String(env, transform.to_string());

    let block_modes = Local_ObjectArray(env, 1, "java/lang/String");
    let paddings = Local_ObjectArray(env, 1, "java/lang/String");
    expect!(block_modes.set(0, Some(&*local_block_mode)));
    expect!(paddings.set(0, Some(&*local_padding)));

    let keygen = Local_KeyGenerator(env, &local_algorithm, &local_provider);
    return match Local_AlgorithmParameterSpec1(env, &local_alias, &block_modes, &paddings, key_size) {
        Err(e) => Err(e),
        Ok(None) => Ok(None),
        Ok(Some(spec)) => match Local_KeyGenerator_init(env, &keygen, &spec) {
            Err(e) => Err(format!("{:?}", e.toString().unwrap().unwrap())),
            Ok(_) => {
                Local_KeyGenerator_generateKey(env, &keygen)
                    .and_then(|secret_key| {
                        Local_Cipher(env, &local_transform, Cipher::ENCRYPT_MODE, &secret_key, None).and_then(|cipher| {
                            resopt!(cipher.getIV()).and_then(|iv_bytes| {
                                Local_Base64_encodeToString(env, &iv_bytes).and_then(|local_iv| {
                                    resopt!(local_value.getBytes()).and_then(|value_bytes| {
                                        resopt!(cipher.doFinal_byte_array(Some(&*value_bytes))).and_then(|encrypted_bytes| {
                                            Local_Base64_encodeToString(env, &encrypted_bytes).and_then(|encrypted_value| {
                                                let context = Local_Context(env, &activity);
                                                resopt!(context.getSharedPreferences(Some(&*local_app), Context::MODE_PRIVATE)).and_then(|pref| {
                                                    resopt!(pref.edit()).and_then(|edit| {
                                                        resopt!(edit.putString(Some(&*local_key), Some(&*encrypted_value)))
                                                            .map_result(|_| edit.putString(Some(&*local_iv_key), Some(&*local_iv)))
                                                            .and_then(|_| {
                                                                resopt!(match edit.commit() {
                                                                    Ok(true) => Ok(Some(())),
                                                                    Ok(false) => Err(format!("Unknown Android error - failed writing changes to disk.")),
                                                                    Err(e) => Err(format!("{:?}", e.toString().unwrap().unwrap())),
                                                                })
                                                            })
                                                    })
                                                })
                                            })
                                        })
                                    })
                                })
                            })
                        })
                    })
                    .inner
            }
        },
    };
}

pub fn get(env: &JNIEnv, activity: JObject, service: JString, account: JString) -> Result<Option<String>, String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(env, service);
    let local_iv_key = Local_String(env, format!("{}iv", &account));
    let local_key = Local_String(env, account);
    let local_alias = Local_String(env, alias.to_string());
    let local_algorithm = Local_String(env, algorithm.to_string());
    let local_provider = Local_String(env, provider.to_string());
    let local_transform = Local_String(env, transform.to_string());

    let context = Local_Context(env, &activity);
    let pref = Local_Context_getSharedPreferences(&context, &local_app);
    let keystore = Local_KeyStore(env, &local_provider);
    let secret_key = Local_KeyStore_getKey(&keystore, &local_alias);

    let encrypted_str = double_expect!(pref.getString(Some(&*local_key), None));
    let iv_str = double_expect!(pref.getString(Some(&*local_iv_key), None));

    let encrypted_bytes = Local_Base64_decode(env, &encrypted_str);
    let iv_bytes = Local_Base64_decode(env, &iv_str);
    let spec = Local_AlgorithmParameterSpec2(env, &iv_bytes);

    return Local_Cipher(env, &local_transform, Cipher::DECRYPT_MODE, &secret_key, Some(&spec))
        .map_result(|cipher| {
            let decrypted_bytes = Local_Cipher_doFinal(&cipher, &encrypted_bytes);
            let decrypted_str = expect!(java::lang::String::new_byte_array(
                unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
                Some(&*decrypted_bytes)
            ));
            Ok(Some(format!("{:?}", decrypted_str)))
        })
        .inner;
    //    env.new_string(format!("{:?}", decrypted_str)).expect("Couldn't get java string!").into_inner()
}

pub fn contains(env: &JNIEnv, activity: JObject, service: JString, account: JString) -> Result<Option<bool>, String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(env, service);
    let local_iv_key = Local_String(env, format!("{}iv", &account));
    let local_key = Local_String(env, account);
    let local_alias = Local_String(env, alias.to_string());
    let local_algorithm = Local_String(env, algorithm.to_string());
    let local_provider = Local_String(env, provider.to_string());
    let local_transform = Local_String(env, transform.to_string());

    let context = Local_Context(env, &activity);
    let pref = Local_Context_getSharedPreferences(&context, &local_app);
    pref.contains(Some(&*local_key)).map(|x| Some(x)).or(Err(format!("checking account contained in SharedPreferences")))
}

pub fn delete(env: &JNIEnv, activity: JObject, service: JString, account: JString) -> Result<Option<()>, String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(env, service);
    let local_iv_key = Local_String(env, format!("{}iv", &account));
    let local_key = Local_String(env, account);
    let local_alias = Local_String(env, alias.to_string());
    let local_algorithm = Local_String(env, algorithm.to_string());
    let local_provider = Local_String(env, provider.to_string());
    let local_transform = Local_String(env, transform.to_string());

    let context = Local_Context(env, &activity);
    let pref = Local_Context_getSharedPreferences(&context, &local_app);

    let edit = double_expect!(pref.edit());
    double_expect!(edit.remove(Some(&*local_key)));
    double_expect!(edit.remove(Some(&*local_iv_key)));
    match expect!(edit.commit()).into() {
        true => Ok(Some(())),
        false => Err(format!("Unknown Android error - failed writing changes to disk.")),
    }
}
