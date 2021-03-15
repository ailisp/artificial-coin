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
        call!(master_account, agov.stake_and_mint(master_account.account_id(), deposit_amount));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(total_supply.0, (to_yocto(INIT_AGOV_BALANCE) / 2 * 20 / 5));
    let master_ausd_balance: U128 =
        view!(ausd.get_balance(master_account.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(total_supply, master_ausd_balance);
    let master_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(master_unstaked_agov_balance, (to_yocto(INIT_AGOV_BALANCE) / 2).to_string());
    let master_staked_agov_balance: String = view!(agov.get_staked_balance(
        master_account.account_id().try_into().unwrap(),
        master_account.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(master_staked_agov_balance, (to_yocto(INIT_AGOV_BALANCE) / 2).to_string());
}

#[test]
fn test_transfer_agov() {
    let (master_account, agov, ausd) = init();
    let stake_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, agov.stake_and_mint(master_account.account_id(), stake_amount))
        .assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    let master_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(
        master_unstaked_agov_balance,
        (to_yocto(INIT_AGOV_BALANCE) / 2 - to_yocto("10000")).to_string()
    );
    let master_staked_agov_balance: String = view!(agov.get_staked_balance(
        master_account.account_id().try_into().unwrap(),
        master_account.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(master_staked_agov_balance, (to_yocto(INIT_AGOV_BALANCE) / 2).to_string());

    let alice_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_agov_balance, to_yocto("10000").to_string());

    let alice_staked_agov_balance: String = view!(agov.get_staked_balance(
        alice.account_id().try_into().unwrap(),
        alice.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(alice_staked_agov_balance, (to_yocto("0")).to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(alice, agov.transfer(bob.account_id(), to_yocto("1000").to_string())).assert_success();

    let alice_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_agov_balance, to_yocto("9000").to_string());

    let bob_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(bob_unstaked_agov_balance, to_yocto("1000").to_string());
}

#[test]
fn test_mint_ausd() {
    let (master_account, agov, ausd) = init();
    let stake_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, agov.stake_and_mint(master_account.account_id(), stake_amount))
        .assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    let res = call!(alice, agov.stake_and_mint(alice.account_id(), to_yocto("10000").to_string()));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(total_supply.0, ((to_yocto(INIT_AGOV_BALANCE) / 2 + to_yocto("10000")) * 20 / 5));

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), alice_ausd_balance);
    let alice_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_agov_balance, (to_yocto("0")).to_string());
    let alice_staked_agov_balance: String = view!(agov.get_staked_balance(
        alice.account_id().try_into().unwrap(),
        alice.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(alice_staked_agov_balance, (to_yocto("10000")).to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    assert!(!call!(alice, agov.transfer(bob.account_id(), to_yocto("1000").to_string())).is_ok());

    // Alice can use her ausd freely
    call!(
        alice,
        ausd.transfer(bob.account_id(), U128(to_yocto("30000"))),
        deposit = STORAGE_AMOUNT / 10
    )
    .assert_success();
    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5 - to_yocto("30000")), alice_ausd_balance);
    let bob_ausd_balance: U128 =
        view!(ausd.get_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("30000")), bob_ausd_balance);
}

#[test]
fn test_burn_unstake() {
    let (master_account, agov, ausd) = init();
    let stake_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, agov.stake_and_mint(master_account.account_id(), stake_amount))
        .assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    call!(alice, agov.stake_and_mint(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    let r = call!(alice, agov.burn_to_unstake(alice.account_id(), to_yocto("10000").to_string()));
    println!("{:?}", r);
    r.assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("0")), alice_ausd_balance);
    let alice_unstaked_agov_balance: String =
        view!(agov.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_agov_balance, (to_yocto("10000")).to_string());
    let alice_staked_agov_balance: String = view!(agov.get_staked_balance(
        alice.account_id().try_into().unwrap(),
        alice.account_id().try_into().unwrap()
    ))
    .unwrap_json();
    assert_eq!(alice_staked_agov_balance, (to_yocto("0")).to_string());
}

#[test]
fn test_unstake_when_price_change() {
    let (master_account, agov, ausd) = init();
    let stake_amount = (to_yocto(INIT_AGOV_BALANCE) / 2).to_string();
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, agov.stake_and_mint(master_account.account_id(), stake_amount))
        .assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    call!(alice, agov.stake_and_mint(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    println!("{:?}", alice_ausd_balance);

    // price of agov rise, ausd/agov falls

    call!(
        master_account,
        agov.submit_price("4000000000".to_string()), // every agov is now $40
        gas = DEFAULT_GAS * 4
    )
    .assert_success();

    let r = call!(alice, agov.burn_to_unstake(alice.account_id(), to_yocto("5000").to_string()));
    println!("{:?}", r);
    r.assert_success();

    // alice restake her agov
    call!(alice, agov.stake_and_mint(alice.account_id(), to_yocto("5000").to_string()))
        .assert_success();
    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("5000") * 40 / 5), alice_ausd_balance);

    // now price goes down
    call!(
        master_account,
        agov.submit_price("2000000000".to_string()), // every agov is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();

    // now bob stake and mint
    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(master_account, agov.transfer(bob.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    call!(bob, agov.stake_and_mint(bob.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    let bob_ausd_balance: U128 =
        view!(ausd.get_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), bob_ausd_balance);
}
