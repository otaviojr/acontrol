use fingerprint::{Fingerprint};
use nfc::{NfcReader};

pub struct acontrol_system<'a> {
    fingerprint_drv: Box<&'a mut Fingerprint>,
    nfc_drv: Box<&'a mut NfcReader>
}

impl<'a> acontrol_system<'a> {
}

pub fn init_acontrol_system(fingerprint_drv: &mut Fingerprint, nfc_drv: &mut NfcReader) -> bool {
  return true;
}
