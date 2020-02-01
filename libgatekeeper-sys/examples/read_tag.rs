use libgatekeeper_sys::Nfc;
use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {

    let mut nfc = Nfc::new().ok_or("failed to create NFC context")?;
    let mut device = nfc.gatekeeper_device().ok_or("failed to get gatekeeper device")?;
    let mut tag = device.first_tag().ok_or("failed to get tag")?;

    println!("Tag UID: {:?}", tag.get_uid());
    println!("Tag Name: {:?}", tag.get_friendly_name());

    Ok(())
}
