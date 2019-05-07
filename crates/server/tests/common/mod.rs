use actix::prelude::*;
use godcoin::{blockchain::GenesisBlockInfo, prelude::*};
use godcoin_server::{handle_request, prelude::*, ServerData};
use sodiumoxide::randombytes;
use std::{
    env, fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct TestMinter(ServerData, GenesisBlockInfo, PathBuf);

impl TestMinter {
    pub fn new() -> Self {
        godcoin::init().unwrap();
        let mut tmp_dir = env::temp_dir();
        {
            let mut s = String::from("godcoin_test_");
            let mut num: [u8; 8] = [0; 8];
            randombytes::randombytes_into(&mut num);
            s.push_str(&format!("{}", u64::from_be_bytes(num)));
            tmp_dir.push(s);
        }
        fs::create_dir(&tmp_dir).expect(&format!("Could not create temp dir {:?}", &tmp_dir));

        let chain = Arc::new(Blockchain::new(&tmp_dir));
        let minter_key = KeyPair::gen_keypair();
        let info = chain.create_genesis_block(minter_key.clone());

        let minter = Minter::new(Arc::clone(&chain), minter_key, (&info.script).into()).start();
        let data = ServerData { chain, minter };
        Self(data, info, tmp_dir)
    }

    pub fn chain(&self) -> &Blockchain {
        &self.0.chain
    }

    pub fn genesis_info(&self) -> &GenesisBlockInfo {
        &self.1
    }

    pub fn request(&self, req: MsgRequest) -> MsgResponse {
        handle_request(&self.0, req)
    }
}

impl Drop for TestMinter {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.2).expect("Failed to rm dir");
    }
}

pub fn get_balance(gold: &str, silver: &str) -> Balance {
    let gold = gold.parse().unwrap();
    let silver = silver.parse().unwrap();
    Balance::from(gold, silver).unwrap()
}

pub fn create_tx(tx_type: TxType, fee: &str) -> Tx {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    Tx {
        tx_type,
        timestamp,
        fee: fee.parse().unwrap(),
        signature_pairs: Vec::with_capacity(8),
    }
}
