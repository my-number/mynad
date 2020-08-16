use jsonrpc_http_server::jsonrpc_core::*;
use jsonrpc_http_server::ServerBuilder;

mod error;
mod methods;
mod utils;
use methods::{Methods, RpcImpl};
fn main() {
    let mut io = IoHandler::default();
    let rpc: RpcImpl = Default::default();
    io.extend_with(rpc.to_delegate());

    let server = ServerBuilder::new(io)
        .start_http(&"0.0.0.0:3030".parse().unwrap())
        .expect("Server must start with no issues");

    server.wait()
}
