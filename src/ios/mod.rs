mod arg;
mod error;
mod ffi;
mod ret;

pub use arg::*;
use error::*;
use ffi::*;
pub use ret::*;

use core_foundation::{
    base::{kCFAllocatorDefault, CFTypeRef, FromVoid, TCFType, ToVoid},
    boolean::*,
    data::*,
    dictionary::*,
    string::*,
};
use std::{ffi::CString, os::raw::c_char, os::unix::ffi::OsStrExt, path::Path, ptr};

pub fn put(service: String, account: String, seed: String) -> Result<(), String> {
    let attrs = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecValueData.into()).as_CFType(), CFData::from_buffer(seed.as_bytes()).as_CFType()),
        ])
    };
    let mut result: CFTypeRef = ptr::null_mut();
    let status = unsafe { SecItemAdd(attrs.as_concrete_TypeRef(), &mut result) };
    if let Some(e) = Error::maybe_from_OSStatus(status) {
        Err(format!("{}", e))
    } else {
        Ok(())
    }
}

pub fn get(service: String, account: String) -> Result<Option<String>, String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account.as_ref()).as_CFType()),
            (
                CFString::wrap_under_get_rule(kSecMatchLimit.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecMatchLimitOne.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecReturnAttributes.into()).as_CFType(), CFBoolean::from(true).as_CFType()),
            (CFString::wrap_under_get_rule(kSecReturnData.into()).as_CFType(), CFBoolean::from(true).as_CFType()),
        ])
    };
    let mut result: CFTypeRef = ptr::null_mut();
    let status = unsafe { SecItemCopyMatching(query.as_concrete_TypeRef(), &mut result) };
    if let Some(e) = Error::maybe_from_OSStatus(status) {
        Err(format!("{}", e))
    } else {
        Ok(unsafe {
            (&*CFDictionary::from_void(result) as &CFDictionary).find(kSecValueData.to_void()).map(|item| {
                let data: &CFData = &*CFData::from_void(*item);
                let length = CFDataGetLength(data.as_concrete_TypeRef());
                let bytes = CFDataGetBytePtr(data.as_concrete_TypeRef());
                CFString::wrap_under_get_rule(CFStringCreateWithBytes(kCFAllocatorDefault, bytes, length, kCFStringEncodingUTF8, 0)).to_string()
            })
        })
    }
}

pub fn contains(service: String, account: String) -> Result<bool, String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecUseNoAuthenticationUI.into()).as_CFType(), CFBoolean::from(true).as_CFType()),
        ])
    };
    let mut result: CFTypeRef = ptr::null_mut();
    let status = unsafe { SecItemCopyMatching(query.as_concrete_TypeRef(), &mut result) };
    if status == errSecInteractionNotAllowed {
        Ok(true)
    } else if status == errSecItemNotFound {
        Ok(false)
    } else {
        Error::maybe_from_OSStatus(status).map_or(Ok(true), |e| Err(format!("{}", e)))
    }
}

pub fn delete(service: String, account: String) -> Result<(), String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service.as_ref()).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account.as_ref()).as_CFType()),
        ])
    };
    let mut result: CFTypeRef = ptr::null_mut();
    let status = unsafe { SecItemAdd(query.as_concrete_TypeRef(), &mut result) };
    if let Some(e) = Error::maybe_from_OSStatus(status) {
        Err(format!("{}", e))
    } else {
        Ok(())
    }
}
