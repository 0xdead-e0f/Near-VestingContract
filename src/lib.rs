use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, require, AccountId, Balance, Promise};
use near_sdk_contract_tools::{owner::Owner, Owner};

pub use near_sdk_contract_tools::owner::OwnerExternal;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Owner)]
pub struct VestingContract {
    token_contract: AccountId,
    allocations: std::collections::HashMap<AccountId, Allocation>,
    lockin_block_length: u128,
    unlock_block_length: u128,
    lock_all_accounts: bool,
    // Other vesting-related fields...
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Allocation {
    total_amount: Balance,
    released_amount: Balance,
    start_block: u64,
    last_released_block: u64,
    per_block_release_amount: Balance,
}

const MONTH_IN_SECONDS:u128 = 3600 * 24 * 30;

#[near_bindgen]
impl VestingContract {
    #[init]
    pub fn new(token_contract: AccountId, lockin_period: u128, unlock_period: u128) -> Self {
        let lockin_block_length = lockin_period * MONTH_IN_SECONDS * 10 / 12;
        let unlock_block_length = unlock_period * MONTH_IN_SECONDS * 10 / 12;

        let mut contract = Self {
            token_contract,
            allocations: std::collections::HashMap::new(),
            lockin_block_length,
            unlock_block_length,
            lock_all_accounts: false,
            // Initialize other vesting-related fields...
        };

        Owner::init(&mut contract, &env::signer_account_id());
        contract
    }

    pub fn owner_only(&self) {
        Self::require_owner();
        // ...
    }

    pub fn release_vested_tokens(&mut self) {
        // Perform checks and calculations...
        let allocation = self
            .allocations
            .get(&env::predecessor_account_id())
            .expect("Account has no allocation");

        require!((env::block_height() - allocation.start_block) as u128 > self.lockin_block_length, "Account is in lock status");

        let elapsed_blocks = (env::block_height() - allocation.last_released_block) as u128;
        let release_amount = elapsed_blocks * allocation.per_block_release_amount;

        let total_amount = allocation.total_amount;
        let unreleased_amount = total_amount.saturating_sub(allocation.released_amount);
        let release_amount = std::cmp::min(release_amount, unreleased_amount);

        Promise::new(self.token_contract.clone()).function_call(
            "transfer".to_string(),
            format!(
                r#"{{"receiver":"{}", "amount":"{}"}}"#,
                env::predecessor_account_id(),
                release_amount
            )
            .into_bytes(), // Convert String to Vec<u8>
            0,
            env::prepaid_gas(),
        );

        let allocation = self
            .allocations
            .get_mut(&env::predecessor_account_id())
            .unwrap();
        allocation.released_amount += release_amount as u128;
        allocation.last_released_block = env::block_height();
    }

    pub fn add_allocation(&mut self, account_id: AccountId, total_amount: Balance) {
        self.owner_only();
        
        require!(total_amount > self.unlock_block_length, "Insufficient amount" );
        let per_block_release_amount = total_amount / self.unlock_block_length;       
        let start_block = env::block_height();
        let allocation = Allocation {
            total_amount,
            released_amount: 0,
            start_block,
            last_released_block: start_block + self.lockin_block_length as u64,
            per_block_release_amount,
        };
        self.allocations.insert(account_id, allocation);
    }

    pub fn set_lock_all_accounts(&mut self, lock_all_accounts: bool) {
        self.owner_only();
        self.lock_all_accounts = lock_all_accounts;
    }
    // pub fn set_per_block_release_amount(&mut self, per_block_release_amount: Balance) {
    //     self.per_block_release_amount = per_block_release_amount;
    // }

    // Other vesting-related functions...
}

// Implement the Default trait for VestingContract
// Only do this if a default state makes sense for your contract
impl Default for VestingContract {
    fn default() -> Self {
        Self {
            token_contract: AccountId::new_unchecked("".to_string()),
            allocations: std::collections::HashMap::new(),
            lockin_block_length: 0,
            unlock_block_length: 0,
            lock_all_accounts: false
            // Set default values for other fields...
        }
    }
}
