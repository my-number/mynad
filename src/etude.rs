extern crate pcsc;
use hex_literal::hex;
use sha2::{Sha256, Digest};
use myna::card::{
    apdu,
    binary_reader::BinaryReader,
    make_apdu,
    apdu::{ApduRes, Apdu},
};
pub struct MynaCard {
    ctx: Option<pcsc::Context>,
    card: Option<pcsc::Card>,
}
impl MynaCard {
    pub fn search_card() -> Result<MynaCard, pcsc::Error> {
        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;

        let buflen = ctx.list_readers_len()?;
        let mut buf = vec![0u8; buflen];
        let mut readers = ctx.list_readers(&mut buf)?;
        let reader = match readers.next() {
            Some(r) => r,
            None => return Err(pcsc::Error::ReaderUnavailable),
        };
        let card = ctx.connect(reader, pcsc::ShareMode::Exclusive, pcsc::Protocols::ANY)?;

        Ok(MynaCard {
            ctx: Some(ctx),
            card: Some(card),
        })
    }
    pub fn transmit<'a>(&self, apdu: &[u8], recv_buffer: &'a mut [u8]) -> Result<&'a [u8], ()> {
        let card = self.card.as_ref().expect("Card is not None");
        match card.transmit(apdu, recv_buffer) {
            Ok(buf) => Ok(&buf),
            Err(_) => Err(()),
        }
    }
}

pub fn main() {
    let mynacard = MynaCard::search_card().unwrap();

    let mut responder = Apdu::new(|data| {
        let mut buf = [0u8; 300];
        let buf = mynacard.transmit(data, &mut buf).unwrap();
        ApduRes::from_apdu(buf)
    });
    responder.select_jpki_ap().unwrap();
    responder.select_jpki_cert_auth().unwrap();
    let cert = responder.read_binary().unwrap();
    let mut hasher = Sha256::new();
    hasher.input(cert);
    let hash = hasher.result();
    println!("{:?}", hash);
    responder.select_jpki_auth_pin().unwrap();
    responder.verify_pin("1919").unwrap();
    responder.select_jpki_auth_key().unwrap();
    let sig = responder.compute_sig(&hash);

    println!("{:?}", sig);
}
