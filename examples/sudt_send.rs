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
    packed::OutPoint,
    prelude::{Entity, Pack},
};
use ckb_types::{packed::CellDep, prelude::Builder};
use std::{error::Error as StdErr, str::FromStr};

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
                h256!("0x6df1c2c75152567978b2dcae07ff18d484f986e9934afd1e631ffdab1cbf0efd").pack(),
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
fn main() -> Result<(), Box<dyn StdErr>> {
    let (cell_dep, script_id) = gen_dev_udt_script();

    let network_info = NetworkInfo::devnet();
    let mut configuration =
        TransactionBuilderConfiguration::new_with_network(network_info.clone())?;
    let udt_handler = ckb_sdk::transaction::handler::sudt::SudtHandler::new_with_customize(
        vec![cell_dep],
        script_id,
    );
    configuration.register_script_handler(Box::new(udt_handler));

    let sender = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqgx5lf4pczpamsfam48evs0c8nvwqqa59qapt46f")?;
    let receiver= Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4vqqyehpxn47deg5l6eeqtkfrt5kfkfchkwv62")?;
    let iterator = InputIterator::new_with_address(&[sender.clone()], &network_info);
    let mut builder = SudtTransactionBuilder::new(configuration, iterator, &sender, false)?;
    builder.add_output(&receiver, 51200);

    eprintln!("now ...");
    let mut tx_with_groups = builder.build(&Default::default())?;
    eprintln!("finished ..");
    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    let private_keys = vec![h256!(
        "0x6c9ed03816e3111e49384b8d180174ad08e29feb1393ea1b51cef1c505d4e36a"
    )];
    TransactionSigner::new(&network_info).sign_transaction(
        &mut tx_with_groups,
        &SignContexts::new_sighash_h256(private_keys)?,
    )?;

    let json_tx = ckb_jsonrpc_types::TransactionView::from(tx_with_groups.get_tx_view().clone());
    println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());

    let tx_hash = CkbRpcClient::new(network_info.url.as_str())
        .send_transaction(json_tx.inner, None)
        .expect("send transaction");

    println!(">>> tx {} sent! <<<", tx_hash);

    Ok(())
}
