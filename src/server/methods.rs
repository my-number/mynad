use jsonrpc_core::{
    types::{
        error::{Error as RpcErr, ErrorCode},
        Value,
        Params
    },
    IoHandler,
};
use std::error::Error;
use pcsc::{Card, Context, Scope};

pub fn register(io: &mut IoHandler) {

    io.add_method("get_readers", |_params: Params| {
        let context = Context::establish(Scope::User).unwrap();
        
        let buflen = context.list_readers_len().map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None
        })?;
        let mut buf: Vec<u8> = vec![0u8; buflen];
        let readerIter = context.list_readers(&mut buf).map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.description().to_string(),
            data: None
        })?;
        let readers: Vec<Value> = readerIter
            .map(|s| Value::String(s.to_str().unwrap().to_string()))
            .collect();

        context.release().map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.1.description().to_string(),
            data: None
        })?;
        Ok(Value::Array(readers))
    });

    io.add_method("open_reader", |params: Params|{
        let context = Context::establish(Scope::User).unwrap();
        
        
        context.release().map_err(|e| RpcErr {
            code: ErrorCode::ServerError(-1),
            message: e.1.description().to_string(),
            data: None
        })?;
        Ok(Value::Array(readers))
    });
}
