#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;

use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolCall},
    call,
    call::RawCall,
    contract,
    prelude::*,
    storage::StorageAddress,
};


sol_storage! {
    #[entrypoint]
    pub struct EmployerPool {
        address token;
        address admin;
        mapping(address => uint256) balances;
    }
}

sol_interface! {
    interface ERC20 {
    function balanceOf(address account) external view returns (uint256);
    function transfer(address recipient, uint256 amount)
        external
        returns (bool);
    function transferFrom(address sender, address recipient, uint256 amount)
        external
        returns (bool);
    }
}

pub struct Worker {
    worker_address: Address,
    amount_paid: U256,
}

sol! {

    function balanceOf(address account) returns (uint256);
    function transfer(address recipient, uint256 value) returns (bool);
    event Transfer(address indexed from, address indexed to, uint256 value);
}

#[public]
impl EmployerPool {
    /// (Constructor) Initializes the admin to the caller if not already set.
    pub fn initialize_admin(&mut self) {
        if Address::from(*self.admin.get()) == Address::default() {
            self.admin.set(self.vm().msg_sender());
        }
    }
    pub fn set_address(&mut self, _token: Address) {
        self.token.set(_token);
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

    pub fn deposit(&mut self, amount: U256) -> bool {
        let employer = self.vm().msg_sender();
        let current = self.balances.get(employer);

        // Check the external token balance of the employer.
        let ext_balance = self.token_balance(employer);
        if ext_balance < amount {
            return false;
        }

        let success = Self::transfer_from_token(self, employer, contract::address(), amount);
        
        self.balances.setter(employer).set(current + amount);
        success
    }

    pub fn pay_workers(&mut self, workers: Vec<(Address, U256)>) -> bool {
        let employer = self.vm().msg_sender();
        let mut bal = Self::employer_balance(&self, employer);

        for (worker_address, amount) in workers {
            if bal < amount {
                continue;
            }
            let success = Self::transfer_token(self, worker_address, amount);
            bal = bal - amount;
            self.balances.setter(employer).set(bal);
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

impl EmployerPool {

    //internal functions

    fn transfer_from_token(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let token: ERC20 = ERC20::new(alloy_primitives::Address(*self.token.get()));

        token
            .transfer_from(self, from, to, amount)
            .expect("approve token first")
    }
    fn transfer_token(&mut self, to: Address, amount: U256) -> bool {
        let token: ERC20 = ERC20::new(alloy_primitives::Address(*self.token.get()));
        token
            .transfer(self, to, amount)
            .expect("approve token first")
    } //contract balance
    fn balance(&self) -> U256{
        let token: ERC20 = ERC20::new(alloy_primitives::Address(*self.token.get()));
        token.balance()
    }
}
