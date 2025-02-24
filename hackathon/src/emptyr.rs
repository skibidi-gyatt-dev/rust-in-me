#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{
    alloy_sol_types::{sol, SolCall},
    contract,
    alloy_primitives::{Address, U256},
    call::RawCall, 
    prelude::*,
    storage::StorageAddress,
};


sol_storage! {
    #[entrypoint]
    pub struct Counter {
        address token;
        address admin;
        mapping(address => uint256) balances;
    }
}

pub struct Worker {
    worker_address:Address,
    amount_paid: U256,
}

sol! {
    
    function balanceOf(address account) returns (uint256);
    function transfer(address recipient, uint256 value) returns (bool);
    event Transfer(address indexed from, address indexed to, uint256 value);
}

#[public]
impl Counter {
    /// (Constructor) Initializes the admin to the caller if not already set.
    pub fn initialize_admin(&mut self) {
        if Address::from(*self.admin.get()) == Address::default() {
            self.admin.set(self.vm().msg_sender());
        }
    }

    /// Allows the current admin to change the admin.
    pub fn change_admin(&mut self, new_admin: Address) {
        assert_eq!(
            Address::from(*self.admin.get()),
            self.vm().msg_sender(),
            "Only admin can change admin"
        );
        self.admin.set(new_admin);
    }

    /// Sets the token address used for deposits.
    pub fn set_address(&mut self, _token: Address) {
        self.token.set(_token);
    }

    pub fn deposit(&mut self, amount: U256) -> bool {
        let employer = self.vm().msg_sender();

        // Check the external token balance of the employer.
        let ext_balance = self.token_balance(employer);
        if ext_balance < amount {
            // Not enough tokens available externally.
            return false;
        }

        // Attempt to transfer tokens from the employer to this contract.
        // This call should use the token's transfer method.
        let success = Self::_perform_transfer(*self.token, contract::address(), amount);
        if success {
            
            let current = self.balances.get(employer);
            self.balances.setter(employer).set(current + amount);
        }
        success
    }

    /// Pay workers from the caller's deposit balance.
    ///
    /// The caller (employer) must have deposited tokens previously.
    /// For each worker in `workers` (a vector of {worker address, amount}), this function:
    ///   1. Checks that the caller's internal balance is at least `amount`.
    ///   2. Deducts `amount` from the caller’s balance.
    ///   3. Transfers `amount` tokens from this contract to the worker.
    ///
    /// If any payment fails (or if the employer does not have enough deposited tokens),
    /// the function will skip that worker.
    pub fn pay_workers(&mut self, workers: Vec<(Address, U256)>) -> bool {
        let employer = self.vm().msg_sender();

        let mut available = self.balances.get(employer);

        for (worker_address, amount) in workers {
           
            if available < amount {
                continue;
            }
            
            let success = Self::_perform_transfer(*self.token, worker_address, amount);
            if success {
              
                available = available - amount;
                self.balances.setter(employer).set(available);
            }
        }
        true
    }

 
    pub fn employer_balance(&self, employer: Address) -> U256 {
        self.balances.get(employer)
    }


    pub fn my_balance(&self) -> U256 {
        self.balances.get(self.vm().msg_sender())
    }

   
    pub fn token_balance(&self, owner: Address) -> U256 {
        let result = RawCall::new_static().call(
            alloy_primitives::Address(*self.token.get()),
            &balanceOfCall { account: owner }.abi_encode(),
        );

        match result {
            Ok(data) => U256::from_be_bytes::<32>(data.try_into().unwrap_or([0u8; 32])),
            Err(_) => U256::from(0), // Returns 0 if the call fails
        }
    }
}

impl Counter {
    fn _perform_transfer(token_addr: Address, recipient: Address, amount: U256) -> bool {
        let call_data = transferCall { recipient, value: amount }.abi_encode();
        let result = RawCall::new().call(token_addr, &call_data);
        match result {
            Ok(data) => data.first().copied() == Some(1),
            Err(_) => false,
        }
    }
}
