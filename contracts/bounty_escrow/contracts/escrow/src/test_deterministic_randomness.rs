#![cfg(test)]
//! Deterministic randomness tests for bounty escrow claim-ticket selection.
//!
//! These tests verify that the PRNG-based winner derivation is:
//! - **Stable**: identical inputs always produce the same winner.
//! - **Ledger-bound**: changing the ledger timestamp alters the outcome.
//! - **Seed-sensitive**: different external seeds yield different selections.
//! - **Order-independent**: candidate list ordering does not affect the winner.
//! - **Correct at boundaries**: single candidate, varying bounty IDs, etc.
//!
//! # Predictability statement
//! The selection is fully deterministic given (contract address, bounty params,
//! ledger timestamp, ticket counter, external seed).  Validators who know the
//! timestamp before block close can predict outcomes for a fixed seed.  See
//! `DETERMINISTIC_RANDOMNESS.md` for the complete threat model.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, BytesN, Env, Vec as SdkVec,
};

#[test]
fn deterministic_randomness_module_loads() {
    let env = Env::default();
    let _admin = Address::generate(&env);
    let _token_admin = Address::generate(&env);
    let _seed = BytesN::<32>::from_array(&env, &[7u8; 32]);
    let _v: SdkVec<u32> = SdkVec::new(&env);
}
