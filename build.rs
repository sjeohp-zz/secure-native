// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = PathBuf::from("../jni-android-sys");
//    jni_android_sys_gen::generate(out_path.as_path(), 24, 29, Some("/Users/sjeohp/Library/Android/sdk"));
}

