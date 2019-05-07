use actix::prelude::*;
use godcoin::prelude::*;

mod common;

pub use common::*;

#[test]
fn fresh_blockchain() {
    System::run(|| {
        let minter = TestMinter::new();
        let chain = minter.chain();
        assert!(chain.get_block(0).is_some());
        assert_eq!(chain.get_chain_height(), 0);

        let owner = chain.get_owner();
        assert_eq!(owner.minter, minter.genesis_info().minter_key.0);
        assert_eq!(
            owner.script,
            script::Builder::new().push(OpFrame::False).build()
        );
        assert_eq!(owner.wallet, (&minter.genesis_info().script).into());

        assert!(chain.get_block(1).is_none());
        System::current().stop();
    })
    .unwrap();
}

#[test]
fn mint_tx_verification() {
    System::run(|| {
        let minter = TestMinter::new();
        let chain = minter.chain();
        let config = VerifyConfig::strict();

        let mut tx = MintTx {
            base: create_tx(TxType::MINT, "0 GOLD"),
            to: (&minter.genesis_info().script).into(),
            amount: Balance::default(),
            script: minter.genesis_info().script.clone(),
        };

        tx.append_sign(&minter.genesis_info().wallet_keys[0]);
        tx.append_sign(&minter.genesis_info().wallet_keys[3]);

        let tx = TxVariant::MintTx(tx);
        assert!(chain.verify_tx(&tx, &[], config).is_ok());

        System::current().stop();
    })
    .unwrap();
}

#[test]
#[ignore]
fn mint_tx_updates_balances() {
    System::run(|| {
        let minter = TestMinter::new();
        let chain = minter.chain();

        let mut tx = MintTx {
            base: create_tx(TxType::MINT, "0 GOLD"),
            to: (&minter.genesis_info().script).into(),
            amount: get_balance("10.0 GOLD", "1000 SILVER"),
            script: minter.genesis_info().script.clone(),
        };

        tx.append_sign(&minter.genesis_info().wallet_keys[0]);
        tx.append_sign(&minter.genesis_info().wallet_keys[3]);

        let tx = TxVariant::MintTx(tx);
        let res = minter.request(MsgRequest::Broadcast(tx));
        assert!(!res.is_err(), format!("{:?}", res));

        let props = chain.get_properties();
        assert_eq!(props.token_supply, get_balance("10 GOLD", "1000 SILVER"));

        System::current().stop();
    })
    .unwrap();
}
