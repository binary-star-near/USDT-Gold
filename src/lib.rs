use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, sys, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue,
};

use std::convert::TryFrom;

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum BlackListStatus {
    // An address might be using
    Allowable,
    // All acts with an address have to be banned
    Banned,
}

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize,
)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractStatus {
    Working,
    Paused,
}

impl std::fmt::Display for ContractStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractStatus::Working => write!(f, "working"),
            ContractStatus::Paused => write!(f, "paused"),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner_id: AccountId,
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    black_list: LookupMap<AccountId, BlackListStatus>,
    status: ContractStatus,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str =
    "data:image/svg+xml;charset=UTF-8,%3Csvg%20xmlns=%22http://www.w3.org/2000/svg%22%20xmlns:xlink=%22http://www.w3.org/1999/xlink%22%20viewBox=%220%200%20611.9%20532.43%22%3E%3Cdefs%3E%3Cstyle%3E.cls-1,.cls-2%7Bfill-rule:evenodd;%7D.cls-1%7Bfill:url(#linear-gradient);%7D.cls-2%7Bfill:#fff;%7D%3C/style%3E%3ClinearGradient%20id=%22linear-gradient%22%20x1=%224846.23%22%20y1=%22-4671.46%22%20x2=%2213324.84%22%20y2=%22-4671.46%22%20gradientTransform=%22matrix(0.07,%200,%200,%20-0.07,%20-348.93,%20-70.13)%22%20gradientUnits=%22userSpaceOnUse%22%3E%3Cstop%20offset=%220%22%20stop-color=%22#d8b35d%22/%3E%3Cstop%20offset=%221%22%20stop-color=%22#f5e7bf%22/%3E%3C/linearGradient%3E%3C/defs%3E%3Ctitle%3Etether-gold%3C/title%3E%3Cg%20id=%22Layer_2%22%20data-name=%22Layer%202%22%3E%3Cg%20id=%22Layer_1-2%22%20data-name=%22Layer%201%22%3E%3Cg%20id=%22Layer_1-3%22%20data-name=%22Layer_1%22%3E%3Cpath%20class=%22cls-1%22%20d=%22M112.11,2.71.49,237.12a4.36,4.36,0,0,0,.9,5.23L302.7,531.22a4.72,4.72,0,0,0,6.32,0L610.51,242.35a4.41,4.41,0,0,0,.9-5.23L499.79,2.71a4.12,4.12,0,0,0-4-2.71H116.07a4.49,4.49,0,0,0-4,2.71Z%22/%3E%3Cpath%20class=%22cls-2%22%20d=%22M344.72,261.1h0c-2.16.18-13.34.9-38.23.9-19.83,0-33.9-.54-38.95-.9h0c-76.63-3.42-133.79-16.77-133.79-32.64s57.16-29.39,133.79-32.81v52.11c5,.36,19.3,1.26,39.13,1.26,23.8,0,35.7-1.08,37.87-1.26V195.65c76.45,3.42,133.61,16.77,133.61,32.63s-57,29.4-133.43,32.82Zm0-70.68V143.71H451.47V72.49H160.8v71.22H267.54v46.71c-86.73,4-152,21.09-152,41.83s65.27,37.69,152,41.83V423.57h77.18V274.08c86.55-4,151.65-21.1,151.65-41.65s-65.1-38.05-151.65-42h0Z%22/%3E%3C/g%3E%3C/g%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    // Initializes the contract with the given total supply owned by the given `owner_id` with
    // default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "USDT Gold".to_string(),
                symbol: "XAUT".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 8,
            },
        )
    }

    // Initializes the contract with the given total supply owned by the given `owner_id` with
    // the given fungible token metadata.
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            owner_id: owner_id.clone(),
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            black_list: LookupMap::new(b"b".to_vec()),
            status: ContractStatus::Working,
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        this
    }

    pub fn upgrade_name_symbol(&mut self, name: String, symbol: String) {
        self.abort_if_not_owner();
        let metadata = self.metadata.get();
        if let Some(mut metadata) = metadata {
            metadata.name = name;
            metadata.symbol = symbol;
            self.metadata.replace(&metadata);
        }
    }

    pub fn set_owner(&mut self, owner_id: AccountId) {
        self.abort_if_not_owner();
        self.owner_id = owner_id;
    }

    pub fn upgrade_icon(&mut self, data: String) {
        self.abort_if_not_owner();
        let metadata = self.metadata.get();
        if let Some(mut metadata) = metadata {
            metadata.icon = Some(data);
            self.metadata.replace(&metadata);
        }
    }

    pub fn get_blacklist_status(&self, account_id: &AccountId) -> BlackListStatus {
        self.abort_if_pause();
        return match self.black_list.get(account_id) {
            Some(x) => x.clone(),
            None => BlackListStatus::Allowable,
        };
    }

    pub fn add_to_blacklist(&mut self, account_id: &AccountId) {
        self.abort_if_not_owner();
        self.abort_if_pause();
        self.black_list.insert(account_id, &BlackListStatus::Banned);
    }

    pub fn remove_from_blacklist(&mut self, account_id: &AccountId) {
        self.abort_if_not_owner();
        self.abort_if_pause();
        self.black_list
            .insert(account_id, &BlackListStatus::Allowable);
    }

    pub fn destroy_black_funds(&mut self, account_id: &AccountId) {
        self.abort_if_not_owner();
        self.abort_if_pause();
        assert_eq!(
            self.get_blacklist_status(&account_id),
            BlackListStatus::Banned
        );
        let black_balance = self.ft_balance_of(account_id.clone());
        if black_balance.0 <= 0 {
            env::panic_str("The account doesn't have enough balance");
        }
        self.token.accounts.insert(account_id, &0u128);
        self.token.total_supply = self
            .token
            .total_supply
            .checked_sub(u128::from(black_balance))
            .expect("Failed to decrease total supply");
    }

    // Issue a new amount of tokens
    // these tokens are deposited into the owner address
    pub fn issue(&mut self, amount: U128) -> Balance {
        self.mint(&env::current_account_id(), amount)
    }

    // Creates `amount` tokens and assigns them to `account`, increasing
    // the total supply.
    pub fn mint(&mut self, account_id: &AccountId, amount: U128) -> Balance {
        self.abort_if_not_owner();
        self.abort_if_pause();
        // Add amount to total_supply
        self.token.total_supply = self
            .token
            .total_supply
            .checked_add(u128::from(amount))
            .expect("Issue caused supply overflow");
        // Add amount to owner balance
        if let Some(owner_amount) = self.token.accounts.get(account_id) {
            self.token.accounts.insert(
                account_id,
                &(owner_amount
                    .checked_add(u128::from(amount))
                    .expect("Owner has exceeded balance") as Balance),
            );
        } else {
            self.token
                .accounts
                .insert(account_id, &Balance::from(amount));
        }
        // Return upgraded total_supply
        self.token.total_supply
    }

    // Redeem tokens (burn).
    // These tokens are withdrawn from the owner address
    // if the balance must be enough to cover the redeem
    // or the call will fail.
    pub fn redeem(&mut self, amount: U128) {
        self.burn(&env::current_account_id(), amount)
    }

    // Redeem tokens (burn).
    // These tokens are withdrawn from the owner address
    // if the balance must be enough to cover the redeem
    // or the call will fail.
    pub fn burn(&mut self, account_id: &AccountId, amount: U128) {
        self.abort_if_not_owner();
        self.abort_if_pause();
        assert!(&self.token.total_supply >= &Balance::from(amount));
        assert!(u128::from(self.ft_balance_of(account_id.clone())) >= u128::from(amount));

        self.token.total_supply = self
            .token
            .total_supply
            .checked_sub(u128::from(amount))
            .expect("Redeem caused supply underflow");

        if let Some(owner_amount) = self.token.accounts.get(&account_id.clone().into()) {
            self.token.accounts.insert(
                account_id,
                &(owner_amount
                    .checked_sub(u128::from(amount))
                    .expect("The owner has subceed balance") as Balance),
            );
        }
    }

    // If we have to pause contract
    pub fn pause(&mut self) {
        self.abort_if_not_owner();
        self.status = ContractStatus::Paused;
    }

    // If we have to resume contract
    pub fn resume(&mut self) {
        self.abort_if_not_owner();
        self.status = ContractStatus::Working;
    }

    pub fn contract_status(&self) -> ContractStatus {
        self.status
    }

    /**
     * @dev Returns the name of the token.
     */
    pub fn name(&mut self) -> String {
        self.abort_if_pause();
        let metadata = self.metadata.get();
        metadata.expect("Unable to get decimals").name
    }

    /**
     * Returns the symbol of the token.
     */
    pub fn symbol(&mut self) -> String {
        self.abort_if_pause();
        let metadata = self.metadata.get();
        metadata.expect("Unable to get decimals").symbol
    }

    /**
     * Returns the decimals places of the token.
     */
    pub fn decimals(&mut self) -> u8 {
        self.abort_if_pause();
        let metadata = self.metadata.get();
        metadata.expect("Unable to get decimals").decimals
    }

    pub fn get_version(&self) -> String {
        format!("{}:{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// This method is called from `upgrade()` method.
    /// For next version upgrades, change this function.
    #[init(ignore_state)]
    #[private]
    pub fn migrate() -> Self {
        let this: Contract = env::state_read().expect("Contract is not initialized.");
        this
    }

    fn abort_if_pause(&self) {
        if self.status == ContractStatus::Paused {
            env::panic_str("Operation aborted because the contract under maintenance")
        }
    }

    fn abort_if_not_owner(&self) {
        if env::predecessor_account_id() != env::current_account_id()
            && env::predecessor_account_id() != self.owner_id
        {
            env::panic_str("This method might be called only by owner account")
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: &AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

#[no_mangle]
pub fn upgrade() {
    env::setup_panic_hook();

    let contract: Contract = env::state_read().expect("Contract is not initialized");
    contract.abort_if_not_owner();

    const MIGRATE_METHOD_NAME: &[u8; 7] = b"migrate";
    const UPGRADE_GAS_LEFTOVER: Gas = Gas(5_000_000_000_000);

    unsafe {
        // Load code into register 0 result from the input argument if factory call or from promise if callback.
        sys::input(0);
        // Create a promise batch to upgrade current contract with code from register 0.
        let promise_id = sys::promise_batch_create(
            env::current_account_id().as_bytes().len() as u64,
            env::current_account_id().as_bytes().as_ptr() as u64,
        );
        // Deploy the contract code from register 0.
        sys::promise_batch_action_deploy_contract(promise_id, u64::MAX, 0);
        // Call promise to migrate the state.
        // Batched together to fail upgrade if migration fails.
        sys::promise_batch_action_function_call(
            promise_id,
            MIGRATE_METHOD_NAME.len() as u64,
            MIGRATE_METHOD_NAME.as_ptr() as u64,
            0,
            0,
            0,
            (env::prepaid_gas() - env::used_gas() - UPGRADE_GAS_LEFTOVER).0,
        );
        sys::promise_return(promise_id);
    }
}

/// The core methods for a basic fungible token. Extension standards may be
/// added in addition to this macro.

#[near_bindgen]
impl FungibleTokenCore for Contract {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.abort_if_pause();
        let sender_id = AccountId::try_from(env::signer_account_id())
            .expect("Couldn't validate sender address");
        match self.get_blacklist_status(&sender_id) {
            BlackListStatus::Allowable => {
                assert!(u128::from(self.ft_balance_of(sender_id)) >= u128::from(amount));
                self.token.ft_transfer(receiver_id.clone(), amount, memo);
            }
            BlackListStatus::Banned => {
                env::panic_str("Signer account is banned. Operation is not allowed.");
            }
        };
    }

    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.abort_if_pause();
        let sender_id = AccountId::try_from(env::signer_account_id())
            .expect("Couldn't validate sender address");
        match self.get_blacklist_status(&sender_id) {
            BlackListStatus::Allowable => {
                assert!(u128::from(self.ft_balance_of(sender_id)) >= u128::from(amount));
                self.token
                    .ft_transfer_call(receiver_id.clone(), amount, memo, msg)
            }
            BlackListStatus::Banned => {
                env::panic_str("Signer account is banned. Operation is not allowed.")
            }
        }
    }

    fn ft_total_supply(&self) -> U128 {
        self.abort_if_pause();
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.abort_if_pause();
        self.token.ft_balance_of(account_id)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let sender_id: AccountId = sender_id.into();
        let (used_amount, burned_amount) =
            self.token
                .internal_ft_resolve_transfer(&sender_id, receiver_id, amount);
        if burned_amount > 0 {
            self.on_tokens_burned(
                &AccountId::try_from(sender_id).expect("Couldn't validate sender address"),
                burned_amount,
            );
        }
        used_amount.into()
    }
}

near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_contract_status() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        assert_eq!(contract.contract_status(), ContractStatus::Working);
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }

    #[test]
    fn test_blacklist() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .current_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());

        assert_eq!(
            contract.get_blacklist_status(&accounts(1)),
            BlackListStatus::Allowable
        );

        contract.add_to_blacklist(&accounts(1));
        assert_eq!(
            contract.get_blacklist_status(&accounts(1)),
            BlackListStatus::Banned
        );

        contract.remove_from_blacklist(&accounts(1));
        assert_eq!(
            contract.get_blacklist_status(&accounts(1)),
            BlackListStatus::Allowable
        );
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.issue(U128::from(transfer_amount));

        contract.add_to_blacklist(&accounts(1));
        let total_supply_before = contract.token.total_supply;

        contract.destroy_black_funds(&accounts(1));
        assert_ne!(total_supply_before, contract.token.total_supply);
    }

    #[test]
    #[should_panic]
    fn test_destroy_black_funds_panic() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .current_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());

        contract.issue(U128::from(1000));
        contract.add_to_blacklist(&accounts(1));
        contract.destroy_black_funds(&accounts(1));

        contract.issue(U128::from(1000));
        contract.remove_from_blacklist(&accounts(1));
        contract.destroy_black_funds(&accounts(1));
    }

    #[test]
    fn test_issuance() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .current_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());

        let previous_total_supply = contract.ft_total_supply().0;
        let previous_balance = contract.ft_balance_of(accounts(1)).0;
        let reissuance_balance: Balance = 1_234_567_891;
        contract.issue(U128::from(reissuance_balance));
        assert_eq!(
            previous_total_supply + reissuance_balance,
            contract.ft_total_supply().0
        );
        assert_eq!(
            previous_balance + reissuance_balance,
            contract.ft_balance_of(accounts(1)).0
        );
    }

    #[test]
    fn test_redeem() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .current_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());

        let previous_total_supply = contract.ft_total_supply();
        let previous_balance = contract.ft_balance_of(accounts(1));
        let reissuance_balance: Balance = 1_234_567_891;
        contract.issue(U128::from(reissuance_balance));
        contract.redeem(U128::from(reissuance_balance));
        assert_eq!(previous_total_supply, contract.ft_total_supply());
        assert_eq!(previous_balance, contract.ft_balance_of(accounts(1)));
    }

    #[test]
    fn test_maintenance() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .current_account_id(accounts(1))
            .signer_account_id(accounts(1))
            .build());
        assert_eq!(contract.contract_status(), ContractStatus::Working);
        contract.pause();
        assert_eq!(contract.contract_status(), ContractStatus::Paused);
        contract.resume();
        assert_eq!(contract.contract_status(), ContractStatus::Working);
        contract.pause();
        let result = std::panic::catch_unwind(move || {
            let _ = contract.ft_total_supply();
        });
        assert!(result.is_err());
    }
}
