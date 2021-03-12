use near_sdk::serde_json::json;
use near_sdk::{env, json_types::U128};
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};

use std::convert::TryInto;

extern crate agov;
use agov::AGovContract;

extern crate ausd;
use ausd::AUSDContract;

lazy_static::lazy_static! {
    static ref AGOV_WASM_BYTES: &'static [u8] = include_bytes!("../res/agov.wasm").as_ref();
    static ref AUSD_WASM_BYTES: &'static [u8] = include_bytes!("../../ausd/res/ausd.wasm").as_ref();
}

const INIT_AGOV_BALANCE: &'static str = "1000000000";

fn init() -> (UserAccount, ContractAccount<AGovContract>, ContractAccount<AUSDContract>) {
    let master_account = init_simulator(None);
    let initial_balance = to_yocto(INIT_AGOV_BALANCE);
    let agov = deploy! {
        contract: AGovContract,
        contract_id: "agov",
        bytes: &AGOV_WASM_BYTES,
        signer_account: master_account,
        init_method: new(master_account.account_id(), initial_balance.to_string(), "ausd".to_string())
    };

    let ausd = deploy! {
        contract: AUSDContract,
        contract_id: "ausd",
        bytes: &AUSD_WASM_BYTES,
        signer_account: master_account,
        init_method: new(master_account.account_id(), 0u128.into(), "agov".to_string())
    };

    (master_account, agov, ausd)
}

#[test]
fn test_initial_issue() {
    let (master_account, agov, ausd) = init();
    let deposit_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    let res =
        call!(master_account, agov.deposit_and_mint(master_account.account_id(), deposit_amount));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(total_supply.0, (to_yocto(INIT_AGOV_BALANCE) / 2 * 20 / 5));
    let master_ausd_balance: U128 =
        view!(ausd.get_balance(master_account.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(total_supply, master_ausd_balance);
    let master_unlocked_agov_balance: String =
        view!(agov.get_unlocked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(master_unlocked_agov_balance, (to_yocto(INIT_AGOV_BALANCE) / 2).to_string());
    let master_locked_agov_balance: String = view!(agov.get_locked_balance(
        master_account.account_id().try_into().unwrap(),
        master_account.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(master_locked_agov_balance, (to_yocto(INIT_AGOV_BALANCE) / 2).to_string());
}

#[test]
fn test_transfer_agov() {
    let (master_account, agov, ausd) = init();
    let deposit_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, agov.deposit_and_mint(master_account.account_id(), deposit_amount))
        .assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    let master_unlocked_agov_balance: String =
        view!(agov.get_unlocked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(
        master_unlocked_agov_balance,
        (to_yocto(INIT_AGOV_BALANCE) / 2 - to_yocto("10000")).to_string()
    );
}
