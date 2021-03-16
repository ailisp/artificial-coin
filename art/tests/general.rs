use near_sdk::json_types::U128;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};

use std::convert::TryInto;

extern crate art;
use art::ArtContract;

extern crate ausd;
use ausd::AUSDContract;

lazy_static::lazy_static! {
    static ref ART_WASM_BYTES: &'static [u8] = include_bytes!("../res/art.wasm").as_ref();
    static ref AUSD_WASM_BYTES: &'static [u8] = include_bytes!("../../ausd/res/ausd.wasm").as_ref();
}

const INIT_ART_BALANCE: &'static str = "1000000000";

fn init() -> (UserAccount, ContractAccount<ArtContract>, ContractAccount<AUSDContract>) {
    let master_account = init_simulator(None);
    let initial_balance = to_yocto(INIT_ART_BALANCE);
    let art = deploy! {
        contract: ArtContract,
        contract_id: "art",
        bytes: &ART_WASM_BYTES,
        signer_account: master_account,
        init_method: new(master_account.account_id(), initial_balance.to_string(), "ausd".to_string())
    };

    let ausd = deploy! {
        contract: AUSDContract,
        contract_id: "ausd",
        bytes: &AUSD_WASM_BYTES,
        signer_account: master_account,
        init_method: new(master_account.account_id(), 0u128.into(), "art".to_string())
    };

    (master_account, art, ausd)
}

#[test]
fn test_initial_issue() {
    let (master_account, art, ausd) = init();
    let deposit_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    let res = call!(master_account, art.stake_and_mint(deposit_amount));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(total_supply.0, (to_yocto(INIT_ART_BALANCE) / 2 * 20 / 5));
    let master_ausd_balance: U128 =
        view!(ausd.get_balance(master_account.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(total_supply, master_ausd_balance);
    let master_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(master_unstaked_art_balance, (to_yocto(INIT_ART_BALANCE) / 2).to_string());
    let master_staked_art_balance: String =
        view!(art.get_staked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(master_staked_art_balance, (to_yocto(INIT_ART_BALANCE) / 2).to_string());
}

#[test]
fn test_transfer_art() {
    let (master_account, art, ausd) = init();
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    let master_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(
        master_unstaked_art_balance,
        (to_yocto(INIT_ART_BALANCE) / 2 - to_yocto("10000")).to_string()
    );
    let master_staked_art_balance: String =
        view!(art.get_staked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(master_staked_art_balance, (to_yocto(INIT_ART_BALANCE) / 2).to_string());

    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, to_yocto("10000").to_string());

    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_staked_art_balance, (to_yocto("0")).to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(alice, art.transfer(bob.account_id(), to_yocto("1000").to_string())).assert_success();

    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, to_yocto("9000").to_string());

    let bob_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(bob_unstaked_art_balance, to_yocto("1000").to_string());
}

#[test]
fn test_mint_ausd() {
    let (master_account, art, ausd) = init();
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    let res = call!(alice, art.stake_and_mint(to_yocto("10000").to_string()));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(total_supply.0, ((to_yocto(INIT_ART_BALANCE) / 2 + to_yocto("10000")) * 20 / 5));

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), alice_ausd_balance);
    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, (to_yocto("0")).to_string());
    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_staked_art_balance, (to_yocto("10000")).to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    assert!(!call!(alice, art.transfer(bob.account_id(), to_yocto("1000").to_string())).is_ok());

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
    let (master_account, art, ausd) = init();
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    call!(alice, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();
    let r = call!(alice, art.burn_to_unstake(to_yocto("10000").to_string()));
    println!("{:?}", r);
    r.assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("0")), alice_ausd_balance);
    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, (to_yocto("10000")).to_string());
    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_staked_art_balance, (to_yocto("0")).to_string());
}

#[test]
fn test_unstake_when_price_change() {
    let (master_account, art, ausd) = init();
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    call!(alice, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    println!("{:?}", alice_ausd_balance);

    // price of art rise, ausd/art falls

    call!(
        master_account,
        art.submit_price("4000000000".to_string()), // every art is now $40
        gas = DEFAULT_GAS * 4
    )
    .assert_success();

    let r = call!(alice, art.burn_to_unstake(to_yocto("5000").to_string()));
    println!("{:?}", r);
    r.assert_success();

    // alice restake her art
    call!(alice, art.stake_and_mint(to_yocto("5000").to_string())).assert_success();
    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("5000") * 40 / 5), alice_ausd_balance);

    // now price goes down
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();

    // now bob stake and mint
    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(bob.account_id(), to_yocto("10000").to_string()))
        .assert_success();
    call!(bob, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();
    let bob_ausd_balance: U128 =
        view!(ausd.get_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), bob_ausd_balance);
}

#[test]
fn test_exchange_ausd_abtc() {
    let (master_account, art, ausd) = init();
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS * 4
    )
    .assert_success();
    call!(master_account, art.submit_asset_price("aBTC".to_string(), "3000000000000".to_string()))
        .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(master_account, art.transfer(alice.account_id(), to_yocto("10000").to_string()))
        .assert_success();

    call!(alice, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();
    call!(alice, art.buy_asset_with_ausd("aBTC".to_string(), to_yocto("1").to_string()))
        .assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000")), alice_ausd_balance);
    let alice_abtc_balance: u128 =
        view!(art.get_asset_balance(&alice.account_id().try_into().unwrap(), &"aBTC".to_string()))
            .unwrap_json();
    assert_eq!(alice_abtc_balance, to_yocto("1").to_string());
}
