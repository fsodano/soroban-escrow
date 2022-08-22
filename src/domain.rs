use soroban_sdk::{contracttype, FixedBinary};
use soroban_token_contract::public_types::U256;

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Auction {
    pub auction_id: u32,
    pub top_bid: u32,
    pub top_bidder: FixedBinary<32>,
    pub auct_token: U256,
    pub auct_amt: u32,
    pub bid_token: U256,
    pub status: Status,
}

#[contracttype]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Open,
    Closed,
    ClsdNoBid,
    Claimed
}