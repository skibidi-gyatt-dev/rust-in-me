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
    pub struct TriviaBase {
        address token;
        address admin;
        address bank_admin;
        address deployer;
        mapping(address => uint256) deposits;
    }
}

sol_interface! {
    interface IERC20 {
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
    }
}

sol! {
    function balanceOf(address account) returns (uint256);
    function transfer(address recipient, uint256 value) returns (bool);
}

#[public]
impl TriviaBase {
    pub fn initialize(
        &mut self,
        _token: Address,
        admin: Address,
        bank_admin: Address,
        deployer: Address,
    ) {
        if Address::from(*self.admin.get()) == Address::default() {
            self.token.set(_token);
            self.admin.set(admin);
            self.bank_admin.set(bank_admin);
            self.deployer.set(deployer);
        }
    }

    pub fn deposit(&mut self, amount: U256) -> bool {
        let sender = self.vm().msg_sender();
        let current = self.deposits.get(sender);
        let ext_balance = Self::token_balance(self, sender);
        if ext_balance < amount {
            return false;
        }

        let success = Self::transfer_from_token(self, sender, contract::address(), amount);
        self.deposits.setter(sender).set(current + amount);
        success
    }

    pub fn get_deposit(&self, user: Address) -> U256 {
        self.deposits.get(user)
    }

    fn contract_balance(&self) -> U256 {
        let contract_address = contract::address();
        let contract_bal = Self::token_balance(&self, contract_address);
        contract_bal
    }

    pub fn emergency_withdraw(&mut self) -> bool {
        let sender = self.vm().msg_sender();
        assert_eq!(
            Address::from(*self.admin.get()),
            sender,
            "Only admin can withdraw!"
        );

        let contract_balance = Self::contract_balance(&self);
        Self::transfer_token(
            self,
            alloy_primitives::Address(*self.admin.get()),
            contract_balance,
        )
    }

    pub fn reward_winners(&mut self, host: Address, winners: Vec<Address>) -> bool {
        let sender = self.vm().msg_sender();
        assert_eq!(
            Address::from(*self.deployer.get()),
            sender,
            "Not deployer address"
        );
        assert_eq!(winners.len(), 3, "Must provide exactly 3 winners");

        let host_allocation = self.deposits.get(host);
        assert!(host_allocation > U256::from(0), "Host has no allocation");

        let reward0 = (host_allocation * U256::from(48)) / U256::from(100);
        let reward1 = (host_allocation * U256::from(29)) / U256::from(100);
        let reward2 = (host_allocation * U256::from(19)) / U256::from(100);
        let admin_reward = (host_allocation * U256::from(4)) / U256::from(100);

        let contract_balance = Self::contract_balance(&self);

        assert!(
            contract_balance >= host_allocation,
            "Insufficient contract balance"
        );

        Self::transfer_token(self, winners[0], reward0);
        Self::transfer_token(self, winners[1], reward1);
        Self::transfer_token(self, winners[2], reward2);

        Self::transfer_token(
            self,
            alloy_primitives::Address(*self.bank_admin.get()),
            admin_reward,
        );

        self.deposits.setter(host).set(U256::from(0));

        true
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

impl TriviaBase {
    //internal func

    fn transfer_from_token(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let token: IERC20 = IERC20::new(alloy_primitives::Address(*self.token.get()));

        token
            .transfer_from(self, from, to, amount)
            .expect("approve token first")
    }
    fn transfer_token(&mut self, to: Address, amount: U256) -> bool {
        let token: IERC20 = IERC20::new(alloy_primitives::Address(*self.token.get()));
        token
            .transfer(self, to, amount)
            .expect("approve token first")
    }
}
