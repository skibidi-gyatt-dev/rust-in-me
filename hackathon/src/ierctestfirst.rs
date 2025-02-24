#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
    contract,
    storage::StorageAddress,
};

/// Define the ERC-20 interface
#[external]
pub trait IERC20 {
    fn balance_of(&self, account: Address) -> U256;
    fn transfer(&mut self, recipient: Address, amount: U256) -> bool;
}

/// Storage for Counter contract
sol_storage! {
    #[entrypoint]
    pub struct Counter {
        address token;
        address admin;
        mapping(address => uint256) balances;
    }
}

#[public]
impl Counter {
    /// Initialize admin only if not already set.
    pub fn initialize_admin(&mut self) {
        if Address::from(*self.admin.get()) == Address::default() {
            self.admin.set(self.vm().msg_sender());
        }
    }

    /// Change contract admin
    pub fn change_admin(&mut self, new_admin: Address) {
        assert_eq!(
            Address::from(*self.admin.get()),
            self.vm().msg_sender(),
            "Only admin can change admin"
        );
        self.admin.set(new_admin);
    }

    /// Set ERC-20 token address
    pub fn set_token(&mut self, token_addr: Address) {
        self.token.set(token_addr);
    }

    /// Deposit tokens into the contract
    pub fn deposit(&mut self, amount: U256) -> bool {
        let employer = self.vm().msg_sender();
        let token = IERC20::from(*self.token);

        // Check balance first
        let ext_balance = token.balance_of(employer);
        assert!(ext_balance >= amount, "Not enough balance");

        // Transfer tokens
        let success = token.transfer(contract::address(), amount);
        assert!(success, "Transfer failed");

        // Update employer's balance
        let current = self.balances.get(employer);
        self.balances.setter(employer).set(current + amount);

        true
    }

    /// Pay workers from employer's balance
    pub fn pay_workers(&mut self, workers: Vec<(Address, U256)>) -> bool {
        let employer = self.vm().msg_sender();
        let mut available = self.balances.get(employer);
        let token = IERC20::from(*self.token);

        for (worker_address, amount) in workers {
            if available < amount {
                continue;
            }

            let success = token.transfer(worker_address, amount);
            assert!(success, "Transfer failed");

            available -= amount;
            self.balances.setter(employer).set(available);
        }
        true
    }

    /// Get employer's internal token balance
    pub fn employer_balance(&self, employer: Address) -> U256 {
        self.balances.get(employer)
    }

    /// Get caller's internal balance
    pub fn my_balance(&self) -> U256 {
        self.balances.get(self.vm().msg_sender())
    }

    /// Get balance from the ERC-20 token contract
    pub fn token_balance(&self, owner: Address) -> U256 {
        let token = IERC20::from(*self.token);
        token.balance_of(owner)
    }
}
