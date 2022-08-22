use soroban_sdk::contracttype;
use soroban_token_contract::public_types::U256;

#[derive(Clone)]
#[contracttype]
pub enum EscrowMessage{
    Setup(SetupMessage),
    Bid(BidMessage),
    Claim(ClaimMessage),
}

#[derive(Clone)]
#[contracttype]
pub struct SetupMessage {
    pub auction_id: u32,
    pub auct_token: U256,
    pub auct_amt: u32,
    pub bid_token: U256,
    pub rsv_amt: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct BidMessage {
    pub auction_id: u32,
    pub amount: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct ClaimMessage {
    pub auction_id: u32,
}
