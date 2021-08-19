use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

use super::*;



// 1. Create proof test case.
#[test]
fn create_clain_works() {
    new_test_ext().execute_with(|| {
        let claim= vec![0,1];
        assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
        assert_eq!(
            Proofs::<Test>::get(&claim),
            Some((1, frame_system::Pallet::<Test>::block_number()))
        );
    })
}

// 2. revoke proof test case.
#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(||{
        let claim = vec![0,1];
        // Create proof value for assert.
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());

        assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
        assert_eq!(Proofs::<Test>::get(&claim), None);
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![0,1];
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofAlreadyExists
        );
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist () {
    new_test_ext().execute_with( || {
        let claim = vec![0,1];
        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    } )
}

// 3. test claim transfer when claim key not exists.
#[test]
fn transfer_claim_when_claim_not_exist () {
    new_test_ext().execute_with(|| {
        let claim = vec![0,1];
        let claim_not_exists = vec![1,2];
        // create proof with claim
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(
            PoeModule::transfer_claim(Origin::signed(1), claim_not_exists.clone(), 3),
            Error::<Test>::ClaimNotExist
        );
    })
}

// 4. test claim transfer when owner not match sender.
#[test]
fn transfer_claim_when_owner_not_match () {
    new_test_ext().execute_with(|| {
        let claim = vec![0,1];
        // create proof with claim
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        assert_noop!(
            // Other owner try to call transfer_claim
            PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 3),
            Error::<Test>::NotClaimOwner
        );
    })
}

// 5. test claim transfer success.
#[test]
fn transfer_claim_is_success () {
    new_test_ext().execute_with(|| {
        let claim = vec![0,1];
        // create proof with claim
        let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
        // call transfer_claim transfer to userid of 3.
        let _ = PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 3);
        // assert userid of 3 is new owner.
        assert_eq!(
            Proofs::<Test>::get(&claim),
            Some((3, frame_system::Pallet::<Test>::block_number()))
        );
    })
}

// 6. Save tool long proof and throw an exception.
#[test]
fn save_tool_long_proof () {
    new_test_ext().execute_with(|| {
        let claim = vec![0,1,2,3,4,5,6,7,8,9,10];
        assert_noop!(
            // Create a proof but it too long.
            PoeModule::create_claim(Origin::signed(1), claim.clone()),
            Error::<Test>::ProofLengthTooLong
        );
    })
}