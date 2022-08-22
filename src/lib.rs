#![no_std]
pub mod auth;
pub mod domain;
pub mod messages;
#[cfg(test)]
pub mod test;

extern crate alloc;

use core::panic;

use alloc::string::ToString;
use auth::{check_auth, PublicKeyTrait};
use domain::{Auction, Status};
use messages::EscrowMessage;
use soroban_sdk::{contractimpl, Env, Symbol, BigInt};
use soroban_token_contract as token;
use token::public_types::{KeyedAuthorization, U256, Identifier};

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

        let auctioned_token_balance = token::balance(&env, &setup_msg.auct_token, &Identifier::Contract(env.get_current_contract()));
        if auctioned_token_balance != BigInt::from_u32(&env, setup_msg.auct_amt) {
            panic!("You must transfer {} auction tokens to this contract first", setup_msg.auct_amt.to_string())
        }

        let auctioneer_public_key = auctioneer_auth.get_public_key(&env);

        let auction = Auction {
            auction_id,
            top_bid: setup_msg.rsv_amt,
            top_bidder: auctioneer_public_key,
            status: domain::Status::Open,
            auct_token: setup_msg.auct_token,
            auct_amt: setup_msg.auct_amt,
            bid_token: setup_msg.bid_token
        };

        env.contract_data()
            .set(setup_msg.auction_id, auction.clone());

        auction
    }

    pub fn bid(env: Env, bidder_auth: KeyedAuthorization, msg: EscrowMessage) -> Auction {
        let bid_msg = match &msg {
            EscrowMessage::Bid(bid) => bid.clone(),
            _ => panic!("Incorrect message type"),
        };

        check_auth(&env, bidder_auth.clone(), msg.clone());
        let bidder_public_key = bidder_auth.get_public_key(&env);

        let mut auction = get_auction(&env, bid_msg.auction_id);
        let top_bid = auction.top_bid;
        let new_bid = bid_msg.amount;

        match auction.status {
            Status::Open => {
                if new_bid > top_bid {
                    auction.top_bidder = bidder_public_key;
                    auction.top_bid = new_bid;
                }else{
                    panic!("The current top bid {} is higher than the new offer {}", top_bid.to_string(), new_bid.to_string())
                }      
            },
            _ => panic!("This auction is already closed"),
        };

        env.contract_data()
            .set(bid_msg.auction_id, auction.clone());

        auction
    }

    pub fn claim(env: Env, auction_id: u32, claimant: KeyedAuthorization) {}

    pub fn get(env: Env, auction_id: u32) -> Auction {
        get_auction(&env, auction_id)
    }
}

