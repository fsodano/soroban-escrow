use soroban_sdk::serde::Serialize;
use soroban_sdk::{Account, Env, FixedBinary};
use soroban_token_contract::public_types::{
    KeyedAccountAuthorization, KeyedAuthorization, KeyedEd25519Signature, U256,
};

use super::messages::EscrowMessage;

fn check_ed25519_auth(e: &Env, auth: KeyedEd25519Signature, msg: EscrowMessage) {
    let msg_bin = msg.serialize(e);
    e.verify_sig_ed25519(auth.public_key.into(), msg_bin, auth.signature.into());
}

fn check_account_auth(e: &Env, auth: KeyedAccountAuthorization, msg: EscrowMessage) {
    let acc = Account::from_public_key(&auth.public_key).unwrap();

    let msg_bin = msg.serialize(e);

    let threshold = acc.medium_threshold();
    let mut weight = 0u32;

    let sigs = &auth.signatures;
    let mut prev_pk: Option<U256> = None;
    for sig in sigs.iter().map(Result::unwrap) {
        // Cannot take multiple signatures from the same key
        if let Some(prev) = prev_pk {
            if prev >= sig.public_key {
                panic!("signature out of order")
            }
        }

        e.verify_sig_ed25519(
            sig.public_key.clone().into(),
            msg_bin.clone(),
            sig.signature.into(),
        );
        // TODO: Check for overflow
        weight += acc.signer_weight(&sig.public_key);

        prev_pk = Some(sig.public_key);
    }

    if weight < threshold {
        panic!("insufficient signing weight")
    }
}

pub fn check_auth(e: &Env, auth: KeyedAuthorization, msg: EscrowMessage) {
    match auth {
        KeyedAuthorization::Contract => {
            e.get_invoking_contract();
        }
        KeyedAuthorization::Ed25519(kea) => check_ed25519_auth(e, kea, msg),
        KeyedAuthorization::Account(kaa) => check_account_auth(e, kaa, msg),
    }
}

pub trait PublicKeyTrait {
    fn get_public_key(&self, env: &Env) -> FixedBinary<32>;
}

impl PublicKeyTrait for KeyedAuthorization {
    fn get_public_key(&self, env: &Env) -> FixedBinary<32> {
        match self {
            KeyedAuthorization::Contract => env.get_invoking_contract().clone(),
            KeyedAuthorization::Ed25519(kea) => kea.public_key.clone(),
            KeyedAuthorization::Account(kaa) => kaa.public_key.clone(),
        }
    }
}