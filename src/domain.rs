use soroban_sdk::{contracttype, FixedBinary};

#[contracttype]
#[derive(Clone, PartialEq, Eq)]
pub struct Auction {
    pub auction_id: u32,
    pub top_bid: u32,
    pub top_bidder: FixedBinary<32>,
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