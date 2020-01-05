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
            &apdu::compute_sig(myna::test_vector::MSG1_SHA256)[..],
            &mut buf,
        )
        .unwrap();
    println!("Verifying with JPKI Auth Key {:x?}", res);
}
struct FuzzApdu {
    ins: u8,
    sw1: u8,
    sw2: u8,
}
pub fn fuzz() {
    let mynacard = MynaCard::search_card().unwrap();
    let mut buf = [0u8; 300];
    let mut avail = &mut Vec::<FuzzApdu>::new();
    for i in 0..255 {
        let res = mynacard
            .transmit(&make_apdu(0x00, i, (0, 0), &[], 0)[..], &mut buf)
            .unwrap();
        if res.sw1 == 0x6d && res.sw2 == 0x00 {
            continue;
        }
        println!("INS: 0x{:x}, SW: {:x} {:x}", i, res.sw1, res.sw2);
        avail.push(FuzzApdu {
            ins: i,
            sw1: res.sw1,
            sw2: res.sw2,
        });
    }
}
