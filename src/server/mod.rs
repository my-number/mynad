use jsonrpc_core::IoHandler;
use jsonrpc_derive::rpc;
mod methods;
use methods::{Rpc, RpcImpl};
use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, RestApi, ServerBuilder};

pub fn start(){
    let mut io = IoHandler::new();
    let rpc: RpcImpl = Default::default();
    io.extend_with(rpc.to_delegate());
    
    let server = ServerBuilder::new(io)
		    .threads(3)
        
		    .rest_api(RestApi::Unsecure)
        .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]))
		    .start_http(&"127.0.0.1:3030".parse().unwrap())
		    .unwrap();
    server.wait();
}
