extern crate pcsc;
use hex_literal::hex;

use myna::card::{
    apdu,
    binary_reader::BinaryReader,
    make_apdu,
    responder::{ApduRes, Responder},
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
    let k = myna::crypto::convert_pubkey_der(&hex!("3082010a0282010100c4accad3c0e232d15ecb1d8081b6dcda02a85eb5fa2825ff05d183121952019d90f3c4acf8b68343598e2637ddf9528f6c68f4ec4edb0845e7796b31407a41375614ca4ab7e1037f6e1acb090541227eac5acb7598f1788020f1398c1e41f0673a29e77d1af9b3c00ed354dc60e4cf6ae48d3e831b49da872a316388ec0f2d7cbd94a703aac9560a4595578ca22404f21b62b34b759ac0922dc5b020b297bad1e3b34eb77b0b656d68cf67f125466e6c3ee8017cca9998687d4f73dca406e9d81368dccea673ca867c5b4f05ccebccd05b9d87f28922affea0276de002bf9f47c67e20cb8a9028fb75ffbb2eff9a6ffdd922771bd35f902f95c4807b3b44bac70203010001")).unwrap();
    let res = myna::crypto::verify(k, &hex!("032cd856106f33b0d19022e72da6cc070586809d0ac8dec2ec2720044219923e"), &hex!("2addf5bce542900c6f93ab3ccfce694bc20fbf94d6096342c217cff14658047f4c1e40db2368267842081093b80a8a1cb9d0925efe110240a7115fb9831ecbb5f70e1fa38bb97842ad68204f411a938ac7fb316bb86dd0e32ea248d780bf8bf4e130dbf156a336ede2c0a1a52f4c46f25c59843973c19e910a11a72b802a55fe4a98d202003f287ab62f90bbf83f577c74a499561ee005ad9bed1056977a529a4f3c8cd395a37e7f5b3c9e7f98c113a091ab75525589e91dc5f152d35ad209f6c066c0b69bc1193b92c6eb8781d5cccbc353f6d521cc37af3cac600c61df67a7117c8dfc5b33446276e2cc0515e859bea1dfd37aa4c238e665f655d1b14f5fd3"));
    println!("{:?}", res);
}
fn aaa() {
    let mynacard = MynaCard::search_card().unwrap();

    let mut responder = Responder::new(|data| {
        let mut buf = [0u8; 300];
        let buf = mynacard.transmit(data, &mut buf).unwrap();
        ApduRes::from_apdu(buf)
    });
    responder.select_jpki_ap().unwrap();
    responder.select_jpki_auth_key().unwrap();
    let sig = responder.compute_sig(myna::test_vector::PKCS1_ENCODED);

    println!("{:?}", sig);
}
