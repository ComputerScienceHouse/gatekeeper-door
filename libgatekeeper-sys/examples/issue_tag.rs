use libgatekeeper_sys::{Nfc, Realm};
use std::error::Error;

fn main() {
    match run() {
        Ok(()) => (),
        Err(e) => println!("{}", e.description()),
    }
}

fn run() -> std::result::Result<(), Box<dyn Error>> {
    let mut nfc = Nfc::new().ok_or("failed to open NFC context")?;
    let mut device = nfc.gatekeeper_device().ok_or("failed to open NFC device")?;
    let mut tag = device.first_tag().ok_or("failed to read tag")?;
    let mut realm = Realm::new(0, "Door",
        "7c5d9984-8392-4dce-8dc1-75791fa6bf31",
        "c789aef4d156b9e1a23bcbe66742b4eb",
        "53e49fedce8a1fad6be924cb51f79bfe",
        "96e874711115cde3ca530c9a15c4838a",
        "-----BEGIN PUBLIC KEY-----\nMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEUSCSsyBgHLLs9d5+p+cTGljR9aeFZ19D\ngBkuomyNPEy2rYI/0g9jeftRkkRXlZNQG/jk8PNtKuYoq4cKTYnMiZEiIcHq6fRi\nusrdYdkrS2iau+xENfzkkouvYJwarMtu\n-----END PUBLIC KEY-----\n",
        "-----BEGIN EC PRIVATE KEY-----\nMIGkAgEBBDCYfNkZFFqtgPRwxWy3SWfNvznHO0V5CNOlysmE3jXOGtO/99XpmKx4\nAsPFrMm6iragBwYFK4EEACKhZANiAARRIJKzIGAcsuz13n6n5xMaWNH1p4VnX0OA\nGS6ibI08TLatgj/SD2N5+1GSRFeVk1Ab+OTw820q5iirhwpNicyJkSIhwerp9GK6\nyt1h2StLaJq77EQ1/OSSi69gnBqsy24=\n-----END EC PRIVATE KEY-----\n",
    ).ok_or("failed to create realm")?;

    tag.issue("cdfc36ef1b3d87a81a4114cb75459e27", &mut realm)
        .map_err(|_| "failed to issue tag")?;

    println!("Successfully issued tag!");

    tag.authenticate(&mut realm)
        .map_err(|_| "failed to authenticate!")?;

    println!("Successfully authenticated tag!");

    Ok(())
}
