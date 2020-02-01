use std::mem::MaybeUninit;
use std::ffi::{CString, CStr};

use libgatekeeper_sys::ffi::{
    context_t,
    nfc_init,
    nfc_open,
    nfc_close,
    nfc_exit,
    freefare_get_tags,
    freefare_get_tag_uid,
    freefare_get_tag_friendly_name,
    freefare_free_tags,
    realm_create,
    realm_free,
    issue_tag,
    authenticate_tag,
};

fn main() {

    // Initialize the NFC Context
    let mut context = MaybeUninit::<*mut context_t>::uninit();
    unsafe { nfc_init(context.as_mut_ptr()); }
    if context.as_mut_ptr() == std::ptr::null_mut() {
        println!("Failed to initialize NFC context");
        return;
    }
    let context = unsafe { context.assume_init() };

    let device_string = CString::new("pn532_uart:/dev/ttyUSB0").unwrap();
    let device = unsafe { nfc_open(context, device_string.as_ptr()) };

    let tags = unsafe { freefare_get_tags(device) };
    let tag = unsafe { *tags };

    let tag_type = unsafe { freefare_get_tag_friendly_name(tag) };
    let tag_type_str = unsafe { CStr::from_ptr(tag_type) };
    println!("Tag type: {}", tag_type_str.to_str().expect("should get string"));

    let tag_uid = unsafe { freefare_get_tag_uid(tag) };
    let tag_uid_str = unsafe { CStr::from_ptr(tag_uid) };
    println!("Tag uid: {}", tag_uid_str.to_str().expect("should get uid"));

    // Create the realm
    let mut realm = unsafe {
        let name = CString::new("Doors").unwrap();
        let association = CString::new("7c5d9984-8392-4dce-8dc1-75791fa6bf31").unwrap();
        let auth = CString::new("c789aef4d156b9e1a23bcbe66742b4eb").unwrap();
        let read = CString::new("53e49fedce8a1fad6be924cb51f79bfe").unwrap();
        let update = CString::new("96e874711115cde3ca530c9a15c4838a").unwrap();
        let public = CString::new("-----BEGIN PUBLIC KEY-----\nMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEUSCSsyBgHLLs9d5+p+cTGljR9aeFZ19D\ngBkuomyNPEy2rYI/0g9jeftRkkRXlZNQG/jk8PNtKuYoq4cKTYnMiZEiIcHq6fRi\nusrdYdkrS2iau+xENfzkkouvYJwarMtu\n-----END PUBLIC KEY-----\n").unwrap();
        let private = CString::new("-----BEGIN EC PRIVATE KEY-----\nMIGkAgEBBDCYfNkZFFqtgPRwxWy3SWfNvznHO0V5CNOlysmE3jXOGtO/99XpmKx4\nAsPFrMm6iragBwYFK4EEACKhZANiAARRIJKzIGAcsuz13n6n5xMaWNH1p4VnX0OA\nGS6ibI08TLatgj/SD2N5+1GSRFeVk1Ab+OTw820q5iirhwpNicyJkSIhwerp9GK6\nyt1h2StLaJq77EQ1/OSSi69gnBqsy24=\n-----END EC PRIVATE KEY-----\n").unwrap();
        realm_create(0,
                     name.as_ptr(),
                     association.as_ptr(),
                     auth.as_ptr(),
                     read.as_ptr(),
                     update.as_ptr(),
                     public.as_ptr(),
                     private.as_ptr())
    };

    let realms = &mut realm;
    let system_secret = CString::new("cdfc36ef1b3d87a81a4114cb75459e27").unwrap();
    unsafe {
        let issue_result = issue_tag(tag, system_secret.as_ptr(), realms as *mut _, 1);
        if issue_result != 0 {
            println!("Failed to issue tag");
        } else {
            println!("Successfully issued tag");
        }

        let authenticate_result = authenticate_tag(tag, realm);
        if authenticate_result != 0 {
            println!("Tag authenticates!");
        } else {
            println!("Tag failed authentication");
        }
    }

    unsafe {
        realm_free(realm);
        freefare_free_tags(tags);
        nfc_close(device);
        nfc_exit(context);
    }

    println!("Did not crash! Woooooo");
}
