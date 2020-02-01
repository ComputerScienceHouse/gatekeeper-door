#![allow(non_camel_case_types)]

use std::ffi::CString;
use std::os::raw::{c_void, c_char};

pub type context_t = c_void;
pub type mifare_t = c_void;
pub type device_t = c_void;
pub type realm_t = c_void;

#[repr(transparent)]
pub struct Realm(*mut c_void);

impl Realm {
    pub fn new(
        slot: u8,
        name: &str,
        association_id: &str,
        auth: &str,
        read: &str,
        update: &str,
        public: &str,
        private: &str,
    ) -> Option<Self> {
        let ffi_name = CString::new(name).ok()?;
        let ffi_association = CString::new(association_id).ok()?;
        let ffi_auth = CString::new(auth).ok()?;
        let ffi_read = CString::new(read).ok()?;
        let ffi_update = CString::new(update).ok()?;
        let ffi_public = CString::new(public).ok()?;
        let ffi_private = CString::new(private).ok()?;

        let inner = unsafe {
            realm_create(
                slot,
                ffi_name.into_raw(),
                ffi_association.into_raw(),
                ffi_read.into_raw(),
                ffi_auth.into_raw(),
                ffi_update.into_raw(),
                ffi_public.into_raw(),
                ffi_private.into_raw(),
            )
        };
        Some(Realm(inner))
    }
}

#[link(name = "gatekeeper")]
extern {
    pub fn realm_create(
        slot: u8,
        name: *const c_char,
        association_id: *const c_char,
        auth_key: *const c_char,
        read_key: *const c_char,
        update_key: *const c_char,
        public_key: *const c_char,
        private_key: *const c_char,
    ) -> *mut c_void;

    pub fn realm_free(realm: *const c_void);

    pub fn issue_tag(
        tag: *mut mifare_t,
        system_secret: *const c_char,
        realms: *mut *mut c_void,
        num_realms: usize,
    ) -> i32;

    pub fn authenticate_tag(
        tag: *mut mifare_t,
        realm: *mut realm_t,
    ) -> i32;
}

#[link(name = "nfc")]
extern {
    pub fn nfc_init(context: *mut *mut context_t);
    pub fn nfc_list_devices(context: *mut context_t, devices: *mut *mut c_char, device_count: usize) -> u32;
    pub fn nfc_open(context: *mut context_t, device_id: *const c_char) -> *mut device_t;
    pub fn nfc_close(context: *mut context_t);
    pub fn nfc_exit(context: *mut context_t);
}

#[link(name = "freefare")]
extern {
    pub fn freefare_get_tags(device: *const device_t) -> *mut *mut mifare_t;
    pub fn freefare_get_tag_type(tag: *mut mifare_t) -> i8;
    pub fn freefare_get_tag_uid(tag: *mut mifare_t) -> *const c_char;
    pub fn freefare_get_tag_friendly_name(tag: *mut mifare_t) -> *const c_char;
    pub fn freefare_free_tags(tags: *mut *mut mifare_t);
}
