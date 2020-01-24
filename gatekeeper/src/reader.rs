use nearfield::{NFC, Initiator, Error, target, modulation};

pub struct Reader {
    nfc: NFC,
    initiator: Initiator,
}

impl Reader {
    pub fn new() -> Result<Reader, Error> {
        let mut nfc = NFC::new()?;
        let initiator = nfc.open_initiator()?;

        Ok(Reader {
            nfc,
            initiator,
        })
    }

    pub fn name(&mut self) -> &'static str {
        self.initiator.name()
    }

    pub fn poll(&mut self) -> Result<target::Target, Error> {
        self.initiator.poll(modulation::MIFARE, 20, 2)
    }
}