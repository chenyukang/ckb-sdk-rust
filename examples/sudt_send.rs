use anyhow::Error;
use ckb_sdk::{
    transaction::{
        builder::{sudt::SudtTransactionBuilder, CkbTransactionBuilder},
        input::InputIterator,
        signer::{SignContexts, TransactionSigner},
        TransactionBuilderConfiguration,
    },
    Address, CkbRpcClient, NetworkInfo, ScriptId,
};
use ckb_types::{
    core::DepType,
    h256,
    packed::{OutPoint, Script},
    prelude::{Entity, Pack},
    H256,
};
use ckb_types::{packed::CellDep, prelude::Builder};
use std::{convert::TryInto, error::Error as StdErr, str::FromStr};

// fn gen_udt_type_script() {
//     use ckb_types::{core::ScriptHashType, packed::Script};
//     let sudt_code_hash =
//         h256!("0xe1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419");
//     let lock_hash = h256!("0x4e58a837b1ac0a72adc56aa876dad176227a9c17c464ecd8e49451d72c79bc29");
//     let type_script = Script::new_builder()
//         .code_hash(sudt_code_hash.pack())
//         .hash_type(ScriptHashType::Data1.into())
//         .args(lock_hash.as_bytes().pack())
//         .build();
//     let script_id = ScriptId::from(&type_script);
//     eprintln!("type_script: {:?}", type_script);
//     eprintln!("script_id: {:?}", script_id);
// }

fn gen_dev_udt_script() -> (CellDep, ScriptId) {
    let (out_point, sudt_script_id) = (
        OutPoint::new_builder()
            .tx_hash(
                h256!("0xc24e3b64b9fb890ec319115713742029cefbd8cd2c9a47e9b4547192b23f3985").pack(),
            )
            .index(0u32.pack())
            .build(),
        ScriptId::new_data1(h256!(
            "0xe1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419"
        )),
    );

    let cell_dep = CellDep::new_builder()
        .out_point(out_point)
        .dep_type(DepType::Code.into())
        .build();
    return (cell_dep, sudt_script_id);
}

#[allow(dead_code)]
fn parse_u128(data: &[u8]) -> Result<u128, Error> {
    let data_bytes: Vec<u8> = data.into();
    let amount = u128::from_le_bytes(data_bytes.try_into().unwrap());
    eprintln!("amount: {:?}", amount);
    return Ok(amount);
}

#[allow(dead_code)]
fn test_amount() {
    let bytes = hex::decode("a0860100000000000000000000000000").expect("Decoding failed");
    let v1 = parse_u128(&bytes).unwrap();
    eprintln!("v1: {}", v1);

    let bytes = hex::decode("a0860100000000000000000000000000").expect("Decoding failed");
    let v1 = parse_u128(&bytes).unwrap();
    eprintln!("v1: {}", v1);

    let bytes = hex::decode("82090000000000000000000000000000").expect("Decoding failed");
    let v1 = parse_u128(&bytes).unwrap();
    eprintln!("v1: {}", v1);

    let bytes = hex::decode("ce810100000000000000000000000000").expect("Decoding failed");
    let v1 = parse_u128(&bytes).unwrap();
    eprintln!("v1: {}", v1);

    let bytes = hex::decode("f0810100000000000000000000000000").expect("Decoding failed");
    let v1 = parse_u128(&bytes).unwrap();
    eprintln!("v1: {}", v1);
}

fn send(
    sender_info: &(&str, H256),
    receiver: &str,
    amount: u128,
    udt_owner: Option<&str>,
) -> Result<(), Box<dyn StdErr>> {
    let (cell_dep, script_id) = gen_dev_udt_script();

    let network_info = NetworkInfo::devnet();
    let mut configuration =
        TransactionBuilderConfiguration::new_with_network(network_info.clone())?;
    let udt_handler = ckb_sdk::transaction::handler::sudt::SudtHandler::new_with_customize(
        vec![cell_dep],
        script_id,
    );
    configuration.register_script_handler(Box::new(udt_handler));

    let sender = Address::from_str(sender_info.0)?;
    let receiver = Address::from_str(receiver)?;
    let iterator = InputIterator::new_with_address(&[sender.clone()], &network_info);
    let udt_owner = if let Some(udt_owner) = udt_owner {
        Address::from_str(udt_owner)?
    } else {
        sender.clone()
    };
    const CKB_SHANNONS: u64 = 100_000_000;

    let mut builder = SudtTransactionBuilder::new(configuration, iterator, &udt_owner, false)?;
    let (account_ckb_amount, account_udt_amount) = builder.check()?;
    eprintln!(
        "account: {:?} udt_amount: {}",
        account_ckb_amount / CKB_SHANNONS,
        account_udt_amount
    );

    let ckb_amount = 100000000 * CKB_SHANNONS;
    eprintln!(
        "send ckb_amount: {} udt_amount: {}",
        ckb_amount / CKB_SHANNONS,
        amount
    );
    builder.add_output_with_capacity(&receiver, amount, ckb_amount);

    let mut tx_with_groups = builder.build(&Default::default())?;

    let _json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    //println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    let private_keys = vec![sender_info.1.clone()];
    TransactionSigner::new(&network_info).sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_sighash_h256(private_keys)?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!(
        "final tx: {}",
        serde_json::to_string_pretty(&json_tx).unwrap()
    );

    eprintln!("url: {:?}", network_info.url.as_str());
    // let tx_hash = CkbRpcClient::new(network_info.url.as_str())
    //     .send_transaction(json_tx.inner, None)
    //     .expect("send transaction");
    // println!(">>> tx {} sent! <<<", tx_hash);

    let result = CkbRpcClient::new(network_info.url.as_str())
        .test_tx_pool_accept(json_tx.inner, None)
        .expect("accept transaction");
    println!(">>> tx result: {:?}  <<<", result);

    Ok(())
}

fn main() {
    let wallets = [
        ("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqgx5lf4pczpamsfam48evs0c8nvwqqa59qapt46f", h256!("0xcccd5f7e693b60447623fb71a5983f15a426938c33699b1a81d1239cfa656cd1")),
        ("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4vqqyehpxn47deg5l6eeqtkfrt5kfkfchkwv62", h256!("0x85af6ff21ea891dbb384b771e02317427e7b66e84b4516c03d74ca4fd5ad0500")),
        ("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqtrnd9f2lh5vlwlj23dedf7jje65cdj8qs7q4awr", h256!("0xd00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb975ffffff")),
    ];

    //test_amount();
    //send(&wallets[0], wallets[1].0, 10000, None).unwrap();
    send(&wallets[0], wallets[2].0, 10000, None).unwrap();
    //send(&wallets[1], wallets[2].0, 100000, Some(wallets[0].0)).unwrap();
    //send(&wallets[2], wallets[0].0, 10000, Some(wallets[0].0)).unwrap();
}
