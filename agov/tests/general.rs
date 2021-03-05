use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, UserAccount, DEFAULT_GAS,
    STORAGE_AMOUNT,
};

extern crate agov;
use agov::AGovContract;

extern crate ausd;
use ausd::AUSDContract;

near_sdk_sim::lazy_static! {
    static ref AGOV_WASM_BYTES: &'static [u8] = include_bytes!("../res/agov.wasm").as_ref();
    static ref AUSD_WASM_BYTES: &'static [u8] = include_bytes!("../../ausd/res/ausd.wasm").as_ref();
}

fn init(
    initial_balance: u128,
) -> (UserAccount, ContractAccount<AGovContract>, UserAccount) {
    let master_account = init_simulator(None);
    let agov = deploy! {
        contract: AGovContract,
        contract_id: "agov",
        bytes: &AGOV_WASM_BYTES,
        signer_account: master_account
    };

    let ausd = deploy! {
        contract: AUSDContract,
        contract_id: "ausd",
        bytes: &AUSD_WASM_BYTES,
        signer_account: master_account
    };

    let alice = master_account.create_user("alice".to_string(), initial_balance);
    (master_account, agov, alice)
}
