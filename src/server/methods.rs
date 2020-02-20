use jsonrpc_core::{
    types::{
        error::{Error as RpcErr, ErrorCode},
        Value,
    },
    IoHandler, Result,
};
use myna::card::{
    apdu::*
};
use std::sync::{Arc, RwLock};
use jsonrpc_derive::rpc;
use pcsc::{Card, Context, Scope};
use std::error::Error;
use std::ffi::CStr;
use hex::decode;
use serde_json::json;
static mut ctx: Option<&'static Context> = None;


#[rpc]
pub trait Rpc {
    #[rpc(name = "getReaders")]
    fn get_reader(&self) -> Result<Vec<String>>;
    #[rpc(name = "openReader")]
    fn open_reader(&self, name: String) -> Result<u64>;
    #[rpc(name = "getStatus")]
    fn get_status(&self, fd: u64) -> Result<Value>;
    #[rpc(name = "getCert")]
    fn get_cert(&self, fd: u64) -> Result<Value>;
    #[rpc(name = "computeSig")]
    fn compute_sig(&self, fd: u64, pin: String, hashHex: String) -> Result<Value>;
}

pub struct RpcImpl {
    ctx: Context,
    fd: Arc<RwLock<Vec<Card>>>
}

impl Default for RpcImpl {
    fn default() -> Self {
        Self {
            ctx: Context::establish(Scope::User).unwrap(),
            fd: Arc::new(RwLock::new(Vec::new()))
        }
    }
}
impl Rpc for RpcImpl {
    fn get_reader(&self) -> Result<Vec<String>> {
        let context = &self.ctx;

        let buflen = context.list_readers_len().map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None,
        })?;
        let mut buf: Vec<u8> = vec![0u8; buflen];
        let readerIter = context.list_readers(&mut buf).map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None,
        })?;
        let readers: Vec<String> = readerIter
            .map(|s| s.to_str().unwrap().to_string())
            .collect();
        
        Ok(readers)
    }
    fn open_reader(&self, name: String) -> Result<u64> {
        let context = &self.ctx;
        let mut vname = name.as_bytes().to_vec();
        vname.push(0u8);
        
        let card = context
            .connect(
                CStr::from_bytes_with_nul(&vname[..]).unwrap(),
                pcsc::ShareMode::Exclusive,
                pcsc::Protocols::ANY,
            )
            .map_err(|e| RpcErr {
                code: ErrorCode::ServerError(-1),
                message: e.description().to_string(),
                data: None,
            })?;
        self.fd.write().unwrap().push(card);
        Ok((self.fd.read().unwrap().len() - 1) as u64)
    }
    fn get_status(&self, fd: u64) -> Result<Value> {
        let fds = self.fd.read().unwrap();
        let card = &fds[fd as usize];
        let (name_len, atr_len) = card.status2_len().map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None,
        })?;
        let mut name_buf = vec![0u8; name_len];
        let mut atr_buf = vec![0u8; atr_len];
        let status = card.status2(&mut name_buf[..], &mut atr_buf[..]).map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None,
        })?;
        let readers: Vec<String> = status.reader_names()
            .map(|s| s.to_str().unwrap().to_string())
            .collect();
        //Ok(1)
        Ok(json!({
            "name": readers,
            "atr": status.atr()
        }))
    }
    fn get_cert(&self, fd: u64) -> Result<Value>{
        let fds = self.fd.read().unwrap();
        let card = &fds[fd as usize];
        let mut responder = Apdu::new(|data| {
            let mut buf = [0u8; 300];
            ApduRes::from_apdu(card.transmit(data, &mut buf).unwrap())
        });
        responder.select_jpki_ap().unwrap();
        responder.select_jpki_cert_auth().unwrap();
        let cert = responder.read_binary().unwrap();
        Ok(json!({
            "cert": cert
        }))
    }
    fn compute_sig(&self, fd: u64, pin: String, hashHex: String) -> Result<Value>{
        let fds = self.fd.read().unwrap();
        let card = &fds[fd as usize];
        let mut responder = Apdu::new(|data| {
            let mut buf = [0u8; 300];
            ApduRes::from_apdu(card.transmit(data, &mut buf).unwrap())
        });
        responder.select_jpki_ap().unwrap();
        responder.select_jpki_auth_pin().unwrap();
        responder.verify_pin(pin.as_str()).unwrap();
        responder.select_jpki_auth_key().unwrap();
        let hash = decode(hashHex).unwrap();
        let sig = responder.compute_sig(&hash[..]);
        Ok(json!({
            "sig": sig
        }))
    }
}
