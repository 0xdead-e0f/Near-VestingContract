use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct VestingContract {
    token_contract: AccountId,
    allocations: std::collections::HashMap<AccountId, Allocation>,
    per_block_release_amount: Balance,
    // Other vesting-related fields...
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Allocation {
    total_amount: Balance,
    released_amount: Balance,
    start_block: u64,
}

#[near_bindgen]
impl VestingContract {
    #[init]
    pub fn new(token_contract: AccountId, per_block_release_amount: Balance) -> Self {
        Self {
            token_contract,
            allocations: std::collections::HashMap::new(),
            per_block_release_amount,
            // Initialize other vesting-related fields...
        }
    }

    pub fn release_vested_tokens(&mut self) {
        // Perform checks and calculations...

        let allocation = self
            .allocations
            .get(&env::predecessor_account_id())
            .expect("Account has no allocation");

        let elapsed_blocks = env::block_height() - allocation.start_block;
        let release_amount = elapsed_blocks as u128 * self.per_block_release_amount;

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
    }

    pub fn add_allocation(&mut self, account_id: AccountId, total_amount: Balance) {
        let start_block = env::block_height();
        let allocation = Allocation {
            total_amount,
            released_amount: 0,
            start_block,
        };
        self.allocations.insert(account_id, allocation);
    }

    pub fn set_per_block_release_amount(&mut self, per_block_release_amount: Balance) {
        self.per_block_release_amount = per_block_release_amount;
    }

    // Other vesting-related functions...
}

// Implement the Default trait for VestingContract
// Only do this if a default state makes sense for your contract
impl Default for VestingContract {
    fn default() -> Self {
        Self {
            token_contract: AccountId::new_unchecked("".to_string()),
            allocations: std::collections::HashMap::new(),
            per_block_release_amount: 0,
            // Set default values for other fields...
        }
    }
}
