mod types;
use codec::{Encode};

use sp_core::{Blake2Hasher, Hasher};
fn main() {
    println!("encoder");
    let tx = types::TxCreateAccount {
        cert: vec![1,2,3,4],
        nonce: 0
    };
    let tbs = types::Tx::CreateAccount(tx);
    let encoded = tbs.encode();
    println!("{:?}", &encoded);
    let hash = Blake2Hasher::hash(&encoded);
    println!("{:?}", hash.as_ref());
}
