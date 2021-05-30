#![allow(non_camel_case_types)]

use std::os::raw::{c_void, c_char};

pub type context_t = c_void;
pub type mifare_t = c_void;
pub type device_t = c_void;
pub type realm_t = c_void;

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
    ) -> *mut realm_t;

    pub fn realm_free(realm: *mut realm_t);

    pub fn issue_tag(
        tag: *mut mifare_t,
        system_secret: *const c_char,
        realms: *mut *mut c_void,
        num_realms: usize,
    ) -> i32;

    pub fn authenticate_tag(
        tag: *mut mifare_t,
        realm: *mut realm_t,
        association_id: *mut u8,
    ) -> i32;
}

#[link(name = "nfc")]
extern {
    pub fn nfc_init(context: *mut *mut context_t);
    pub fn nfc_list_devices(context: *mut context_t, devices: *mut *const c_char, device_count: usize) -> u32;
    pub fn nfc_open(context: *mut context_t, device_id: *const c_char) -> *mut device_t;
    pub fn nfc_close(context: *mut device_t);
    pub fn nfc_exit(context: *mut context_t);
}

#[link(name = "freefare")]
extern {
    pub fn freefare_get_tags(device: *const device_t) -> *mut *mut mifare_t;
    pub fn freefare_get_tag_type(tag: *mut mifare_t) -> i8;
    pub fn freefare_get_tag_uid(tag: *mut mifare_t) -> *mut c_char;
    pub fn freefare_get_tag_friendly_name(tag: *mut mifare_t) -> *const c_char;
    pub fn freefare_free_tags(tags: *mut *mut mifare_t);
    pub fn free(data: *mut c_void);
}
