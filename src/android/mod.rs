#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

trace_macros!(false);

mod arg;
mod ret;
#[macro_use]
mod result;
mod util;

pub use arg::*;
use result::*;
pub use ret::*;
use util::*;

use jni::objects::{JObject, JString};
use jni::sys::jboolean;
use jni::JNIEnv;

use android::content::Context;
use android::security::keystore::KeyProperties;
use javax::crypto::Cipher;
use jni_android_sys::*;

const ANDROID_KEYSTORE_PROVIDER: &'static str = "AndroidKeyStore";

pub fn put<'a>(
    env: &'a JNIEnv,
    activity: JObject,
    service: JString,
    account: JString,
    value: JString,
    with_biometry: jboolean,
) -> Result<()> {
    let service: String = env
        .get_string(service)
        .expect(&format!("Getting java string from '{:?}'", *service))
        .into();
    let account: String = env
        .get_string(account)
        .expect(&format!("Getting java string '{:?}'", *account))
        .into();
    let value: String = env
        .get_string(value)
        .expect(&format!("Getting java string '{:?}'", *value))
        .into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let key_size = 128;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let app = java_string(env, &service);
    let iv_key = java_string(env, &format!("{}iv", &account));
    let key = java_string(env, &account);
    let value = java_string(env, &value);
    let alias = java_string(env, &alias);
    let algorithm = java_string(env, &algorithm);
    let provider = java_string(env, &provider);
    let block_mode = java_string(env, &block_mode);
    let padding = java_string(env, &padding);
    let transform = java_string(env, &transform);

    let block_modes = java_string_array(env, 1)?;
    let _ = stringify_throwable!(block_modes.set(0, Some(&*block_mode)))?;
    let paddings = java_string_array(env, 1)?;
    let _ = stringify_throwable!(paddings.set(0, Some(&*padding)))?;

    let keygen = java_key_generator(env, &algorithm, &provider)?;
    let spec = java_algorithm_parameter_spec(
        env,
        &alias,
        &block_modes,
        &paddings,
        key_size,
        with_biometry != 0,
    )?;
    let _ = stringify_throwable!(keygen.init_AlgorithmParameterSpec(Some(&*spec)))?;
    let secret_key = java_generate_key(&keygen)?;

    let cipher = java_cipher(env, &transform, Cipher::ENCRYPT_MODE, secret_key, None)?;
    let iv_bytes = r#try!(resopt!(cipher.getIV()));
    let iv = java_base64_encode(env, &iv_bytes)?;
    let value_bytes = r#try!(resopt!(value.getBytes()));
    let encrypted_bytes = r#try!(resopt!(cipher.doFinal_byte_array(Some(&*value_bytes))));
    let encrypted_value = java_base64_encode(env, &encrypted_bytes)?;
    let context = java_context(env, &activity);
    let pref = r#try!(resopt!(
        context.getSharedPreferences(Some(&*app), Context::MODE_PRIVATE)
    ));
    let edit = r#try!(resopt!(pref.edit()));
    let _ = r#try!(resopt!(edit.putString(Some(&*key), Some(&*encrypted_value))));
    let _ = r#try!(resopt!(edit.putString(Some(&*iv_key), Some(&*iv))));

    return match edit.commit() {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!(
            "Unknown Android error - failed committing changes to disk."
        )),
        Err(e) => Err(format!("{:?}", e.toString().unwrap().unwrap())),
    };
}

pub fn get<'a>(
    env: &'a JNIEnv,
    activity: JObject,
    service: JString,
    account: JString,
) -> Result<String> {
    let service: String = env
        .get_string(service)
        .expect(&format!("Getting java string from '{:?}'", *service))
        .into();
    let account: String = env
        .get_string(account)
        .expect(&format!("Getting java string '{:?}'", *account))
        .into();

    let alias = format!("{}{}", service, &account);
    let algorithm = KeyProperties::KEY_ALGORITHM_AES;
    let provider = ANDROID_KEYSTORE_PROVIDER;
    let block_mode = KeyProperties::BLOCK_MODE_CBC;
    let padding = KeyProperties::ENCRYPTION_PADDING_PKCS7;
    let transform = format!("{}/{}/{}", algorithm, block_mode, padding);

    let app = java_string(env, &service);
    let iv_key = java_string(env, &format!("{}iv", &account));
    let key = java_string(env, &account);
    let alias = java_string(env, &alias);
    let provider = java_string(env, &provider);
    let transform = java_string(env, &transform);

    let context = java_context(env, &activity);
    let pref = r#try!(resopt!(
        context.getSharedPreferences(Some(&*app), Context::MODE_PRIVATE)
    ));
    let keystore = java_keystore(env, &provider)?;
    let _ = keystore.load_LoadStoreParameter(None);
    let secret_key = r#try!(resopt!(keystore.getKey(Some(&*alias), None)));

    let encrypted_str = r#try!(resopt!(pref.getString(Some(&*key), None)));
    let iv_str = r#try!(resopt!(pref.getString(Some(&*iv_key), None)));
    let encrypted_bytes = java_base64_decode(env, &encrypted_str)?;
    let iv_bytes = java_base64_decode(env, &iv_str)?;
    let spec = java_algorithm_parameter_spec_from_bytes(env, &iv_bytes)?;

    return java_cipher(
        env,
        &transform,
        Cipher::DECRYPT_MODE,
        secret_key,
        Some(&spec),
    )
    .and_then(|cipher| {
        let decrypted_bytes = r#try!(resopt!(cipher.doFinal_byte_array(Some(&*encrypted_bytes))));
        let decrypted_str = stringify_throwable!(JavaString::new_byte_array(
            unsafe { jni_glue::Env::from_ptr(env.get_native_interface()) },
            Some(&*decrypted_bytes)
        ))?;
        Ok(format!("{:?}", decrypted_str))
    });
}

pub fn contains<'a>(
    env: &'a JNIEnv,
    activity: JObject,
    service: JString,
    account: JString,
) -> Result<bool> {
    let service: String = env
        .get_string(service)
        .expect(&format!("Getting java string from '{:?}'", *service))
        .into();
    let account: String = env
        .get_string(account)
        .expect(&format!("Getting java string '{:?}'", *account))
        .into();

    let app = java_string(env, &service);
    let key = java_string(env, &account);

    let context = java_context(env, &activity);
    let pref = r#try!(resopt!(
        context.getSharedPreferences(Some(&*app), Context::MODE_PRIVATE)
    ));
    stringify_throwable!(pref.contains(Some(&*key)))
}

pub fn delete<'a>(
    env: &'a JNIEnv,
    activity: JObject,
    service: JString,
    account: JString,
) -> Result<()> {
    let service: String = env
        .get_string(service)
        .expect(&format!("Getting java string from '{:?}'", *service))
        .into();
    let account: String = env
        .get_string(account)
        .expect(&format!("Getting java string '{:?}'", *account))
        .into();

    let app = java_string(env, &service);
    let iv_key = java_string(env, &format!("{}iv", &account));
    let key = java_string(env, &account);

    let context = java_context(env, &activity);
    let pref = r#try!(resopt!(
        context.getSharedPreferences(Some(&*app), Context::MODE_PRIVATE)
    ));

    let edit = r#try!(resopt!(pref.edit()));
    let _ = r#try!(resopt!(edit.remove(Some(&*key))));
    let _ = r#try!(resopt!(edit.remove(Some(&*iv_key))));
    return match edit.commit() {
        Ok(true) => Ok(()),
        Ok(false) => Err(format!(
            "Unknown Android error - failed committing changes to disk."
        )),
        Err(e) => Err(format!("{:?}", e.toString().unwrap().unwrap())),
    };
}
