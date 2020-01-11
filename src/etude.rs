extern crate pcsc;
use myna::card::{apdu, binary_reader::BinaryReader, make_apdu};
pub struct MynaCard {
    ctx: Option<pcsc::Context>,
    card: Option<pcsc::Card>,
}
#[derive(Debug)]
pub struct ApduRes<'a> {
    data: &'a [u8],
    sw1: u8,
    sw2: u8,
}
impl MynaCard {
    pub fn search_card() -> Result<MynaCard, pcsc::Error> {
        let ctx = pcsc::Context::establish(pcsc::Scope::User)?;

        let buflen = ctx.list_readers_len()?;
        let mut buf = vec![0u8; buflen];
        let mut readers = ctx.list_readers(&mut buf)?;
        readers.next();
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
    pub fn transmit<'a>(&self, apdu: &[u8], recv_buffer: &'a mut [u8]) -> Result<ApduRes<'a>, ()> {
        let card = self.card.as_ref().expect("Card is not None");
        match card.transmit(apdu, recv_buffer) {
            Ok(buf) => {
                let len = buf.len();
                Ok(ApduRes {
                    data: &buf[0..len - 2],
                    sw1: buf[len - 2],
                    sw2: buf[len - 1],
                })
            }
            Err(_) => Err(()),
        }
    }
}
pub fn main() {
    let mynacard = MynaCard::search_card().unwrap();
    let mut buf = [0u8; 300];
    let res = mynacard
        .transmit(&apdu::select_jpki_ap()[..], &mut buf)
        .unwrap();
    println!("Selecting JPKI AP {:x?}", res);

    let res = mynacard
        .transmit(&apdu::select_jpki_auth_pin()[..], &mut buf)
        .unwrap();
    println!("Selecting JPKI Auth PIN {:x?}", res);

    let res = mynacard
        .transmit(&apdu::verify("1919")[..], &mut buf)
        .unwrap();
    println!("Verifying PIN {:x?}", res);

    let res = mynacard
        .transmit(&apdu::select_jpki_auth_key()[..], &mut buf)
        .unwrap();
    println!("Selecting JPKI Auth Key {:x?}", res);

    let res = mynacard
        .transmit(
            &apdu::compute_sig(myna::test_vector::PKCS1_ENCODED)[..],
            &mut buf,
        )
        .unwrap();
    println!("Signing with JPKI Auth Key {:?}", res);
}
