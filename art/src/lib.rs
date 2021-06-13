use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::collections::UnorderedMap;
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::{ValidAccountId, U128},
    Gas, StorageUsage,
};
use near_sdk::{env, ext_contract, log, near_bindgen, AccountId, Balance, Promise, PromiseOrValue};
use num_bigint::BigInt;
use num_rational::Ratio;
use num_traits::cast::ToPrimitive;
use std::collections::HashMap;
use std::str::FromStr;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: near_sdk::wee_alloc::WeeAlloc<'_> = near_sdk::wee_alloc::WeeAlloc::INIT;

pub const DAY_INTEREST: [Ratio<u128>; 30] = [
    Ratio::new_raw(1000000, 1000000),
    Ratio::new_raw(1000261, 1000000),
    Ratio::new_raw(1000522, 1000000),
    Ratio::new_raw(1000783, 1000000),
    Ratio::new_raw(1001045, 1000000),
    Ratio::new_raw(1001306, 1000000),
    Ratio::new_raw(1001567, 1000000),
    Ratio::new_raw(1001829, 1000000),
    Ratio::new_raw(1002091, 1000000),
    Ratio::new_raw(1002352, 1000000),
    Ratio::new_raw(1002614, 1000000),
    Ratio::new_raw(1002876, 1000000),
    Ratio::new_raw(1003138, 1000000),
    Ratio::new_raw(1003400, 1000000),
    Ratio::new_raw(1003662, 1000000),
    Ratio::new_raw(1003924, 1000000),
    Ratio::new_raw(1004186, 1000000),
    Ratio::new_raw(1004448, 1000000),
    Ratio::new_raw(1004711, 1000000),
    Ratio::new_raw(1004973, 1000000),
    Ratio::new_raw(1005236, 1000000),
    Ratio::new_raw(1005498, 1000000),
    Ratio::new_raw(1005761, 1000000),
    Ratio::new_raw(1006023, 1000000),
    Ratio::new_raw(1006286, 1000000),
    Ratio::new_raw(1006549, 1000000),
    Ratio::new_raw(1006812, 1000000),
    Ratio::new_raw(1007075, 1000000),
    Ratio::new_raw(1007338, 1000000),
    Ratio::new_raw(1007601, 1000000),
];
const MONTH_INTEREST: [Ratio<u128>; 12] = [
    Ratio::new_raw(1000000, 1000000),
    Ratio::new_raw(1007864, 1000000),
    Ratio::new_raw(1015790, 1000000),
    Ratio::new_raw(1023779, 1000000),
    Ratio::new_raw(1031830, 1000000),
    Ratio::new_raw(1039945, 1000000),
    Ratio::new_raw(1048124, 1000000),
    Ratio::new_raw(1056367, 1000000),
    Ratio::new_raw(1064675, 1000000),
    Ratio::new_raw(1073048, 1000000),
    Ratio::new_raw(1081487, 1000000),
    Ratio::new_raw(1089992, 1000000),
];
const YEAR_INTEREST: [Ratio<u128>; 100] = [
    Ratio::new_raw(1000000, 1000000),
    Ratio::new_raw(1100000, 1000000),
    Ratio::new_raw(1210000, 1000000),
    Ratio::new_raw(1331000, 1000000),
    Ratio::new_raw(1464100, 1000000),
    Ratio::new_raw(1610510, 1000000),
    Ratio::new_raw(1771561, 1000000),
    Ratio::new_raw(1948717, 1000000),
    Ratio::new_raw(2143588, 1000000),
    Ratio::new_raw(2357947, 1000000),
    Ratio::new_raw(2593742, 1000000),
    Ratio::new_raw(2853116, 1000000),
    Ratio::new_raw(3138428, 1000000),
    Ratio::new_raw(3452271, 1000000),
    Ratio::new_raw(3797498, 1000000),
    Ratio::new_raw(4177248, 1000000),
    Ratio::new_raw(4594972, 1000000),
    Ratio::new_raw(5054470, 1000000),
    Ratio::new_raw(5559917, 1000000),
    Ratio::new_raw(6115909, 1000000),
    Ratio::new_raw(6727499, 1000000),
    Ratio::new_raw(7400249, 1000000),
    Ratio::new_raw(8140274, 1000000),
    Ratio::new_raw(8954302, 1000000),
    Ratio::new_raw(9849732, 1000000),
    Ratio::new_raw(10834705, 1000000),
    Ratio::new_raw(11918176, 1000000),
    Ratio::new_raw(13109994, 1000000),
    Ratio::new_raw(14420993, 1000000),
    Ratio::new_raw(15863092, 1000000),
    Ratio::new_raw(17449402, 1000000),
    Ratio::new_raw(19194342, 1000000),
    Ratio::new_raw(21113776, 1000000),
    Ratio::new_raw(23225154, 1000000),
    Ratio::new_raw(25547669, 1000000),
    Ratio::new_raw(28102436, 1000000),
    Ratio::new_raw(30912680, 1000000),
    Ratio::new_raw(34003948, 1000000),
    Ratio::new_raw(37404343, 1000000),
    Ratio::new_raw(41144777, 1000000),
    Ratio::new_raw(45259255, 1000000),
    Ratio::new_raw(49785181, 1000000),
    Ratio::new_raw(54763699, 1000000),
    Ratio::new_raw(60240069, 1000000),
    Ratio::new_raw(66264076, 1000000),
    Ratio::new_raw(72890483, 1000000),
    Ratio::new_raw(80179532, 1000000),
    Ratio::new_raw(88197485, 1000000),
    Ratio::new_raw(97017233, 1000000),
    Ratio::new_raw(106718957, 1000000),
    Ratio::new_raw(117390852, 1000000),
    Ratio::new_raw(129129938, 1000000),
    Ratio::new_raw(142042931, 1000000),
    Ratio::new_raw(156247225, 1000000),
    Ratio::new_raw(171871947, 1000000),
    Ratio::new_raw(189059142, 1000000),
    Ratio::new_raw(207965056, 1000000),
    Ratio::new_raw(228761562, 1000000),
    Ratio::new_raw(251637718, 1000000),
    Ratio::new_raw(276801490, 1000000),
    Ratio::new_raw(304481639, 1000000),
    Ratio::new_raw(334929803, 1000000),
    Ratio::new_raw(368422783, 1000000),
    Ratio::new_raw(405265062, 1000000),
    Ratio::new_raw(445791568, 1000000),
    Ratio::new_raw(490370725, 1000000),
    Ratio::new_raw(539407797, 1000000),
    Ratio::new_raw(593348577, 1000000),
    Ratio::new_raw(652683435, 1000000),
    Ratio::new_raw(717951778, 1000000),
    Ratio::new_raw(789746956, 1000000),
    Ratio::new_raw(868721652, 1000000),
    Ratio::new_raw(955593817, 1000000),
    Ratio::new_raw(1051153199, 1000000),
    Ratio::new_raw(1156268519, 1000000),
    Ratio::new_raw(1271895371, 1000000),
    Ratio::new_raw(1399084908, 1000000),
    Ratio::new_raw(1538993399, 1000000),
    Ratio::new_raw(1692892739, 1000000),
    Ratio::new_raw(1862182013, 1000000),
    Ratio::new_raw(2048400214, 1000000),
    Ratio::new_raw(2253240236, 1000000),
    Ratio::new_raw(2478564259, 1000000),
    Ratio::new_raw(2726420685, 1000000),
    Ratio::new_raw(2999062754, 1000000),
    Ratio::new_raw(3298969029, 1000000),
    Ratio::new_raw(3628865932, 1000000),
    Ratio::new_raw(3991752525, 1000000),
    Ratio::new_raw(4390927778, 1000000),
    Ratio::new_raw(4830020556, 1000000),
    Ratio::new_raw(5313022611, 1000000),
    Ratio::new_raw(5844324873, 1000000),
    Ratio::new_raw(6428757360, 1000000),
    Ratio::new_raw(7071633096, 1000000),
    Ratio::new_raw(7778796406, 1000000),
    Ratio::new_raw(8556676046, 1000000),
    Ratio::new_raw(9412343651, 1000000),
    Ratio::new_raw(10353578016, 1000000),
    Ratio::new_raw(11388935818, 1000000),
    Ratio::new_raw(12527829399, 1000000),
];

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Account {
    /// Current unstaked balance.
    pub balance: Balance,
    /// Allowed account to the allowance amount.
    pub allowances: HashMap<AccountId, Balance>,
    /// Allowed account to staked balance.
    pub staked_balance: Balance,
    pub assets: HashMap<String, Balance>,
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

    pub fn set_staked_balance(&mut self, staked_balance: Balance) {
        self.staked_balance = staked_balance;
    }

    pub fn get_staked_balance(&self) -> Balance {
        self.staked_balance
    }

    pub fn total_balance(&self) -> Balance {
        self.balance + self.staked_balance
    }
}

#[ext_contract(ext_usd)]
pub trait ExtAUSDContract {
    fn mint(&mut self, account_id: String, amount: u128) -> u128;
    fn burn_to_unstake(
        &mut self,
        account_id: String,
        burn_amount: u128,
        unstake_amount: u128,
    ) -> Promise;
    fn burn_to_buy_asset(
        &mut self,
        account_id: String,
        burn_amount: u128,
        asset: String,
        asset_amount: u128,
    ) -> Promise;
    fn buy_ausd(&mut self, new_owner_id: AccountId, amount: U128);
    fn sell_ausd(&mut self, seller_id: AccountId, amount: U128);
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

    // Asset Prices
    pub asset_prices: UnorderedMap<String, u128>,

    /// Stake rewards paid at per account
    pub reward_paid_at: UnorderedMap<AccountId, u64>,

    /// Staking reward enabled at timestamp, when this version of contract deployed
    pub staking_reward_enabled_at: u64,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,
}

impl Default for Art {
    fn default() -> Self {
        panic!("Fun token should be initialized before usage")
    }
}

#[near_bindgen]
impl Art {
    #[init(ignore_state)]
    pub fn new(owner_id: AccountId, total_supply: String, ausd_token: AccountId) -> Self {
        let total_supply = u128::from_str(&total_supply).expect("Failed to parse total supply");
        let mut ft = Self {
            accounts: UnorderedMap::new(b"a".to_vec()),
            asset_prices: UnorderedMap::new(b"b".to_vec()),
            total_supply,
            price: 0,
            owner: owner_id.clone(),
            ausd_token,
            total_staked: 0,
            reward_paid_at: UnorderedMap::new(b"d".to_vec()),
            staking_reward_enabled_at: env::block_timestamp(),
            account_storage_usage: 0,
        };
        let mut account = ft.get_account(&owner_id);
        account.balance = total_supply;
        ft.accounts.insert(&owner_id, &account);
        ft.measure_account_storage_usage();
        ft
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = "a".repeat(64);
        self.accounts.insert(&tmp_account_id, &Default::default());
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        self.accounts.remove(&tmp_account_id);
    }

    pub fn refresh_reward(&mut self) -> bool {
        log!("refresh_reward");
        let account_id = env::predecessor_account_id();
        let staked = self.get_staked_balance(account_id.clone());
        let staked = u128::from_str(&staked).unwrap();
        let mut account = self.get_account(&account_id);

        if staked == 0 {
            env::log(b"no token was staked");
            return false;
        }
        let mut reward_paid_at = self
            .reward_paid_at
            .get(&account_id)
            .unwrap_or(self.staking_reward_enabled_at);
        let now = env::block_timestamp();
        let mut new_staked = staked;
        let mut days = (now - reward_paid_at) / (24 * 60 * 60 * 1000000000);
        if days == 0 {
            env::log(b"not been a day since last reward_paid_at");
            return false;
        }
        reward_paid_at += days * (24 * 60 * 60 * 1000000000);
        if days > 365 {
            let mut r = Ratio::new(new_staked, 1);
            r *= YEAR_INTEREST[(days / 365) as usize];
            new_staked = r.to_integer();
            days %= 365;
        }
        if days > 30 {
            let mut r = Ratio::new(new_staked, 1);
            r *= MONTH_INTEREST[(days / 30) as usize];
            new_staked = r.to_integer();
            days %= 30;
        }
        if days > 0 {
            let mut r = Ratio::new(new_staked, 1);
            r *= DAY_INTEREST[days as usize];
            new_staked = r.to_integer();
        }
        log!("refresh_reward {} {}", account.staked_balance, new_staked);
        account.staked_balance = new_staked;
        self.total_supply += new_staked - staked;
        self.total_staked += new_staked - staked;
        self.accounts.insert(&account_id, &account);
        self.reward_paid_at.insert(&account_id, &reward_paid_at);
        return true;
    }

    #[payable]
    pub fn buy_art_with_near(&mut self) {
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic(b"Can't buy with 0 NEAR");
        }
        let account_id = env::predecessor_account_id();
        let mut account = self.get_account(&account_id);
        let near_price = self._get_asset_price(&"aNEAR".to_string());
        if near_price == 0 {
            env::panic(b"No NEAR price data from oracle");
        }
        let art_price = self.price;
        if art_price == 0 {
            env::panic(b"No price data from oracle");
        }
        let attached_deposit: BigInt = attached_deposit.into();
        let art_amount: Ratio<BigInt> =
            Ratio::<BigInt>::new(near_price.into(), art_price.into()) * attached_deposit;
        let art_amount = art_amount.to_integer().to_u128().unwrap();
        let mut owner = self.get_account(&self.owner);

        account.balance = account.balance.checked_add(art_amount).unwrap();
        owner.balance = owner.balance.checked_sub(art_amount).unwrap();

        self.accounts.insert(&self.owner, &owner);
        self.accounts.insert(&account_id, &account);
    }

    pub fn exchange_art_to_ausd(&mut self, amount: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let amount = u128::from_str(&amount).expect("Failed to parse amount");

        let unit_price = Ratio::<BigInt>::new(self.price.into(), 100_000_000.into());
        let amount_b: BigInt = amount.into();

        let ausd_amount = unit_price * amount_b * Ratio::<BigInt>::new(997.into(), 1000.into());
        let ausd_amount = ausd_amount.to_integer().to_u128().unwrap();

        let account_id = env::predecessor_account_id();
        let mut owner = self.get_account(&self.owner);
        let mut account = self.get_account(&account_id);
        account.balance = account.balance.checked_sub(amount).unwrap();
        owner.balance = owner.balance.checked_add(amount).unwrap();
        self.accounts.insert(&self.owner, &owner);
        self.accounts.insert(&account_id, &account);
        ext_usd::buy_ausd(
            account_id,
            U128(ausd_amount),
            &self.ausd_token,
            0,
            env::prepaid_gas() / 2,
        )
    }

    pub fn exchange_ausd_to_art(&mut self, ausd_amount: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let unit_price = Ratio::<BigInt>::new(self.price.into(), 100_000_000.into());
        let ausd_amount = u128::from_str(&ausd_amount).expect("Failed to parse ausd_amount");
        let ausd_amount_b: BigInt = ausd_amount.into();

        let amount = Ratio::<BigInt>::new(997.into(), 1000.into()) * ausd_amount_b / unit_price;
        let amount = amount.to_integer().to_u128().unwrap();

        let account_id = env::predecessor_account_id();
        let mut owner = self.get_account(&self.owner);
        let mut account = self.get_account(&account_id);
        account.balance = account.balance.checked_add(amount).unwrap();
        owner.balance = owner.balance.checked_sub(amount).unwrap();
        self.accounts.insert(&self.owner, &owner);
        self.accounts.insert(&account_id, &account);

        ext_usd::sell_ausd(
            account_id,
            U128(ausd_amount),
            &self.ausd_token,
            0,
            env::prepaid_gas() / 2,
        )
    }

    #[payable]
    pub fn buy_ausd_with_near(&mut self) -> Promise {
        let attached_deposit = env::attached_deposit();
        if attached_deposit == 0 {
            env::panic(b"Can't buy with 0 NEAR");
        }
        let account_id = env::predecessor_account_id();
        let near_price = self._get_asset_price(&"aNEAR".to_string());
        if near_price == 0 {
            env::panic(b"No NEAR price data from oracle");
        }
        let attached_deposit: BigInt = attached_deposit.into();
        let ausd_amount: Ratio<BigInt> =
            Ratio::<BigInt>::new(near_price.into(), 100000000.into()) * attached_deposit;
        let ausd_amount = ausd_amount.to_integer().to_u128().unwrap();
        ext_usd::buy_ausd(
            account_id,
            U128(ausd_amount),
            &self.ausd_token,
            0,
            env::prepaid_gas() / 2,
        )
    }

    pub fn sell_art_to_near(&mut self) {}

    pub fn sell_ausd_to_near(&mut self) {}

    /// Sets amount allowed to spent by `escrow_account_id` on behalf of the caller of the function
    /// (`predecessor_id`) who is considered the balance owner to the new `allowance`.
    pub fn set_allowance(&mut self, escrow_account_id: AccountId, allowance: String) {
        let allowance = u128::from_str(&allowance).expect("Failed to parse allowance");
        let owner_id = env::predecessor_account_id();
        if escrow_account_id == owner_id {
            env::panic(b"Can't set allowance for yourself");
        }
        let mut account = self.get_account(&owner_id);

        account.set_allowance(&escrow_account_id, allowance);
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

    pub fn submit_asset_price(&mut self, asset: String, price: String) {
        if env::predecessor_account_id() != self.owner {
            env::panic(b"Only owner can submit price data");
        }
        let price = u128::from_str(&price).expect("Failed to parse price");
        self.asset_prices.insert(&asset, &price);
    }

    pub fn stake_and_mint(&mut self, stake: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let stake_amount = self.stake(stake);

        let unit_price = Ratio::<BigInt>::new(self.price.into(), 100_000_000.into());

        let mint_amount = Ratio::<BigInt>::new(stake_amount.into(), 5.into()) * unit_price;
        let mint_amount = mint_amount.to_integer().to_u128().unwrap();

        let account_id = env::predecessor_account_id();
        ext_usd::mint(
            account_id,
            mint_amount,
            &self.ausd_token,
            0,
            env::prepaid_gas() / 2,
        )
    }

    pub fn burn_to_unstake(&mut self, unstake_amount: String) -> Promise {
        if self.price == 0 {
            // Not received any data from oracle
            env::panic(b"No price data from oracle");
        }
        let unstake_amount =
            u128::from_str(&unstake_amount).expect("Failed to parse unstake_amount");

        let account_id = env::predecessor_account_id();

        let unit_price = Ratio::<BigInt>::new(self.price.into(), 100_000_000.into());

        let burn_amount = Ratio::<BigInt>::new(unstake_amount.into(), 5.into()) * unit_price;
        let burn_amount = burn_amount.to_integer().to_u128().unwrap();

        ext_usd::burn_to_unstake(
            account_id,
            burn_amount,
            unstake_amount,
            &self.ausd_token,
            0,
            env::prepaid_gas() / 3,
        )
    }

    pub fn sell_asset_to_ausd(&mut self, asset: String, asset_amount: String) -> Promise {
        let asset_price = self._get_asset_price(&asset);
        if asset_price == 0 {
            env::panic(b"No price data from oracle");
        }
        let asset_amount = u128::from_str(&asset_amount).expect("Failed to parse asset_amount");

        let account_id = env::predecessor_account_id();
        let mut account = self.get_account(&account_id);
        let balance = self._get_asset_balance(&account_id, &asset);
        let new_balance = balance.checked_sub(asset_amount).unwrap();
        account.assets.insert(asset.clone(), new_balance);
        self.accounts.insert(&account_id, &account);

        let unit_price = Ratio::<BigInt>::new(asset_price.into(), 100_000_000.into());
        let asset_amount: BigInt = asset_amount.into();

        let mint_amount: Ratio<BigInt> = unit_price * asset_amount;
        let mint_amount = mint_amount.to_integer().to_u128().unwrap();

        ext_usd::mint(
            account_id,
            mint_amount,
            &self.ausd_token,
            0,
            env::prepaid_gas() / 2,
        )
    }

    pub fn buy_asset_with_ausd(&mut self, asset: String, asset_amount: String) -> Promise {
        let asset_price = self._get_asset_price(&asset);
        if asset_price == 0 {
            env::panic(b"No price data from oracle");
        }
        let asset_amount = u128::from_str(&asset_amount).expect("Failed to parse asset_amount");
        let unit_price = Ratio::<BigInt>::new(asset_price.into(), 100_000_000.into());
        let asset_amount_b: BigInt = asset_amount.into();

        let burn_amount: Ratio<BigInt> = unit_price * asset_amount_b;
        let burn_amount = burn_amount.to_integer().to_u128().unwrap();

        let account_id = env::predecessor_account_id();
        ext_usd::burn_to_buy_asset(
            account_id,
            burn_amount,
            asset.clone(),
            asset_amount,
            &self.ausd_token,
            0,
            env::prepaid_gas() / 3,
        )
    }

    pub fn buy_asset_callback(&mut self, account_id: String, asset: String, asset_amount: u128) {
        assert!(
            env::predecessor_account_id() == self.ausd_token,
            "Only allow unstake originated from ausd token"
        );

        let mut account = self.get_account(&account_id);
        let balance = self._get_asset_balance(&account_id, &asset);
        let new_balance = balance.checked_add(asset_amount).unwrap();
        account.assets.insert(asset.clone(), new_balance);
        self.accounts.insert(&account_id, &account);
    }

    /// Stakes an additional `stake_amount` to the signer
    /// Requirements:
    /// * The signer should have enough unstaked balance.
    fn stake(&mut self, stake_amount: String) -> u128 {
        let stake_amount = u128::from_str(&stake_amount).expect("Failed to parse stake_amount");
        if stake_amount == 0 {
            env::panic(b"Can't stake 0 tokens");
        }
        self.refresh_reward();
        let account_id = env::predecessor_account_id();
        let mut account = self.get_account(&account_id);

        // Checking and updating unstaked balance
        if account.balance < stake_amount {
            env::panic(b"Not enough unstaked balance");
        }
        account.balance -= stake_amount;

        // Updating total stake balance
        let staked_balance = account.get_staked_balance();
        account.set_staked_balance(staked_balance + stake_amount);
        self.total_staked += stake_amount;
        self.reward_paid_at
            .insert(&account_id, &env::block_timestamp());

        self.accounts.insert(&account_id, &account);
        stake_amount
    }

    /// Unstakes the `unstake_amount` from the owner
    pub fn unstake(&mut self, account_id: String, unstake_amount: u128) {
        assert!(
            env::predecessor_account_id() == self.ausd_token,
            "Only allow unstake originated from ausd token"
        );
        if unstake_amount == 0 {
            env::panic(b"Can't unstake 0 tokens");
        }
        self.refresh_reward();

        let mut account = self.get_account(&account_id);

        // Checking and updating staked balance
        let staked_balance = account.get_staked_balance();
        if staked_balance < unstake_amount {
            env::panic(b"Not enough staked tokens");
        }
        account.set_staked_balance(staked_balance - unstake_amount);

        // Updating unstaked balance
        account.balance += unstake_amount;
        self.total_staked -= unstake_amount;
        self.accounts.insert(&account_id, &account);
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
        self.get_account(&owner_id)
            .get_allowance(&escrow_account_id)
            .to_string()
    }

    /// Returns current staked balance for the `owner_id` staked by `escrow_account_id`.
    pub fn get_staked_balance(&self, account_id: AccountId) -> String {
        self.get_account(&account_id)
            .get_staked_balance()
            .to_string()
    }

    pub fn get_price(&self) -> String {
        self.price.to_string()
    }

    pub fn get_asset_price(&self, asset: String) -> String {
        self._get_asset_price(&asset).to_string()
    }

    pub fn get_asset_balance(&self, account_id: AccountId, asset: String) -> String {
        self._get_asset_balance(&account_id, &asset).to_string()
    }

    pub fn get_reward_paid_at(&self, account_id: AccountId) -> u64 {
        self.reward_paid_at
            .get(&account_id)
            .unwrap_or(self.staking_reward_enabled_at)
    }
}

impl Art {
    /// Helper method to get the account details for `owner_id`.
    fn get_account(&self, owner_id: &AccountId) -> Account {
        self.accounts.get(owner_id).unwrap_or_default()
    }

    fn _get_asset_price(&self, asset: &String) -> u128 {
        self.asset_prices.get(asset).unwrap_or_default()
    }

    fn _get_asset_balance(&self, account_id: &AccountId, asset: &String) -> Balance {
        *self
            .get_account(&account_id)
            .assets
            .get(asset)
            .unwrap_or(&0)
    }
}

#[ext_contract(ext_self)]
trait FungibleTokenResolver {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128;
}

#[ext_contract(ext_fungible_token_receiver)]
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleTokenContract {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;

    /// Returns the total supply of the token in a decimal string representation.
    fn ft_total_supply(&self) -> U128;

    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

const GAS_FOR_RESOLVE_TRANSFER: Gas = 5_000_000_000_000;
const GAS_FOR_FT_TRANSFER_CALL: Gas = 25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER;

const NO_DEPOSIT: Balance = 0;

#[near_bindgen]
impl FungibleTokenMetadataProvider for Art {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "Artificial Coin".to_string(),
            symbol: "art".to_string(),
            icon: Some("https://artcoin.network/static/media/logo192.c7f41b7c.png".to_string()),
            decimals: 24,
            reference: None,
            reference_hash: None,
        }
    }
}

#[near_bindgen]
impl StorageManagement for Art {
    // `registration_only` doesn't affect the implementation for vanilla fungible token.
    #[allow(unused_variables)]
    fn storage_deposit(
        &mut self,
        account_id: Option<ValidAccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let account_id = account_id
            .map(|a| a.into())
            .unwrap_or_else(|| env::predecessor_account_id());
        if self.accounts.get(&account_id).is_some() {
            log!("The account is already registered, refunding the deposit");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic(b"The attached deposit is less than the mimimum storage balance");
            }

            self.internal_register_account(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }

    /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
    /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
    /// which means available balance will always be 0. So this implementation:
    /// * panics if `amount > 0`
    /// * never transfers Ⓝ to caller
    /// * returns a `storage_balance` struct if `amount` is 0
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic(b"The amount is greater than the available storage balance");
                }
                _ => storage_balance,
            }
        } else {
            env::panic(
                format!("The account {} is not registered", &predecessor_account_id).as_bytes(),
            );
        }
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.internal_storage_unregister(force).is_some()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: ValidAccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(account_id.as_ref())
    }
}

impl Art {
    /// Internal method that returns the Account ID and the balance in case the account was
    /// unregistered.
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false);
        if let Some(account) = self.accounts.get(&account_id) {
            let balance = account.balance.checked_add(account.staked_balance).unwrap();
            if balance == 0 || force {
                self.accounts.remove(&account_id);
                self.total_supply -= balance;
                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1);
                Some((account_id, balance))
            } else {
                env::panic(b"Can't unregister the account with the positive balance without force")
            }
        } else {
            log!("The account {} is not registered", &account_id);
            None
        }
    }

    fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.accounts.get(account_id).is_some() {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self
            .accounts
            .insert(&account_id, &Default::default())
            .is_some()
        {
            env::panic(b"The account is already registered");
        }
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        match self.accounts.get(&account_id) {
            Some(account) => account.balance,
            None => env::panic(format!("The account {} is not registered", &account_id).as_bytes()),
        }
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let mut account = self.accounts.get(&account_id).unwrap();
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            account.balance = new_balance;
            self.accounts.insert(&account_id, &account);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .expect("Total supply overflow");
        } else {
            env::panic(b"Balance overflow");
        }
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let mut account = self.accounts.get(&account_id).unwrap();
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            account.balance = new_balance;
            self.accounts.insert(&account_id, &account);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .expect("Total supply overflow");
        } else {
            env::panic(b"The account doesn't have enough balance");
        }
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        assert_ne!(
            sender_id, receiver_id,
            "Sender and receiver should be different"
        );
        assert!(amount > 0, "The amount should be a positive number");
        self.internal_withdraw(sender_id, amount);
        self.internal_deposit(receiver_id, amount);
        log!("Transfer {} from {} to {}", amount, sender_id, receiver_id);
        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }
    }
}

#[near_bindgen]
impl FungibleTokenCore for Art {
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, receiver_id.as_ref(), amount, memo);
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        let amount: Balance = amount.into();
        self.internal_transfer(&sender_id, receiver_id.as_ref(), amount, memo);
        // Initiating receiver's call and the callback
        ext_fungible_token_receiver::ft_on_transfer(
            sender_id.clone(),
            amount.into(),
            msg,
            receiver_id.as_ref(),
            NO_DEPOSIT,
            env::prepaid_gas() - GAS_FOR_FT_TRANSFER_CALL,
        )
        .then(ext_self::ft_resolve_transfer(
            sender_id,
            receiver_id.into(),
            amount.into(),
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128 {
        self.accounts
            .get(account_id.as_ref())
            .unwrap_or(Default::default())
            .balance
            .into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use near_sdk::env::STORAGE_PRICE_PER_BYTE;
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
        assert_eq!(
            contract.get_unstaked_balance(bob()),
            total_supply.to_string()
        );
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
        assert_eq!(
            contract.get_unstaked_balance(bob()),
            transfer_amount.to_string()
        );
    }

    #[test]
    fn test_stake_fail() {
        let context = get_context(carol());
        testing_env!(context);
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        let transfer_amount = total_supply / 3;
        let context = get_context(bob());
        testing_env!(context);
        std::panic::catch_unwind(move || {
            contract.stake(transfer_amount.to_string());
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

    #[test]
    fn test_carol_escrows_to_bob_transfers_to_alice() {
        // Acting as carol
        testing_env!(get_context(carol()));
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        assert_eq!(contract.get_total_supply(), total_supply.to_string());
        let allowance = total_supply / 3;
        let transfer_amount = allowance / 3;
        contract.set_allowance(bob(), format!("{}", allowance));
        assert_eq!(
            contract.get_allowance(carol(), bob()),
            format!("{}", allowance)
        );
        // Acting as bob now
        testing_env!(get_context(bob()));
        contract.transfer_from(carol(), alice(), transfer_amount.to_string());
        assert_eq!(
            contract.get_total_balance(carol()),
            (total_supply - transfer_amount).to_string()
        );
        assert_eq!(
            contract.get_unstaked_balance(alice()),
            transfer_amount.to_string()
        );
        assert_eq!(
            contract.get_allowance(carol(), bob()),
            format!("{}", allowance - transfer_amount)
        );
    }

    // Fungible Token Standard tests

    #[test]
    fn test_ft_transfer() {
        let mut context = get_context(carol());
        testing_env!(context.clone());
        let total_supply = 1_000_000_000_000_000u128;
        let mut contract = Art::new(carol(), total_supply.to_string(), "ausd".to_string());
        context.storage_usage = env::storage_usage();
        context.attached_deposit = 1000 * STORAGE_PRICE_PER_BYTE;
        testing_env!(context.clone());
        contract.storage_deposit(Some(bob().try_into().unwrap()), None);

        context.storage_usage = env::storage_usage();
        context.attached_deposit = 1;
        testing_env!(context.clone());
        let transfer_amount = total_supply / 3;
        contract.ft_transfer(bob().try_into().unwrap(), transfer_amount.into(), None);
        context.storage_usage = env::storage_usage();
        context.account_balance = env::account_balance();

        context.is_view = true;
        context.attached_deposit = 0;
        testing_env!(context.clone());
        assert_eq!(
            contract.ft_balance_of(carol().try_into().unwrap()).0,
            (total_supply - transfer_amount)
        );
        assert_eq!(
            contract.ft_balance_of(bob().try_into().unwrap()).0,
            transfer_amount
        );
        assert_eq!(contract.ft_total_supply().0, total_supply);
    }
}
