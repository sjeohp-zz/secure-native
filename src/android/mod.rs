#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod arg;
mod ret;

pub use arg::*;
pub use ret::*;

use jni::objects::{JClass, JObject, JString};
use jni::strings::{JNIString, JNIStr};
use jni::sys::{jboolean, jclass, jobject, jsize, jstring, JNI_FALSE};
use jni::JNIEnv;
use jni_glue::{AsValidJObjectAndEnv, ByteArray, JniType, Local, ObjectArray, AsJValue};
use std::os::raw::c_char;
use std::ptr::null_mut;

use jni_android_sys::*;
use android::app::Activity;
use android::content::{Context, SharedPreferences};
use android::security::keystore::{KeyGenParameterSpec, KeyGenParameterSpec_Builder, KeyProperties};
use android::util::Base64;
use java::lang::Throwable;
use java::security::spec::AlgorithmParameterSpec;
use java::security::{AlgorithmParameters, Key, KeyStore, KeyStore_SecretKeyEntry, SecureRandom};
use javax::crypto::spec::{GCMParameterSpec, IvParameterSpec};
use javax::crypto::{Cipher, KeyGenerator, SecretKey};

use crate::{expect, double_expect};

const ANDROID_KEYSTORE_PROVIDER: &'static str = "AndroidKeyStore";

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
fn Local_KeyGenerator_generateKey<'env>(env: &'env JNIEnv, keygen: &'env KeyGenerator) -> Local<'env, Key> {
    unsafe { std::mem::transmute::<Local<'_, SecretKey>, Local<'_, Key>>(double_expect!(keygen.generateKey())) }
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
) -> Local<'env, AlgorithmParameterSpec> {
    let mut builder = expect!(KeyGenParameterSpec_Builder::new(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        alias,
        KeyProperties::PURPOSE_ENCRYPT | KeyProperties::PURPOSE_DECRYPT,
    ));
    double_expect!(builder.setKeySize(key_size));
    double_expect!(builder.setBlockModes(Some(&*block_mode)));
    double_expect!(builder.setEncryptionPaddings(Some(&*padding)));
    unsafe { std::mem::transmute::<Local<'_, KeyGenParameterSpec>, Local<'_, AlgorithmParameterSpec>>(double_expect!(builder.build())) }
}

#[allow(non_snake_case)]
fn Local_AlgorithmParameterSpec2<'env>(
    env: &'env JNIEnv,
    iv: &'env ByteArray,
) -> Local<'env, AlgorithmParameterSpec> {
    let spec = expect!(IvParameterSpec::new_byte_array(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        Some(iv)));
    unsafe { std::mem::transmute::<Local<'_, IvParameterSpec>, Local<'_, AlgorithmParameterSpec>>(spec) }
}

#[allow(non_snake_case)]
fn Local_Base64_encodeToString<'env>(env: &'env JNIEnv, bytes: &'env ByteArray) -> Local<'env, java::lang::String> {
    double_expect!(Base64::encodeToString_byte_array_int(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        Some(bytes),
        Base64::DEFAULT
    ))
}

#[allow(non_snake_case)]
fn Local_Base64_decode<'env>(env: &'env JNIEnv, s: &'env java::lang::String) -> Local<'env, ByteArray> {
    double_expect!(Base64::decode_String_int(
        unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
        Some(s),
        Base64::DEFAULT
    ))
}

#[allow(non_snake_case)]
fn Local_Cipher<'env>(env: &'env JNIEnv, transform: &'env java::lang::String, mode: i32 , secret_key: &'env Key, spec: Option<&'env AlgorithmParameterSpec>) -> Local<'env, Cipher> {
    let cipher = double_expect!(Cipher::getInstance_String(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, transform));
    cipher.init_int_Key_AlgorithmParameterSpec(mode, Some(secret_key), spec);
    cipher
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

pub fn put(
    env: &JNIEnv,
    activity: JObject,
    service: JString,
    account: JString,
    value: JString,
) -> Result<(), String> {
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
    let spec = Local_AlgorithmParameterSpec1(env, &local_alias, &block_modes, &paddings, key_size);
    if Local_KeyGenerator_init(env, &keygen, &spec).is_ok() {
        let secret_key = Local_KeyGenerator_generateKey(env, &keygen);
        let cipher = Local_Cipher(env, &local_transform, Cipher::ENCRYPT_MODE, &secret_key, None);
        let iv_bytes = double_expect!(cipher.getIV());
        let local_iv = Local_Base64_encodeToString(env, &iv_bytes);
        let value_bytes = double_expect!(local_value.getBytes());
        let encrypted_bytes = Local_Cipher_doFinal(&cipher, &value_bytes);
        let encrypted_value = Local_Base64_encodeToString(env, &encrypted_bytes);
        let context = Local_Context(env, &activity);
        let pref = Local_Context_getSharedPreferences(&context, &local_app);
        let edit = double_expect!(pref.edit());
        double_expect!(edit.putString(Some(&*local_key), Some(&*encrypted_value)));
        double_expect!(edit.putString(Some(&*local_iv_key), Some(&*local_iv)));
        match expect!(edit.commit()).into() {
            true => Ok(()),
            false => Err(format!("committing changes to SharedPreferences")),
        }
    } else {
        Err(format!("initializing KeyGenerator"))
    }
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

    let cipher = Local_Cipher(env, &local_transform, Cipher::DECRYPT_MODE, &secret_key, Some(&spec));
    let decrypted_bytes = Local_Cipher_doFinal(&cipher, &encrypted_bytes);
    let decrypted_str = expect!(java::lang::String::new_byte_array(unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) }, Some(&*decrypted_bytes)));
    Ok(Some(format!("{:?}", decrypted_str)))
//    env.new_string(format!("{:?}", decrypted_str)).expect("Couldn't get java string!").into_inner()
}

pub fn contains(env: &JNIEnv, activity: JObject, service: JString, account: JString) -> Result<bool, String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(&env, service);
    let local_iv_key = Local_String(&env, format!("{}iv", &account));
    let local_key = Local_String(&env, account);
    let local_alias = Local_String(&env, alias.to_string());
    let local_algorithm = Local_String(&env, algorithm.to_string());
    let local_provider = Local_String(&env, provider.to_string());
    let local_transform = Local_String(&env, transform.to_string());

    let context = Local_Context(&env, &activity);
    let pref = Local_Context_getSharedPreferences(&context, &local_app);
    pref.contains(Some(&*local_key)).map_err(|_| format!("checking account contained in SharedPreferences"))
}

pub fn delete(env: &JNIEnv, activity: JObject, service: JString, account: JString) -> Result<(), String> {
    let service: String = env.get_string(service).expect("Couldn't get java string!").into();
    let account: String = env.get_string(account).expect("Couldn't get java string!").into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let local_app = Local_String(&env, service);
    let local_iv_key = Local_String(&env, format!("{}iv", &account));
    let local_key = Local_String(&env, account);
    let local_alias = Local_String(&env, alias.to_string());
    let local_algorithm = Local_String(&env, algorithm.to_string());
    let local_provider = Local_String(&env, provider.to_string());
    let local_transform = Local_String(&env, transform.to_string());

    let context = Local_Context(&env, &activity);
    let pref = Local_Context_getSharedPreferences(&context, &local_app);

    let edit = double_expect!(pref.edit());
    double_expect!(edit.remove(Some(&*local_key)));
    double_expect!(edit.remove(Some(&*local_iv_key)));
    match expect!(edit.commit()).into() {
        true => Ok(()),
        false => Err(format!("committing changes to SharedPreferences")),
    }
}

