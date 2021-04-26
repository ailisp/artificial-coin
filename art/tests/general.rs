use near_sdk::json_types::U128;
use near_sdk_sim::{
    call, deploy, init_simulator,
    runtime::{GenesisConfig, RuntimeStandalone},
    to_yocto, view, ContractAccount, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT,
};
use std::convert::TryInto;
use std::{cell::RefCell, rc::Rc};

extern crate art;
use art::ArtContract;

extern crate ausd;
use ausd::AUSDContract;

lazy_static::lazy_static! {
    static ref ART_WASM_BYTES: &'static [u8] = include_bytes!("../res/art.wasm").as_ref();
    static ref AUSD_WASM_BYTES: &'static [u8] = include_bytes!("../../ausd/res/ausd.wasm").as_ref();
}

const INIT_ART_BALANCE: &'static str = "1000000000";

fn init(
    genesis: Option<GenesisConfig>,
) -> (
    UserAccount,
    ContractAccount<ArtContract>,
    ContractAccount<AUSDContract>,
) {
    let master_account = init_simulator(genesis);
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
    let (master_account, art, ausd) = init(None);
    let deposit_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
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
    assert_eq!(
        master_unstaked_art_balance,
        (to_yocto(INIT_ART_BALANCE) / 2).to_string()
    );
    let master_staked_art_balance: String =
        view!(art.get_staked_balance(master_account.account_id().try_into().unwrap()))
            .unwrap_json();
    assert_eq!(
        master_staked_art_balance,
        (to_yocto(INIT_ART_BALANCE) / 2).to_string()
    );
}

#[test]
fn test_transfer_art() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("10000").to_string())
    )
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
    assert_eq!(
        master_staked_art_balance,
        (to_yocto(INIT_ART_BALANCE) / 2).to_string()
    );

    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, to_yocto("10000").to_string());

    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_staked_art_balance, (to_yocto("0")).to_string());

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(
        alice,
        art.transfer(bob.account_id(), to_yocto("1000").to_string())
    )
    .assert_success();

    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, to_yocto("9000").to_string());

    let bob_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(bob_unstaked_art_balance, to_yocto("1000").to_string());
}

#[test]
fn test_mint_ausd() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("10000").to_string())
    )
    .assert_success();

    let res = call!(alice, art.stake_and_mint(to_yocto("10000").to_string()));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(
        total_supply.0,
        ((to_yocto(INIT_ART_BALANCE) / 2 + to_yocto("10000")) * 20 / 5)
    );

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
    assert!(!call!(
        alice,
        art.transfer(bob.account_id(), to_yocto("1000").to_string())
    )
    .is_ok());

    // Alice can use her ausd freely
    call!(
        alice,
        ausd.transfer(bob.account_id(), U128(to_yocto("30000"))),
        deposit = STORAGE_AMOUNT / 10
    )
    .assert_success();
    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(
        U128(to_yocto("10000") * 20 / 5 - to_yocto("30000")),
        alice_ausd_balance
    );
    let bob_ausd_balance: U128 =
        view!(ausd.get_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("30000")), bob_ausd_balance);
}

#[test]
fn test_burn_unstake() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("10000").to_string())
    )
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
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("10000").to_string())
    )
    .assert_success();

    call!(alice, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    println!("{:?}", alice_ausd_balance);

    // price of art rise, ausd/art falls

    call!(
        master_account,
        art.submit_price("4000000000".to_string()), // every art is now $40
        gas = DEFAULT_GAS
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
        gas = DEFAULT_GAS
    )
    .assert_success();

    // now bob stake and mint
    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(bob.account_id(), to_yocto("10000").to_string())
    )
    .assert_success();
    call!(bob, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();
    let bob_ausd_balance: U128 =
        view!(ausd.get_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), bob_ausd_balance);
}

#[test]
fn test_exchange_ausd_abtc() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(
        master_account,
        art.submit_asset_price("aBTC".to_string(), "3000000000000".to_string())
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("10000").to_string())
    )
    .assert_success();

    call!(alice, art.stake_and_mint(to_yocto("10000").to_string())).assert_success();
    call!(
        alice,
        art.buy_asset_with_ausd("aBTC".to_string(), to_yocto("1").to_string())
    )
    .assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000")), alice_ausd_balance);
    let alice_abtc_balance: String =
        view!(art.get_asset_balance(alice.account_id().try_into().unwrap(), "aBTC".to_string()))
            .unwrap_json();
    assert_eq!(alice_abtc_balance, to_yocto("1").to_string());

    call!(
        master_account,
        art.submit_asset_price("aBTC".to_string(), "6000000000000".to_string())
    )
    .assert_success();

    call!(
        alice,
        art.sell_asset_to_ausd("aBTC".to_string(), to_yocto("1").to_string())
    )
    .assert_success();

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("70000")), alice_ausd_balance);
    let alice_abtc_balance: String =
        view!(art.get_asset_balance(alice.account_id().try_into().unwrap(), "aBTC".to_string()))
            .unwrap_json();
    assert_eq!(alice_abtc_balance, to_yocto("0").to_string());
}

#[test]
fn test_buy_ausd_with_near() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(
        master_account,
        art.submit_asset_price("aNEAR".to_string(), "500000000".to_string())
    ) // 1 NEAR = 5 ausd
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("101"));
    call!(alice, art.buy_ausd_with_near(), deposit = to_yocto("100")).assert_success();
    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("500")), alice_ausd_balance);
}

#[test]
fn test_buy_art_with_near() {
    let (master_account, art, ausd) = init(None);
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(
        master_account,
        art.submit_asset_price("aNEAR".to_string(), "500000000".to_string())
    ) // 1 NEAR = 5 ausd
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("101"));
    call!(alice, art.buy_art_with_near(), deposit = to_yocto("100")).assert_success();

    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, (to_yocto("25")).to_string());
}

use std::str::FromStr;

#[test]
fn test_staking_reward() {
    let mut genesis = GenesisConfig::default();
    genesis.block_time = 3600 * 1000000000;
    let (master_account, art, ausd) = init(Some(genesis));
    let stake_amount = (to_yocto(INIT_ART_BALANCE) / 2).to_string();
    call!(
        master_account,
        art.submit_price("2000000000".to_string()), // every art is $20
        gas = DEFAULT_GAS
    )
    .assert_success();
    call!(master_account, art.stake_and_mint(stake_amount)).assert_success();

    let alice = master_account.create_user("alice".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(alice.account_id(), to_yocto("30000").to_string())
    )
    .assert_success();

    let res = call!(alice, art.stake_and_mint(to_yocto("10000").to_string()));
    assert!(res.is_ok());
    let total_supply: U128 = view!(ausd.get_total_supply()).unwrap_json();
    assert_eq!(
        total_supply.0,
        ((to_yocto(INIT_ART_BALANCE) / 2 + to_yocto("10000")) * 20 / 5)
    );

    let alice_ausd_balance: U128 =
        view!(ausd.get_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(U128(to_yocto("10000") * 20 / 5), alice_ausd_balance);
    let alice_unstaked_art_balance: String =
        view!(art.get_unstaked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_unstaked_art_balance, (to_yocto("20000")).to_string());
    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    assert_eq!(alice_staked_art_balance, (to_yocto("10000")).to_string());

    master_account
        .borrow_runtime_mut()
        .produce_blocks(24)
        .unwrap();
    let reward_paid_at: u64 =
        view!(art.get_reward_paid_at(alice.account_id().try_into().unwrap())).unwrap_json();

    let a = call!(alice, art.refresh_reward());
    println!("{:?}", a);
    a.assert_success();
    let alice_staked_art_balance: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    let alice_staked_art_balance = u128::from_str(&alice_staked_art_balance).unwrap();
    assert!(alice_staked_art_balance > (to_yocto("10000")));
    let reward_paid_at2: u64 =
        view!(art.get_reward_paid_at(alice.account_id().try_into().unwrap())).unwrap_json();
    assert!(reward_paid_at2 > reward_paid_at);

    let a = call!(alice, art.refresh_reward());
    println!("{:?}", a);
    a.assert_success();
    let alice_staked_art_balance2: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    let alice_staked_art_balance2 = u128::from_str(&alice_staked_art_balance2).unwrap();
    assert_eq!(alice_staked_art_balance, alice_staked_art_balance2);
    let reward_paid_at3: u64 =
        view!(art.get_reward_paid_at(alice.account_id().try_into().unwrap())).unwrap_json();
    assert!(reward_paid_at3 == reward_paid_at2);

    master_account
        .borrow_runtime_mut()
        .produce_blocks(29 * 24)
        .unwrap();
    let alice_staked_art_balance3: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    let alice_staked_art_balance3 = u128::from_str(&alice_staked_art_balance3).unwrap();
    // should not change before refresh
    assert_eq!(alice_staked_art_balance3, alice_staked_art_balance2);

    let res = call!(alice, art.stake_and_mint(to_yocto("10000").to_string()));
    println!("=== {:?} {:?}", res, res.promise_results());
    assert!(res.is_ok());

    let alice_staked_art_balance4: String =
        view!(art.get_staked_balance(alice.account_id().try_into().unwrap())).unwrap_json();
    let alice_staked_art_balance4 = u128::from_str(&alice_staked_art_balance4).unwrap();
    // should be more than 10000 + alice_staked_art_balance3 because staking also refresh the reward
    assert!(alice_staked_art_balance4 > to_yocto("10000") + alice_staked_art_balance3);

    let reward_paid_at4: u64 =
        view!(art.get_reward_paid_at(alice.account_id().try_into().unwrap())).unwrap_json();
    // new stake also update the reward paid at, so you have to wait another 24 hours since your last stake, not your last time received stake reward, to get new stake reward
    assert!(reward_paid_at4 > reward_paid_at3 + 29 * 86400 * 1_000_000_000);
    assert!(reward_paid_at4 < reward_paid_at3 + 30 * 86400 * 1_000_000_000);

    let r = call!(alice, art.burn_to_unstake(to_yocto("10000").to_string()));
    println!("{:?}", r);
    r.assert_success();
    let reward_paid_at5: u64 =
        view!(art.get_reward_paid_at(alice.account_id().try_into().unwrap())).unwrap_json();
    assert!(reward_paid_at5 == reward_paid_at4);

    let bob = master_account.create_user("bob".to_string(), to_yocto("10"));
    call!(
        master_account,
        art.transfer(bob.account_id(), to_yocto("10000").to_string())
    )
    .assert_success();

    // stake 3 month, 15 days, check reward is correct
    let res = call!(bob, art.stake_and_mint(to_yocto("10000").to_string()));
    res.assert_success();
    master_account
        .borrow_runtime_mut()
        .produce_blocks((3 * 30 + 15) * 24)
        .unwrap();
    call!(bob, art.refresh_reward());
    let bob_staked_art_balance: String =
        view!(art.get_staked_balance(bob.account_id().try_into().unwrap())).unwrap_json();
    let bob_staked_art_balance = u128::from_str(&bob_staked_art_balance).unwrap();

    assert_eq!(10277963087960000000000000000u128, bob_staked_art_balance);
}
