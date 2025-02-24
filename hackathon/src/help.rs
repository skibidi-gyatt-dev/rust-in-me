use stylus_sdk::prelude::*;
use stylus_sdk::storage::{StorageUint64, StorageAddress, StorageU256};
use stylus_sdk::msg;
use stylus_sdk::call;
use stylus_sdk::alloy_primitives::{Address, U256};
use stylus_sdk::contract;

const TRANSFER_FROM_SELECTOR: [u8; 4] = [0x23, 0xb8, 0x72, 0xdd];

pub struct Counter {
    count: StorageUint64,
    token_address: StorageAddress,
    increment_cost: StorageU256,
}

#[contract]
impl Counter {
    #[constructor]
    pub fn new(token_address: Address, increment_cost: U256) {
        self.token_address.set(token_address);
        self.increment_cost.set(increment_cost);
    }

    fn transfer_tokens(&self) {
        let sender = msg::sender();
        let contract_address = msg::address();
        let cost = self.increment_cost.get();
        let token_addr = self.token_address.get();

        let mut calldata = Vec::new();
        calldata.extend_from_slice(&TRANSFER_FROM_SELECTOR);
        
        let mut from_padded = [0u8; 32];
        from_padded[12..].copy_from_slice(&sender.to_vec());
        calldata.extend_from_slice(&from_padded);
        
        let mut to_padded = [0u8; 32];
        to_padded[12..].copy_from_slice(&contract_address.to_vec());
        calldata.extend_from_slice(&to_padded);
        
        calldata.extend_from_slice(&cost.to_be_bytes());

        let result = call::call(token_addr, &calldata).expect("Token call failed");
        
        if result.len() != 32 || result[31] != 1 {
            panic!("Token transfer failed");
        }
    }

    #[public]
    pub fn increment(&mut self) {
        self.transfer_tokens();
        self.count.set(self.count.get() + 1);
    }

    #[public]
    pub fn decrement(&mut self) {
        let current = self.count.get();
        if current > 0 {
            self.count.set(current - 1);
        }
    }

    #[public]
    pub fn get_count(&self) -> u64 {
        self.count.get()
    }
}



