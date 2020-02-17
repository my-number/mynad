use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{ServerBuilder};
use jsonrpc_derive::rpc;
mod methods;

pub fn start(){
    let mut io = IoHandler::new();

    methods::register(&mut io);
    
    let server = ServerBuilder::new(io)
		    .threads(3)
		    .start_http(&"127.0.0.1:3030".parse().unwrap())
		    .unwrap();
    server.wait();
}
