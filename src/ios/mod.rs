mod arg;
mod error;
mod ffi;
mod ret;

pub use arg::*;
use error::*;
use ffi::*;
pub use ret::*;

use core_foundation::{
    base::{kCFAllocatorDefault, CFOptionFlags, CFTypeID, CFTypeRef, FromVoid, TCFType, ToVoid},
    boolean::*,
    data::*,
    declare_TCFType,
    dictionary::*,
    error::CFErrorRef,
    impl_TCFType,
    string::*,
};
use std::{ffi::*, os::raw::c_char, os::unix::ffi::OsStrExt, path::Path, ptr};

declare_TCFType!(SecAccessControl, SecAccessControlRef);
impl_TCFType!(SecAccessControl, SecAccessControlRef, SecAccessControlGetTypeID);

pub fn put(service: &str, account: &str, value: &str) -> Result<(), String> {
    let mut error: CFErrorRef = ptr::null_mut();
    let access = unsafe {
        SecAccessControlCreateWithFlags(
            kCFAllocatorDefault,
            CFString::wrap_under_get_rule(kSecAttrAccessibleWhenUnlockedThisDeviceOnly.into()).as_CFTypeRef(),
            kSecAccessControlUserPresence,
            &mut error,
        )
    };
    if !error.is_null() {
        Err(format!("{}", Error::from(error)))
    } else {
        let attrs = unsafe {
            CFDictionary::from_CFType_pairs(&[
                (
                    CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                    CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
                ),
                (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service).as_CFType()),
                (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account).as_CFType()),
                (CFString::wrap_under_get_rule(kSecValueData.into()).as_CFType(), CFData::from_buffer(value.as_bytes()).as_CFType()),
                (
                    CFString::wrap_under_get_rule(kSecAttrAccessControl.into()).as_CFType(),
                    SecAccessControl::wrap_under_get_rule(access.into()).as_CFType(),
                ),
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
}

pub fn get(service: &str, account: &str) -> Result<String, String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account).as_CFType()),
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
        unsafe {
            (&*CFDictionary::from_void(result) as &CFDictionary).find(kSecValueData.to_void()).map(|item| {
                let data: &CFData = &*CFData::from_void(*item);
                let length = CFDataGetLength(data.as_concrete_TypeRef());
                let bytes = CFDataGetBytePtr(data.as_concrete_TypeRef());
                CFString::wrap_under_get_rule(CFStringCreateWithBytes(kCFAllocatorDefault, bytes, length, kCFStringEncodingUTF8, 0)).to_string()
            })
        }.ok_or(format!("Couldn't find value for key: {}", account))
    }
}

pub fn contains(service: &str, account: &str) -> Result<bool, String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account).as_CFType()),
            (
                CFString::wrap_under_get_rule(kSecUseAuthenticationUI.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecUseAuthenticationUIFail.into()).as_CFType(),
            ),
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

pub fn delete(service: &str, account: &str) -> Result<(), String> {
    let query = unsafe {
        CFDictionary::from_CFType_pairs(&[
            (
                CFString::wrap_under_get_rule(kSecClass.into()).as_CFType(),
                CFString::wrap_under_get_rule(kSecClassGenericPassword.into()).as_CFType(),
            ),
            (CFString::wrap_under_get_rule(kSecAttrService.into()).as_CFType(), CFString::from(service).as_CFType()),
            (CFString::wrap_under_get_rule(kSecAttrAccount.into()).as_CFType(), CFString::from(account).as_CFType()),
        ])
    };
    let status = unsafe { SecItemDelete(query.as_concrete_TypeRef()) };
    if let Some(e) = Error::maybe_from_OSStatus(status) {
        Err(format!("{}", e))
    } else {
        Ok(())
    }
}
