use crate::error::Error;
use crate::utils::open_card;
use hex::decode;
use jsonrpc_core::types::Value;
use jsonrpc_derive::rpc;
use myna::card::{Apdu, KeyType};
use myna::utils::check_pin;

use pcsc::{Card, Context, Disposition, Protocols, Scope, ShareMode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct MynumberCardInfo {
    /// Auth PIN remaining count
    authPinRemaining: u8,
    /// Sign PIN remaining count
    signPinRemaining: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Reader {
    /// Reader name
    name: String,
    /// PCSC Error code
    error: Option<u32>,
    mynumberCardInfo: Option<MynumberCardInfo>,
}

struct Responder(Card);
impl Apdu for Responder {
    type TransErr = pcsc::Error;
    fn transmit(&self, data: Vec<u8>) -> Result<Vec<u8>, Self::TransErr> {
        let card = &self.0;

        let mut buf = [0u8; 300];
        let result = card.transmit(&data[..], &mut buf)?;
        Ok(result.to_vec())
    }
}
impl Responder {
    fn check_mynumber_card(&self) -> Result<(), Error> {
        if self.is_mynumber_card()? {
            Ok(())
        } else {
            Err(Error::Execution("Card is not mynumber card"))
        }
    }
}

#[rpc(server)]
pub trait Methods {
    #[rpc(name = "getVersion")]
    fn get_version(&self) -> Result<String, Error>;
    #[rpc(name = "getReaders")]
    fn get_readers(&self) -> Result<Vec<Reader>, Error>;
    #[rpc(name = "getReaderStatus")]
    fn get_reader_status(&self, name: String) -> Result<Value, Error>;
    #[rpc(name = "getAuthCert")]
    fn get_auth_cert(&self, name: String) -> Result<Value, Error>;
    #[rpc(name = "getSignCert")]
    fn get_sign_cert(&self, name: String) -> Result<Value, Error>;
    #[rpc(name = "computeAuthSig")]
    fn compute_auth_sig(&self, name: String, pin: String, hash_hex: String)
        -> Result<Value, Error>;
}

pub struct RpcImpl {}
impl Default for RpcImpl {
    fn default() -> Self {
        Self {}
    }
}

impl Methods for RpcImpl {
    fn get_version(&self) -> Result<String, Error> {
        Ok(env!("CARGO_PKG_VERSION").to_string())
    }

    fn get_readers(&self) -> Result<Vec<Reader>, Error> {
        let context = Context::establish(Scope::User)?;
        let buflen = context.list_readers_len()?;
        let mut buf: Vec<u8> = vec![0u8; buflen];
        let reader_iter = context.list_readers(&mut buf)?;
        let readers: Vec<Reader> = reader_iter.map(|raw_name| {
                let card_result = context.connect(raw_name, ShareMode::Shared, Protocols::ANY);

                let mut mynumber_card_info: Option<MynumberCardInfo> = None;
                let error: Option<u32> = match card_result {
                    Ok(card) => {
                        let responder = Responder(card);
                        let is_mynumber_card = responder.is_mynumber_card().unwrap_or(false);
                        if is_mynumber_card {
                            mynumber_card_info = Some(MynumberCardInfo {
                                authPinRemaining: responder
                                    .get_retry_counter(KeyType::UserAuth)
                                    .unwrap_or(0u8),
                                signPinRemaining: responder
                                    .get_retry_counter(KeyType::DigitalSign)
                                    .unwrap_or(0u8),
                            })
                        }
                        let _ = responder.0.disconnect(Disposition::LeaveCard);
                        None
                    }
                    Err(e) => Some(e as u32),
                };
                Reader {
                    name: raw_name
                        .to_str()
                        .expect("PCSC reader names is always valid UTF8")
                        .to_string(),
                    error,
                    mynumberCardInfo: mynumber_card_info,
                }
            })
            .collect();
        Ok(readers)
    }
    fn get_reader_status(&self, name: String) -> Result<Value, Error> {
        let card = open_card(name)?;
        let (name_len, atr_len) = card.status2_len()?;
        let mut name_buf = vec![0u8; name_len];
        let mut atr_buf = vec![0u8; atr_len];
        let status = card.status2(&mut name_buf[..], &mut atr_buf[..])?;
        let readers: Vec<String> = status
            .reader_names()
            .map(|s| s.to_str().unwrap().to_string())
            .collect();

        Ok(json!({
            "name": readers,
            "atr": status.atr()
        }))
    }
    fn get_auth_cert(&self, name: String) -> Result<Value, Error> {
        let card = open_card(name)?;

        let responder = Responder(card);

        responder.check_mynumber_card()?;
        let cert = responder.get_cert(KeyType::UserAuth)?;
        Ok(json!({ "cert": cert }))
    }
    fn get_sign_cert(&self, name: String) -> Result<Value, Error> {
        let card = open_card(name)?;

        let responder = Responder(card);

        responder.check_mynumber_card()?;
        let cert = responder.get_cert(KeyType::DigitalSign)?;
        Ok(json!({ "cert": cert }))
    }
    fn compute_auth_sig(
        &self,
        name: String,
        pin: String,
        hash_hex: String,
    ) -> Result<Value, Error> {
        if !check_pin(&pin) {
            return Err(Error::Execution("PIN is invalid"));
        }
        let card = open_card(name)?;

        let responder = Responder(card);

        responder.check_mynumber_card()?;
        let cert = responder.get_cert(KeyType::UserAuth)?;
        let hash = decode(hash_hex)?;
        let sig = responder.compute_sig(&pin, &hash[..], KeyType::UserAuth)?;
        Ok(json!({ "sig": sig, "cert": cert}))
    }
}
