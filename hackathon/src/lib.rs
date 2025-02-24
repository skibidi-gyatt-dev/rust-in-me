#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolCall},
    call::RawCall, 
    prelude::*,
    storage::{StorageAddress, Mapping},
};

#[storage]
#[entrypoint]
pub struct Counter {
    token: StorageAddress,
    admin: StorageAddress,
    // Adding a mapping equivalent to Solidity’s: mapping(address => uint256)
    balances: Mapping<Address, U256>,
}

pub struct Worker {
    worker_address: Address,
    amount_paid: U256,
}

sol! {
    function balanceOf(address account) returns (uint256);
    function transfer(address recipient, uint256 value) returns (bool);
    event CounterCreated(address counter, address owner);
}

#[public]
impl Counter {
    pub fn initialize_admin(&mut self) {
        if Address::from(*self.admin.get()) == Address::default() {
            self.admin.set(self.vm().msg_sender());
        }
    }

    pub fn change_admin(&mut self, new_admin: Address) {
        assert_eq!(
            Address::from(*self.admin.get()),
            self.vm().msg_sender(),
            "Only admin can change admin"
        );
        self.admin.set(new_admin);
    }

    pub fn set_address(&mut self, _token: Address) {
        self.token.set(_token);
    }
    
    // Functions for interacting with the mapping.
    // This is equivalent to setting a value in Solidity's mapping.
    pub fn set_balance(&mut self, account: Address, amount: U256) {
        self.balances.set(account, amount);
    }
    
    // Retrieves a balance. If no value is set, returns 0.
    pub fn get_balance(&self, account: Address) -> U256 {
        self.balances.get(account).unwrap_or(U256::zero())
    }

    pub fn pay_workers(&mut self, workers: Vec<(Address, U256)>) {
        assert_ne!(
            Address::from(*self.admin.get()),
            Address::default(),
            "Admin not initialized"
        );

        for (worker_address, amount_paid) in workers {
            let success = self.transfer(worker_address, amount_paid);
            if !success {
                continue;
            }
        }
    }

    pub fn current_admin(&self) -> Result<Address, Vec<u8>> {
        Ok(Address::from(*self.admin.get()))
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        let result = RawCall::new_static().call(
            alloy_primitives::Address(*self.token.get()),
            &balanceOfCall { account: owner }.abi_encode(),
        );

        match result {
            Ok(data) => U256::from_be_bytes::<32>(data.try_into().unwrap_or([0u8; 32])),
            Err(_) => U256::from(0),
        }
    }

    pub fn transfer(&self, recipient: Address, amount: U256) -> bool {
        let token_addr = alloy_primitives::Address(*self.token.get());
        Self::perform_transfer(token_addr, recipient, amount)
    }

    pub fn transfer_to(&self, recipient: Address, token_add: Address, amount: U256) -> bool {
        let token_addr = alloy_primitives::Address(*token_add);
        Self::perform_transfer(token_addr, recipient, amount)
    }

    fn perform_transfer(
        token_addr: alloy_primitives::Address,
        recipient: Address,
        amount: U256,
    ) -> bool {
        let call_data = transferCall {
            recipient,
            value: amount,
        }
        .abi_encode();
        let result = RawCall::new().call(token_addr, &call_data);
        match result {
            Ok(data) => data.first().copied() == Some(1),
            Err(_) => false,
        }
    }
}
