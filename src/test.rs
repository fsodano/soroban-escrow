use super::{adv_led, incr_led, Contract};
use crate::{
    domain::{Auction, Status},
    get,
    messages::{EscrowMessage, SetupMessage},
    setup,
};

use ed25519_dalek::{Keypair};
use rand::{thread_rng, RngCore};
use soroban_sdk::{testutils::ed25519::Sign, Env, FixedBinary, IntoVal};
use soroban_token_contract::public_types::{KeyedAuthorization, KeyedEd25519Signature};

use soroban_token_contract::testutils::{
    register_test_contract as register_token, to_ed25519, Token,
};

fn generate_contract_id() -> [u8; 32] {
    let mut id: [u8; 32] = Default::default();
    thread_rng().fill_bytes(&mut id);
    id
}

fn generate_keypair() -> Keypair {
    Keypair::generate(&mut thread_rng())
}

fn create_token_contract(e: &Env, admin: &Keypair) -> ([u8; 32], Token) {
    let id = generate_contract_id();
    register_token(&e, &id);
    let token = Token::new(e, &id);
    token.initialize(&to_ed25519(&e, admin), 7, "name", "symbol");
    (id, token)
}

#[test]
fn test_increment_ledger() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);

    let ledger_number = incr_led::invoke(&env, &contract_id);
    assert_eq!(ledger_number, 1);
}

#[test]
fn test_advance_ledger() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);

    let ledger_number = adv_led::invoke(&env, &contract_id, &5);
    assert_eq!(ledger_number, 5);
}

#[test]
#[should_panic(expected = "Auction id 1 already exists")]
fn test_setup_auction() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);

    let auctioneer = generate_keypair();
    let auctioneer_public_key: FixedBinary<32> = FixedBinary::from_array(&env, auctioneer.public.to_bytes());
    let token_admin = generate_keypair();

    let (auctioned_token_contract, _auctioned_token) = create_token_contract(&env, &token_admin);
    let (bid_token_contract, _bid_token) = create_token_contract(&env, &token_admin);

    const RESERVE_AMOUNT: u32 = 100;
    const VALID_AUCTION_ID: u32 = 1;
    let msg = EscrowMessage::Setup(SetupMessage {
        auction_id: VALID_AUCTION_ID,
        auct_token: FixedBinary::from_array(&env, auctioned_token_contract),
        auct_amt: 1,
        bid_token: FixedBinary::from_array(&env, bid_token_contract),
        rsv_amt: RESERVE_AMOUNT,
    });


    let signed_auctioneer_message = KeyedAuthorization::Ed25519(KeyedEd25519Signature {
        public_key: auctioneer_public_key.clone(),
        signature: auctioneer.sign(&msg).unwrap().into_val(&env),
    });

    let auction: Auction = setup::invoke(
        &env,
        &contract_id,
        &VALID_AUCTION_ID,
        &signed_auctioneer_message,
        &msg,
    );

    assert_eq!(auction.auction_id, VALID_AUCTION_ID);
    assert_eq!(auction.status, Status::Open);
    assert_eq!(auction.top_bid, RESERVE_AMOUNT);
    assert_eq!(auction.top_bidder, auctioneer_public_key);


    let persisted_auction: Auction = get::invoke(&env, &contract_id, &VALID_AUCTION_ID);
    assert_eq!(persisted_auction.auction_id, VALID_AUCTION_ID);

    // Test registering the same auction ID fails
    setup::invoke(
        &env,
        &contract_id,
        &VALID_AUCTION_ID,
        &signed_auctioneer_message,
        &msg,
    );
}

fn test_bid_for_item() {
    // bid for an item first time
    // get winner
    // 2nd bidder bids for an item w/low amount -- fails
    // 2nd bidder bids for an item w/high amount -- wins
    // get winner (should be 2), funds returned to user 1
    // 2nd bidder can add more funds to his bid
    // get winner and amount
}

fn test_time_expires() {
    // increment ledger
    // get winner returns final result
    // execute claim (either item owner or winning bidder)
}
