use jsonrpc_core::{Result, IoHandler};
use jsonrpc_http_server::{ServerBuilder};
use jsonrpc_derive::rpc;

#[rpc]
pub trait Rpc {
	  /// Adds two numbers and returns a result
	  #[rpc(name = "get_readers")]
	  fn get_readers(&self) -> Result<()>;
}

pub struct RpcImpl;
impl Rpc for RpcImpl {
	  fn get_readers(&self) -> Result<()> {
		    Ok(())
	  }
}


pub fn start(){
    let mut io = IoHandler::new();
    io.extend_with(RpcImpl.to_delegate());
    let server = ServerBuilder::new(io)
		    .threads(3)
		    .start_http(&"127.0.0.1:3030".parse().unwrap())
		    .unwrap();
    server.wait();
}
