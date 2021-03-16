use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Balance, Promise};
use num_rational::Ratio;
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Account {
    /// Current unstaked balance.
    pub balance: Balance,
    /// Allowed account to the allowance amount.
    pub allowances: HashMap<AccountId, Balance>,
    /// Allowed account to staked balance.
    pub staked_balances: HashMap<AccountId, Balance>,
}

impl Account {
    pub fn set_allowance(&mut self, escrow_account_id: &AccountId, allowance: Balance) {
        if allowance > 0 {
            self.allowances.insert(escrow_account_id.clone(), allowance);
        } else {
            self.allowances.remove(escrow_account_id);
        }
    }

    pub fn get_allowance(&self, escrow_account_id: &AccountId) -> Balance {
        *self.allowances.get(escrow_account_id).unwrap_or(&0)
    }

    pub fn set_staked_balance(&mut self, escrow_account_id: &AccountId, staked_balance: Balance) {
        if staked_balance > 0 {
            self.staked_balances.insert(escrow_account_id.clone(), staked_balance);
        } else {
            self.staked_balances.remove(escrow_account_id);
        }
    }

    pub fn get_staked_balance(&self, escrow_account_id: &AccountId) -> Balance {
        *self.staked_balances.get(escrow_account_id).unwrap_or(&0)
    }

    pub fn total_balance(&self) -> Balance {
        self.balance + self.staked_balances.values().sum::<Balance>()
    }
}

#[ext_contract(ext_usd)]
pub trait ExtAUSDContract {
    fn mint(&mut self, amount: u128) -> u128;
    fn burn_to_unstake(&mut self, burn_amount: u128, unstake_amount: u128) -> Promise;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Art {
    /// AccountID -> Account details.
    pub accounts: UnorderedMap<AccountId, Account>,

    /// Total supply of the all token, in yocto
    pub total_supply: Balance,

    /// Current price, each 10^8 art in USD
    pub price: u128,

    /// Owner ID
    pub owner: AccountId,

    /// USD token, only allow unstake originate from which
    pub ausd_token: AccountId,

    /// Total staked balance
    pub total_staked: Balance,
}

impl Default for Art {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

#[near_bindgen]
impl Art {
    #[init]
    pub fn new(owner_id: AccountId, total_supply: String, ausd_token: AccountId) -> Self {
        let total_supply = u128::from_str(&total_supply).expect("Failed to parse total supply");
        let mut ft = Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            total_supply,
            price: 0,
            owner: owner_id.clone(),
            ausd_token,
            total_staked: 0,
        };
        let mut account = ft.get_account(&owner_id);
        account.balance = total_supply;
        ft.accounts.insert(&owner_id, &account);
        ft
    }

    /// Sets amount allowed to spent by `escrow_account_id` on behalf of the caller of the function
    /// (`predecessor_id`) who is considered the balance owner to the new `allowance`.
    /// If some amount of tokens is currently staked by the `escrow_account_id` the new allowance is
    /// decreased by the amount of staked tokens.
    pub fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: String) {
        let allowance = u128::from_str(&allowance).expect("Failed to parse allowance");
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);
        let staked_balance = account.get_staked_balance(&escrow_account_id);
        if staked_balance > allowance {
            env::panic(b"The new allowance can't be less than the amount of staked tokens");
        }

        account.set_allowance(&escrow_account_id, allowance - staked_balance);
        self.accounts.insert(&owner_id, &account);
    }

    pub fn submit_price(&mut self, price: String) {
        if env::predecessor_account_id() != self.owner {
            env::panic(b"Only owner can submit price data");
        }
        let price = u128::from_str(&price).expect("Failed to parse price");
        // we completely trust owner for now
        self.price = price;
    }

    pub fn stake_and_mint(&mut self, owner_id: AccountId, stake: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let stake_amount = self.stake(owner_id.clone(), stake);
        let unit_price = Ratio::new(self.price, 100_000_000);
        let mint_amount = Ratio::new(stake_amount, 5) * unit_price;
        let mint_amount = mint_amount.to_integer();
        ext_usd::mint(mint_amount, &self.ausd_token, 0, env::prepaid_gas() / 2)
    }

    pub fn burn_to_unstake(&mut self, owner_id: AccountId, unstake_amount: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let unstake_amount =
            u128::from_str(&unstake_amount).expect("Failed to parse unstake_amount");

        let unit_price = Ratio::new(self.price, 100_000_000);
        let burn_amount = Ratio::new(unstake_amount, 5) * unit_price;
        let burn_amount = burn_amount.to_integer();
        if self.price == 4000000000 {
            log!("burn_amount {}", burn_amount);
        }
        ext_usd::burn_to_unstake(
            burn_amount,
            unstake_amount,
            &self.ausd_token,
            0,
            env::prepaid_gas() / 3,
        )
    }

    /// Stakes an additional `stake_amount` to the caller of the function (`predecessor_id`) from
    /// the `owner_id`.
    /// Requirements:
    /// * The (`predecessor_id`) should have enough allowance or be the owner.
    /// * The owner should have enough unstaked balance.
    fn stake(&mut self, owner_id: AccountId, stake_amount: String) -> u128 {
        let stake_amount = u128::from_str(&stake_amount).expect("Failed to parse stake_amount");
        if stake_amount == 0 {
            env::panic(b"Can't stake 0 tokens");
        }
        let escrow_account_id = env::predecessor_account_id();
        let mut account = self.get_account(&owner_id);

        // Checking and updating unstaked balance
        if account.balance < stake_amount {
            env::panic(b"Not enough unstaked balance");
        }
        account.balance -= stake_amount;

        // If staking by escrow, need to check and update the allowance.
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            if allowance < stake_amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - stake_amount);
        }

        // Updating total stake balance
        let staked_balance = account.get_staked_balance(&escrow_account_id);
        account.set_staked_balance(&escrow_account_id, staked_balance + stake_amount);
        self.total_staked += stake_amount;

        self.accounts.insert(&owner_id, &account);
        stake_amount
    }

    /// Unstakes the `unstake_amount` from the owner
    pub fn unstake(&mut self, owner_id: AccountId, unstake_amount: u128) {
        assert!(
            env::predecessor_account_id() == self.ausd_token,
            "Only allow unstake originated from ausd token"
        );
        if unstake_amount == 0 {
            env::panic(b"Can't unstake 0 tokens");
        }
        let mut account = self.get_account(&owner_id);

        // Checking and updating staked balance
        let staked_balance = account.get_staked_balance(&owner_id);
        if staked_balance < unstake_amount {
            env::panic(b"Not enough staked tokens");
        }
        account.set_staked_balance(&owner_id, staked_balance - unstake_amount);

        // Updating unstaked balance
        account.balance += unstake_amount;
        self.total_staked -= unstake_amount;
        log!("{} {}", unstake_amount, staked_balance - unstake_amount);
        self.accounts.insert(&owner_id, &account);
    }

    /// Transfers unstaked `amount` of tokens from `owner_id` to the `new_owner_id`.
    /// Requirements:
    /// * The caller of the function (`predecessor_id`) should have at least `amount` of allowance tokens.
    /// * The balance owner should have at least `amount` of unstaked (by `predecessor_id`) tokens
    pub fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: String) {
        let amount = u128::from_str(&amount).expect("Failed to parse allow amount");
        if amount == 0 {
            env::panic(b"Can't transfer 0 tokens");
        }
        let escrow_account_id = env::predecessor_account_id();
        let mut account = self.get_account(&owner_id);

        // Checking and updating unstaked balance
        if account.balance < amount {
            env::panic(b"Not enough unstaked balance");
        }
        account.balance -= amount;

        // If transferring by escrow, need to check and update allowance.
        if escrow_account_id != owner_id {
            let allowance = account.get_allowance(&escrow_account_id);
            // Checking and updating unstaked balance
            if allowance < amount {
                env::panic(b"Not enough allowance");
            }
            account.set_allowance(&escrow_account_id, allowance - amount);
        }

        self.accounts.insert(&owner_id, &account);

        // Stake amount to the new owner
        let mut new_account = self.get_account(&new_owner_id);
        new_account.balance += amount;
        self.accounts.insert(&new_owner_id, &new_account);
    }

    /// Same as `transfer_from` with `owner_id` `predecessor_id`.
    pub fn transfer(&mut self, new_owner_id: AccountId, amount: String) {
        self.transfer_from(env::predecessor_account_id(), new_owner_id, amount);
    }

    /// Returns total supply of tokens.
    pub fn get_total_supply(&self) -> String {
        self.total_supply.to_string()
    }

    /// Returns total balance for the `owner_id` account. Including all staked and unstaked tokens.
    pub fn get_total_balance(&self, owner_id: AccountId) -> String {
        self.get_account(&owner_id).total_balance().to_string()
    }

    /// Returns unstaked token balance for the `owner_id`.
    pub fn get_unstaked_balance(&self, owner_id: AccountId) -> String {
        self.get_account(&owner_id).balance.to_string()
    }

    /// Returns current allowance for the `owner_id` to be able to use by `escrow_account_id`.
    pub fn get_allowance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> String {
        self.get_account(&owner_id).get_allowance(&escrow_account_id).to_string()
    }

    /// Returns current staked balance for the `owner_id` staked by `escrow_account_id`.
    pub fn get_staked_balance(&self, owner_id: AccountId, escrow_account_id: AccountId) -> String {
        self.get_account(&owner_id).get_staked_balance(&escrow_account_id).to_string()
    }
}

impl Art {
    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId) -> Account {
        self.accounts.get(owner_id).unwrap_or_default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    use super::*;

    fn alice() -> AccountId {
        "alice.near".to_string()
    }
    fn bob() -> AccountId {
        "bob.near".to_string()
    }
    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id,
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 10u64.pow(6),
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_new() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let contract = Art::new(bob(), total_supply.to_string(), "ausd".to_string());
        assert_eq!(contract.get_total_supply(), total_supply.to_string());
        assert_eq!(contract.get_unstaked_balance(bob()), total_supply.to_string());
        assert_eq!(contract.get_total_balance(bob()), total_supply.to_string());
    }

    #[test]
    fn test_transfer() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        let transfer_amount = total_supply / 3;
        contract.transfer(bob(), transfer_amount.to_string());
        assert_eq!(
            contract.get_unstaked_balance(carol()),
            (total_supply - transfer_amount).to_string()
        );
        assert_eq!(contract.get_unstaked_balance(bob()), transfer_amount.to_string());
    }

    #[test]
    fn test_stake_fail() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        let transfer_amount = total_supply / 3;
        std::panic::catch_unwind(move || {
            contract.stake(bob(), transfer_amount.to_string());
        })
        .unwrap_err();
    }

    #[test]
    fn test_self_allowance_fail() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        std::panic::catch_unwind(move || {
            contract.set_allowance(carol(), format!("{}", total_supply / 2));
        })
        .unwrap_err();
    }

    //    #[test]
    //    fn test_stake_and_unstake_owner() {
    //        let context = get_context(carol());
    //        testing_env!(context);
    //        let total_supply = 1_000_000_000_000_000u128;
    //        let mut contract = FunToken::new(carol(), total_supply.to_string(), "ausd".to_string());
    //        assert_eq!(contract.get_total_supply(), total_supply.to_string());
    //        let stake_amount = total_supply / 3;
    //        contract.stake(carol(), stake_amount.to_string());
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            (total_supply - stake_amount).to_string()
    //        );
    //        assert_eq!(
    //            contract.get_total_balance(carol()),
    //            total_supply.to_string()
    //        );
    //        contract.unstake(carol(), stake_amount);
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            total_supply.to_string()
    //        );
    //        assert_eq!(
    //            contract.get_total_balance(carol()),
    //            total_supply.to_string()
    //        );
    //    }

    //    #[test]
    //    fn test_stake_and_transfer() {
    //        let context = get_context(carol());
    //        testing_env!(context);
    //        let total_supply = 1_000_000_000_000_000u128;
    //        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
    //        assert_eq!(contract.get_total_supply(), total_supply.to_string());
    //        let stake_amount = total_supply / 3;
    //        let transfer_amount = stake_amount / 3;
    //        // Staking
    //        contract.stake(carol(), stake_amount.to_string());
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            (total_supply - stake_amount).to_string()
    //        );
    //        assert_eq!(contract.get_total_balance(carol()), total_supply.to_string());
    //        for i in 1..=5 {
    //            // Transfer to bob
    //            contract.transfer(bob(), transfer_amount.to_string());
    //            assert_eq!(
    //                contract.get_unstaked_balance(carol()),
    //                format!(
    //                    "{}",
    //                    std::cmp::min(total_supply - stake_amount, total_supply - transfer_amount * i)
    //                )
    //            );
    //            assert_eq!(
    //                contract.get_total_balance(carol()),
    //                format!("{}", total_supply - transfer_amount * i)
    //            );
    //            assert_eq!(contract.get_unstaked_balance(bob()), format!("{}", transfer_amount * i));
    //        }
    //    }

    //    #[test]
    //    fn test_carol_escrows_to_bob_transfers_to_alice() {
    //        // Acting as carol
    //        testing_env!(get_context(carol()));
    //        let total_supply = 1_000_000_000_000_000u128;
    //        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
    //        assert_eq!(contract.get_total_supply(), total_supply.to_string());
    //        let allowance = total_supply / 3;
    //        let transfer_amount = allowance / 3;
    //        contract.set_allowance(bob(), format!("{}", allowance));
    //        assert_eq!(contract.get_allowance(carol(), bob()), format!("{}", allowance));
    //        // Acting as bob now
    //        testing_env!(get_context(bob()));
    //        contract.transfer_from(carol(), alice(), transfer_amount.to_string());
    //        assert_eq!(
    //            contract.get_total_balance(carol()),
    //            (total_supply - transfer_amount).to_string()
    //        );
    //        assert_eq!(contract.get_unstaked_balance(alice()), transfer_amount.to_string());
    //        assert_eq!(
    //            contract.get_allowance(carol(), bob()),
    //            format!("{}", allowance - transfer_amount)
    //        );
    //    }
    //
    //    #[test]
    //    fn test_carol_escrows_to_bob_stakes_and_transfers_to_alice() {
    //        // Acting as carol
    //        testing_env!(get_context(carol()));
    //        let total_supply = 1_000_000_000_000_000u128;
    //        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
    //        assert_eq!(contract.get_total_supply(), total_supply.to_string());
    //        let allowance = total_supply / 3;
    //        let transfer_amount = allowance / 3;
    //        let stake_amount = transfer_amount;
    //        contract.set_allowance(bob(), format!("{}", allowance));
    //        assert_eq!(contract.get_allowance(carol(), bob()), format!("{}", allowance));
    //        // Acting as bob now
    //        testing_env!(get_context(bob()));
    //        contract.stake(carol(), stake_amount.to_string());
    //        assert_eq!(contract.get_allowance(carol(), bob()), (allowance - stake_amount).to_string());
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            (total_supply - stake_amount).to_string()
    //        );
    //        assert_eq!(contract.get_total_balance(carol()), total_supply.to_string());
    //        contract.transfer_from(carol(), alice(), transfer_amount.to_string());
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            (total_supply - transfer_amount).to_string()
    //        );
    //        assert_eq!(contract.get_unstaked_balance(alice()), transfer_amount.to_string());
    //        assert_eq!(
    //            contract.get_allowance(carol(), bob()),
    //            format!("{}", allowance - transfer_amount)
    //        );
    //    }

    //    #[test]
    //    fn test_stake_and_unstake_through_allowance() {
    //        // Acting as carol
    //        testing_env!(get_context(carol()));
    //        let total_supply = 1_000_000_000_000_000u128;
    //        let mut contract = FunToken::new(carol(), total_supply.to_string(), "ausd".to_string());
    //        assert_eq!(contract.get_total_supply(), total_supply.to_string());
    //        let allowance = total_supply / 3;
    //        let stake_amount = allowance / 2;
    //        contract.set_allowance(bob(), format!("{}", allowance));
    //        assert_eq!(
    //            contract.get_allowance(carol(), bob()),
    //            format!("{}", allowance)
    //        );
    //        // Acting as bob now
    //        testing_env!(get_context(bob()));
    //        contract.stake(carol(), stake_amount.to_string());
    //        assert_eq!(
    //            contract.get_allowance(carol(), bob()),
    //            (allowance - stake_amount).to_string()
    //        );
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            (total_supply - stake_amount).to_string()
    //        );
    //        assert_eq!(
    //            contract.get_total_balance(carol()),
    //            total_supply.to_string()
    //        );
    //        contract.unstake(carol(), stake_amount);
    //        assert_eq!(
    //            contract.get_allowance(carol(), bob()),
    //            format!("{}", allowance)
    //        );
    //        assert_eq!(
    //            contract.get_unstaked_balance(carol()),
    //            total_supply.to_string()
    //        );
    //        assert_eq!(
    //            contract.get_total_balance(carol()),
    //            total_supply.to_string()
    //        );
    //    }

    #[test]
    fn test_set_allowance_during_stake() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        assert_eq!(contract.get_total_supply(), total_supply.to_string());
        let allowance = 2 * total_supply / 3;
        let stake_amount = allowance / 2;
        contract.set_allowance(bob(), allowance.to_string());
        assert_eq!(contract.get_allowance(carol(), bob()), allowance.to_string());
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.stake(carol(), stake_amount.to_string());
        assert_eq!(contract.get_allowance(carol(), bob()), (allowance - stake_amount).to_string());
        assert_eq!(
            contract.get_unstaked_balance(carol()),
            (total_supply - stake_amount).to_string()
        );
        assert_eq!(contract.get_total_balance(carol()), total_supply.to_string());
        // Acting as carol now
        testing_env!(get_context(carol()));
        contract.set_allowance(bob(), allowance.to_string());
        assert_eq!(contract.get_allowance(carol(), bob()), (allowance - stake_amount).to_string());
    }

    #[test]
    fn test_competing_stakes() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        assert_eq!(contract.get_total_supply(), total_supply.to_string());
        let allowance = 2 * total_supply / 3;
        let stake_amount = allowance;
        contract.set_allowance(bob(), allowance.to_string());
        contract.set_allowance(alice(), allowance.to_string());
        assert_eq!(contract.get_allowance(carol(), bob()), allowance.to_string());
        assert_eq!(contract.get_allowance(carol(), alice()), allowance.to_string());
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.stake(carol(), stake_amount.to_string());
        assert_eq!(contract.get_allowance(carol(), bob()), (allowance - stake_amount).to_string());
        assert_eq!(
            contract.get_unstaked_balance(carol()),
            (total_supply - stake_amount).to_string()
        );
        assert_eq!(contract.get_total_balance(carol()), total_supply.to_string());
        // Acting as alice now
        testing_env!(get_context(alice()));
        std::panic::catch_unwind(move || {
            contract.stake(carol(), stake_amount.to_string());
        })
        .unwrap_err();
    }
}
