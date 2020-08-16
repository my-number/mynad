use hex::FromHexError;
use jsonrpc_core::types::error::{Error as RpcErr, ErrorCode as RpcErrCode};
use jsonrpc_core::types::Value;
use myna::error::ApduError;
use pcsc;

#[repr(i64)]
pub enum ErrorCode {
    PcscError = -1,
    UnknownError = -2,
    CommandError = -3,
    PinIncorrect = -4
}

pub enum Error {
    Pcsc(pcsc::Error),
    Command(u8,u8),
    Execution(&'static str),
    Parse,
    PinIncorrect(u8),
    Other,
}

impl From<pcsc::Error> for Error {
    fn from(e: pcsc::Error) -> Self {
        Self::Pcsc(e)
    }
}
impl From<ApduError<pcsc::Error>> for Error {
    fn from(e: ApduError<pcsc::Error>) -> Self {
        match e {
            ApduError::Transmission(e) => Self::Pcsc(e),
            ApduError::Command(sw1, sw2) => Self::Command(sw1,sw2),
            ApduError::Execution(s) => Self::Execution(s),
            ApduError::PinIncorrect(count) => Self::PinIncorrect(count),
            _ => Self::Other,
        }
    }
}
impl From<FromHexError> for Error {
    fn from(_: FromHexError) -> Self {
        Self::Parse
    }
}

impl Into<RpcErr> for Error {
    fn into(self) -> RpcErr {
        match self {
            Error::Pcsc(e) => RpcErr {
                code: (ErrorCode::PcscError as i64).into(),
                message: e.to_string(),
                data: Some(Value::Number((e as i64).into())),
            },
            Error::PinIncorrect(count) => RpcErr{
                code: (ErrorCode::PinIncorrect as i64).into(),
                message: "PIN is incorrect".into(),
                data: Some(Value::Number(count.into())),
            },
            Error::Command(sw1,sw2) => RpcErr{
                code: (ErrorCode::CommandError as i64).into(),
                message: "Command Execution Failed".into(),
                data: Some(Value::Array(vec![sw1.into(),sw2.into()])),
            },
            
            _ => RpcErr {
                code: RpcErrCode::ServerError(ErrorCode::UnknownError as i64),
                message: "Unknown Error".to_string(),
                data: None,
            },
        }
    }
}
