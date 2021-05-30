#![deny(warnings)]

use std::mem::MaybeUninit;
use std::ffi::{CString, CStr};
use std::marker::PhantomData;

pub mod ffi;

pub struct Nfc {
    context: *mut ffi::context_t,
}

impl Nfc {
    pub fn new() -> Option<Self> {

        let mut context_uninit = MaybeUninit::<*mut ffi::context_t>::uninit();
        let context = unsafe {
            ffi::nfc_init(context_uninit.as_mut_ptr());
            if context_uninit.as_mut_ptr() == std::ptr::null_mut() {
                return None;
            }
            context_uninit.assume_init()
        };

        Some(Nfc { context })
    }

    pub fn gatekeeper_device(&mut self, conn_str: String) -> Option<NfcDevice> {
        let device_string = CString::new(conn_str).unwrap();
        let device = unsafe {
            let device_ptr = ffi::nfc_open(self.context, device_string.as_ptr());
            if device_ptr == std::ptr::null_mut() {
                return None;
            }
            device_ptr
        };
        Some(NfcDevice { device, _context: self })
    }
}

impl Drop for Nfc {
    fn drop(&mut self) {
        unsafe {
            ffi::nfc_exit(self.context);
        }
    }
}

pub struct NfcDevice<'a> {
    device: *mut ffi::device_t,
    _context: &'a Nfc,
}

impl NfcDevice<'_> {
    pub fn first_tag(&mut self) -> Option<NfcTag> {

        let (tags, tag) = unsafe {
            let tags = ffi::freefare_get_tags(self.device);
            if tags == std::ptr::null_mut() { return None; }

            let tag = *tags;
            if tag == std::ptr::null_mut() { return None; }
            (tags, tag)
        };

        Some(NfcTag { tags, tag, _device_lifetime: PhantomData })
    }
}

impl Drop for NfcDevice<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::nfc_close(self.device);
        }
    }
}

pub struct NfcTag <'a> {
    tags: *mut *mut ffi::mifare_t,
    tag: *mut ffi::mifare_t,
    _device_lifetime: std::marker::PhantomData<&'a ()>,
}

impl NfcTag<'_> {
    pub fn get_uid(&mut self) -> Option<String> {
        unsafe {
            let tag_uid = ffi::freefare_get_tag_uid(self.tag);
            if tag_uid == std::ptr::null_mut() { return None; }
            let tag_uid_string = CString::from_raw(tag_uid);
            Some(tag_uid_string.to_string_lossy().to_string())
        }
    }

    pub fn get_friendly_name(&mut self) -> Option<&str> {
        unsafe {
            let tag_name = ffi::freefare_get_tag_friendly_name(self.tag);
            let tag_name_string = CStr::from_ptr(tag_name);
            tag_name_string.to_str().ok()
        }
    }

    pub fn issue(&mut self, system_secret: &str, realm: &mut Realm) -> Result<(), ()> {
        let system_secret = CString::new(system_secret).unwrap();
        let realms = &mut realm.realm;
        unsafe {
            let issue_result = ffi::issue_tag(self.tag, system_secret.as_ptr(), realms as *mut _, 1);
            if issue_result != 0 { return Err(()); }
            return Ok(());
        }
    }

    // TODO: None of this is super ideal...
    pub fn authenticate(&mut self, realm: &mut Realm) -> Result<String, ()> {
        let mut association_id = [0u8; 37];
        let auth_result = unsafe {
            ffi::authenticate_tag(self.tag, realm.realm, association_id.as_mut_ptr())
        };
        if auth_result == 0 { return Err(()); }

        let mut association_id = association_id.to_vec();
        // Pop off NUL byte
        association_id.pop();

        Ok(String::from_utf8(association_id).unwrap())
    }
}

impl Drop for NfcTag<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::freefare_free_tags(self.tags);
        }
    }
}

pub struct Realm {
    realm: *mut ffi::realm_t,
}

// A realm is a global thing, it's not tied to a card.
// Keys here are secrets for that particular project (e.g. drink, gatekeeper)
// Most likely, the only thing you want to change here is 'association' for each card
impl Realm {
    pub fn new(
        slot: u8,
        name: &str,
        association: &str,
        auth_key: &str,
        read_key: &str,
        update_key: &str,
        public_key: &str,
        private_key: &str,
    ) -> Option<Self> {
        let ffi_name = CString::new(name).ok()?;
        let ffi_association = CString::new(association).ok()?;
        let ffi_auth_key = CString::new(auth_key).ok()?;
        let ffi_read_key = CString::new(read_key).ok()?;
        let ffi_update_key = CString::new(update_key).ok()?;
        let ffi_public_key = CString::new(public_key).ok()?;
        let ffi_private_key = CString::new(private_key).ok()?;

        let realm = unsafe {
            ffi::realm_create(slot,
                ffi_name.as_ptr(),
                ffi_association.as_ptr(),
                ffi_auth_key.as_ptr(),
                ffi_read_key.as_ptr(),
                ffi_update_key.as_ptr(),
                ffi_public_key.as_ptr(),
                ffi_private_key.as_ptr(),
            )
        };

        Some(Realm { realm })
    }
}

impl Drop for Realm {
    fn drop(&mut self) {
        unsafe {
            ffi::realm_free(self.realm);
        }
    }
}
