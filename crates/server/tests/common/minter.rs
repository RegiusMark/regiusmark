use super::create_tx_header;
use godcoin::{
    blockchain::{GenesisBlockInfo, ReindexOpts},
    prelude::*,
};
use godcoin_server::{prelude::*, process_message, ServerData, WsState};
use sodiumoxide::randombytes;
use std::{
    env, fs,
    io::Cursor,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio_tungstenite::tungstenite::Message;

type Indexed = bool;

pub struct TestMinter(ServerData, GenesisBlockInfo, PathBuf, Indexed);

impl TestMinter {
    pub fn new() -> Self {
        godcoin::init().unwrap();
        let tmp_dir = {
            let mut tmp_dir = env::temp_dir();
            let mut num: [u8; 8] = [0; 8];
            randombytes::randombytes_into(&mut num);
            tmp_dir.push(&format!("godcoin_test_{}", u64::from_be_bytes(num)));
            tmp_dir
        };
        fs::create_dir(&tmp_dir).expect(&format!("Could not create temp dir {:?}", &tmp_dir));

        let blocklog_loc = &Path::join(&tmp_dir, "blklog");
        let index_loc = &Path::join(&tmp_dir, "index");
        let chain = Arc::new(Blockchain::new(blocklog_loc, index_loc));
        let minter_key = KeyPair::gen();
        let info = chain.create_genesis_block(minter_key.clone());

        {
            let txs = {
                let mut txs = Vec::with_capacity(1);

                let mut tx = TxVariant::V0(TxVariantV0::MintTx(MintTx {
                    base: create_tx_header("0.00000 GRAEL"),
                    to: (&info.script).into(),
                    amount: "1000.00000 GRAEL".parse().unwrap(),
                    attachment: vec![1, 2, 3],
                    attachment_name: "".to_owned(),
                    script: info.script.clone(),
                }));

                tx.append_sign(&info.wallet_keys[1]);
                tx.append_sign(&info.wallet_keys[0]);
                txs.push(tx);

                txs.push(TxVariant::V0(TxVariantV0::RewardTx(RewardTx {
                    base: Tx {
                        fee: "0.00000 GRAEL".parse().unwrap(),
                        timestamp: 0,
                        signature_pairs: Vec::new(),
                    },
                    to: (&info.script).into(),
                    rewards: Asset::default(),
                })));
                txs
            };

            let head = chain.get_chain_head();
            let child = match head.as_ref() {
                Block::V0(block) => {
                    let mut b = block.new_child(txs);
                    b.sign(&info.minter_key);
                    b
                }
            };
            chain.insert_block(child).unwrap();
        }

        let client_pool = ClientPool::default();
        let minter = Minter::new(Arc::clone(&chain), minter_key, client_pool.clone(), false);
        let data = ServerData {
            chain,
            minter,
            client_pool,
        };
        Self(data, info, tmp_dir, true)
    }

    pub fn unindexed(&mut self) {
        let unindexed_path = {
            let mut unindexed_path = self.2.clone();
            let mut num: [u8; 8] = [0; 8];
            randombytes::randombytes_into(&mut num);
            unindexed_path.push(&format!("unindexed_{}", u64::from_be_bytes(num)));
            unindexed_path
        };
        fs::create_dir(&unindexed_path)
            .expect(&format!("Could not create temp dir {:?}", &unindexed_path));
        fs::copy(self.2.join("blklog"), unindexed_path.join("blklog"))
            .expect("Could not copy block log");

        let blocklog_loc = &Path::join(&unindexed_path, "blklog");
        let index_loc = &Path::join(&unindexed_path, "index");
        self.0.chain = Arc::new(Blockchain::new(blocklog_loc, index_loc));
        self.3 = false;
    }

    pub fn reindex(&mut self) {
        let chain = Arc::clone(&self.0.chain);
        assert_eq!(chain.index_status(), IndexStatus::None);
        chain.reindex(ReindexOpts { auto_trim: true });
        let key = self.1.minter_key.clone();
        let pool = self.0.client_pool.clone();
        self.0.minter = Minter::new(chain, key, pool, false);
        self.3 = true;
    }

    pub fn chain(&self) -> &Blockchain {
        &self.0.chain
    }

    pub fn genesis_info(&self) -> &GenesisBlockInfo {
        &self.1
    }

    pub fn produce_block(&self) -> Result<(), verify::BlockErr> {
        self.0.minter.force_produce_block(true)
    }

    pub fn request(&self, body: RequestBody) -> ResponseBody {
        let (tx, _) = futures::sync::mpsc::unbounded();
        let mut state = WsState::new(SocketAddr::from(([127, 0, 0, 1], 7777)), tx);
        let res = self.send_request(&mut state, net::Request { id: 0, body });
        res.body
    }

    pub fn send_request(&self, state: &mut WsState, req: net::Request) -> net::Response {
        let mut buf = Vec::with_capacity(1_048_576);
        req.serialize(&mut buf);
        self.raw_request(state, buf)
    }

    pub fn raw_request(&self, state: &mut WsState, req_bytes: Vec<u8>) -> net::Response {
        assert!(
            self.3,
            "attempting to send a request to an unindexed minter"
        );

        let res = match process_message(&self.0, state, Message::Binary(req_bytes)).unwrap() {
            Message::Binary(res) => res,
            _ => panic!("Expected binary response"),
        };
        let mut cur = Cursor::<&[u8]>::new(&res);
        net::Response::deserialize(&mut cur).unwrap()
    }
}

impl Drop for TestMinter {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.2).expect("Failed to rm dir");
    }
}
