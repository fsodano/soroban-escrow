#![no_std]
pub mod auth;
pub mod domain;
pub mod messages;
#[cfg(test)]
pub mod test;

extern crate alloc;

use alloc::string::ToString;
use auth::{check_auth, PublicKeyTrait};
use domain::Auction;
use messages::EscrowMessage;
use soroban_sdk::{contractimpl, Env, Symbol};
use soroban_token_contract as token;
use token::public_types::{KeyedAuthorization, U256};

pub struct Contract;

const LEDGER_NUMBER: Symbol = Symbol::from_str("LEDGERNUM");

pub fn get_auction(env: &Env, auction_id: u32) -> Auction {
    if !env.contract_data().has(auction_id) {
        panic!("Auction id {} does not exist", auction_id.to_string())
    }

    let game: Auction = env.contract_data().get(auction_id).unwrap().unwrap();
    game
}

#[contractimpl(export_if = "export")]
impl Contract {
    pub fn incr_led(env: Env) -> u32 {
        let mut ledger_number: u32 = env
            .contract_data()
            .get(LEDGER_NUMBER)
            .unwrap_or(Ok(0))
            .unwrap();

        ledger_number += 1;
        env.contract_data().set(LEDGER_NUMBER, ledger_number);

        ledger_number
    }

    pub fn adv_led(env: Env, number: u32) -> u32 {
        let mut ledger_number: u32 = env
            .contract_data()
            .get(LEDGER_NUMBER)
            .unwrap_or(Ok(0))
            .unwrap();

        ledger_number += number;
        env.contract_data().set(LEDGER_NUMBER, ledger_number);

        ledger_number
    }

    pub fn setup(
        env: Env,
        auction_id: u32,
        auctioneer_auth: KeyedAuthorization,
        msg: EscrowMessage,
    ) -> Auction {
        let setup_msg = match &msg {
            EscrowMessage::Setup(setup) => setup.clone(),
            _ => panic!("Incorrect message type"),
        };

        check_auth(&env, auctioneer_auth.clone(), msg.clone());

        if env.contract_data().has(setup_msg.auction_id) {
            panic!("Auction id {} already exists", setup_msg.auction_id);
        }

        let auctioneer_public_key = auctioneer_auth.get_public_key(&env);

        let auction = Auction {
            auction_id,
            top_bid: setup_msg.rsv_amt,
            top_bidder: auctioneer_public_key,
            status: domain::Status::Open,
        };

        env.contract_data()
            .set(setup_msg.auction_id, auction.clone());

        auction
    }

    pub fn bid(env: Env, auction_id: u32, bidder: KeyedAuthorization, token: U256, amount: u32) {}

    pub fn claim(env: Env, auction_id: u32, claimant: KeyedAuthorization) {}

    pub fn get(env: Env, auction_id: u32) -> Auction {
        get_auction(&env, auction_id)
    }
}

