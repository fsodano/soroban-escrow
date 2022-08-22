use super::{adv_led, incr_led, Contract};
use crate::{
    bid,
    domain::{Auction, Status},
    get,
    messages::{BidMessage, EscrowMessage, SetupMessage},
    setup,
};

use ed25519_dalek::Keypair;
use rand::{thread_rng, RngCore};
use soroban_sdk::{testutils::ed25519::Sign, BigInt, Env, FixedBinary, IntoVal};
use soroban_token_contract::public_types::{Identifier, KeyedAuthorization, KeyedEd25519Signature};

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
#[should_panic(expected = "Auction id 1 does not exist")]
fn test_get_auction_fails() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);

    const INVALID_AUCTION_ID: u32 = 1;
    get::invoke(&env, &contract_id, &INVALID_AUCTION_ID);
}

fn setup_auction(env: &Env, id: u32, contract_id: FixedBinary<32>) -> Auction {
    let escrow_id = Identifier::Contract(contract_id.clone());

    let auctioneer = generate_keypair();
    let auctioneer_public_key: FixedBinary<32> =
        FixedBinary::from_array(env, auctioneer.public.to_bytes());
    let auctioneer_id = to_ed25519(&env, &auctioneer);
    let token_admin = generate_keypair();

    let (auctioned_token_contract, auctioned_token) = create_token_contract(env, &token_admin);
    let (bid_token_contract, _bid_token) = create_token_contract(env, &token_admin);

    auctioned_token.mint(&token_admin, &auctioneer_id, &BigInt::from_u32(env, 1));
    auctioned_token.xfer(&auctioneer, &escrow_id, &BigInt::from_u32(env, 1));

    const RESERVE_AMOUNT: u32 = 100;
    let msg = EscrowMessage::Setup(SetupMessage {
        auction_id: id,
        auct_token: FixedBinary::from_array(env, auctioned_token_contract),
        auct_amt: 1,
        bid_token: FixedBinary::from_array(env, bid_token_contract),
        rsv_amt: RESERVE_AMOUNT,
    });

    let signed_auctioneer_message = KeyedAuthorization::Ed25519(KeyedEd25519Signature {
        public_key: auctioneer_public_key.clone(),
        signature: auctioneer.sign(&msg).unwrap().into_val(env),
    });

    let auction: Auction = setup::invoke(env, &contract_id, &id, &signed_auctioneer_message, &msg);

    auction
}

#[test]
#[should_panic(expected = "You must transfer 1 auction tokens to this contract first")]
fn test_duplicate_auction_id_fails() {
    let env = &Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);
    const VALID_AUCTION_ID: u32 = 1;

    let auctioneer = generate_keypair();
    let auctioneer_public_key: FixedBinary<32> = FixedBinary::from_array(env, auctioneer.public.to_bytes());
    let auctioneer_id = to_ed25519(&env, &auctioneer);
    let token_admin = generate_keypair();

    let (auctioned_token_contract, auctioned_token) = create_token_contract(env, &token_admin);
    let (bid_token_contract, _bid_token) = create_token_contract(env, &token_admin);

    // token is minted but not transferred
    auctioned_token.mint(&token_admin, &auctioneer_id, &BigInt::from_u32(env, 1));

    const RESERVE_AMOUNT: u32 = 100;
    let msg = EscrowMessage::Setup(SetupMessage {
        auction_id: VALID_AUCTION_ID,
        auct_token: FixedBinary::from_array(env, auctioned_token_contract),
        auct_amt: 1,
        bid_token: FixedBinary::from_array(env, bid_token_contract),
        rsv_amt: RESERVE_AMOUNT,
    });

    let signed_auctioneer_message = KeyedAuthorization::Ed25519(KeyedEd25519Signature {
        public_key: auctioneer_public_key.clone(),
        signature: auctioneer.sign(&msg).unwrap().into_val(env),
    });

    setup::invoke(env, &contract_id, &VALID_AUCTION_ID, &signed_auctioneer_message, &msg);
}

#[test]
#[should_panic(expected = "Auction id 1 already exists")]
fn test_setup_fails_if_auction_token_not_transfered() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);
    const VALID_AUCTION_ID: u32 = 1;
    setup_auction(&env, VALID_AUCTION_ID, contract_id.clone());
    setup_auction(&env, VALID_AUCTION_ID, contract_id.clone());
}


#[test]
fn test_bid_for_item() {
    let env = Env::default();
    let contract_id = FixedBinary::from_array(&env, [0; 32]);
    env.register_contract(&contract_id, Contract);

    let auctioneer = generate_keypair();
    let auctioneer_public_key: FixedBinary<32> =
        FixedBinary::from_array(&env, auctioneer.public.to_bytes());
    let auctioneer_id = to_ed25519(&env, &auctioneer);
    let bidder: Keypair = generate_keypair();
    let bidder_public_key: FixedBinary<32> =
        FixedBinary::from_array(&env, bidder.public.to_bytes());
    let bidder_id = to_ed25519(&env, &bidder);
    let token_admin = generate_keypair();

    let (auctioned_token_contract, auctioned_token) = create_token_contract(&env, &token_admin);
    let (bid_token_contract, bid_token) = create_token_contract(&env, &token_admin);

    auctioned_token.mint(&token_admin, &auctioneer_id, &BigInt::from_u32(&env, 1000));
    assert_eq!(
        auctioned_token.balance(&auctioneer_id),
        BigInt::from_u32(&env, 1000)
    );

    bid_token.mint(&token_admin, &bidder_id, &BigInt::from_u32(&env, 1000));
    assert_eq!(
        auctioned_token.balance(&bidder_id),
        BigInt::from_u32(&env, 1000)
    );

    const RESERVE_AMOUNT: u32 = 100;
    const VALID_AUCTION_ID: u32 = 1;
    let setup_msg = EscrowMessage::Setup(SetupMessage {
        auction_id: VALID_AUCTION_ID,
        auct_token: FixedBinary::from_array(&env, auctioned_token_contract),
        auct_amt: 1,
        bid_token: FixedBinary::from_array(&env, bid_token_contract),
        rsv_amt: RESERVE_AMOUNT,
    });

    let signed_auctioneer_message = KeyedAuthorization::Ed25519(KeyedEd25519Signature {
        public_key: auctioneer_public_key.clone(),
        signature: auctioneer.sign(&setup_msg).unwrap().into_val(&env),
    });

    let auction: Auction = setup::invoke(
        &env,
        &contract_id,
        &VALID_AUCTION_ID,
        &signed_auctioneer_message,
        &setup_msg,
    );

    let bid_msg = EscrowMessage::Bid(BidMessage {
        auction_id: VALID_AUCTION_ID,
        amount: RESERVE_AMOUNT + 1,
    });

    let signed_bid_message = KeyedAuthorization::Ed25519(KeyedEd25519Signature {
        public_key: auctioneer_public_key.clone(),
        signature: auctioneer.sign(&bid_msg).unwrap().into_val(&env),
    });

    let persisted_auction: Auction = get::invoke(&env, &contract_id, &VALID_AUCTION_ID);
    assert_eq!(persisted_auction.auction_id, VALID_AUCTION_ID);

    // bid for an item below reserve prices (should fail)
    // bid for an item above reserve price for the first time
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

fn test_no_winner() {}
