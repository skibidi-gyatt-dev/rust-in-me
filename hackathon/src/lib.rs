#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
extern crate alloc;


use stylus_sdk::{
    alloy_primitives::{Address, U256},
    alloy_sol_types::{sol, SolCall},
    call::RawCall,
    call,
    contract,
    prelude::*,
    storage::StorageAddress,
};

const TRANSFER_FROM_SELECTOR: [u8; 4] = [0x23, 0xb8, 0x72, 0xdd];

sol_storage! {
    #[entrypoint]
    pub struct Counter {
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
impl Counter {
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

    // # transfer test

    fn transfer_usdc(&mut self, from: Address, to: Address, amount: U256) -> bool {
        let usdc: ERC20 = ERC20::new(alloy_primitives::Address(*self.token.get()));

        usdc.transfer_from(self,from, to, amount).expect("REASON")
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

    pub fn _perform_transfer(token_addr: Address, recipient: Address, amount: U256) -> bool {
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

   /*  fn transfer_tokens(&self) {
        let sender = self.vm().msg_sender();
        let contract_address = contract::address();
        //let cost = self.increment_cost.get();
        let token_addr = self.token.get();

        let mut calldata = Vec::new();
        calldata.extend_from_slice(&TRANSFER_FROM_SELECTOR);
        
        let mut from_padded = [0u8; 32];
        from_padded[12..].copy_from_slice(&sender.to_vec());
        calldata.extend_from_slice(&from_padded);
        
        let mut to_padded = [0u8; 32];
        to_padded[12..].copy_from_slice(&contract_address.to_vec());
        calldata.extend_from_slice(&to_padded);
        
        //calldata.extend_from_slice(&cost.to_be_bytes());

        let result = call::call(token_addr, &calldata).expect("Token call failed");
        
        if result.len() != 32 || result[31] != 1 {
            panic!("Token transfer failed");
        }
    } */
    


    
}



/* impl Counter {
    fn _perform_transfer(token_addr: Address, recipient: Address, amount: U256) -> bool {
        let call_data = transferCall { recipient, value: amount }.abi_encode();
        let result = RawCall::new().call(token_addr, &call_data);
        match result {
            Ok(data) => data.first().copied() == Some(1),
            Err(_) => false,
        }
    }
} */

