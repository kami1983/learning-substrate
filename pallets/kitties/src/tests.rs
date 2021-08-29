use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::{LockIdentifier, WithdrawReasons, ReservableCurrency}};
use serde::de::Unexpected::Option;
use frame_support::traits::LockableCurrency;
use frame_system::{EventRecord, Phase};

#[test]
fn test_create_kitties() {
	new_test_ext().execute_with(|| {
		setup_blocks(162);
		assert_eq!(System::block_number(), 162);
		// Begin testiing, count = 0
		assert_eq!(Kitties::kitties_count(), None);
		// Create a kitty cat.
		assert_ok!(Kitties::create(Origin::signed(1)));
		// Kitties count add 1
		assert_eq!(Kitties::kitties_count(), Some(1));
		// Kitties owner is Origin::signed(1)
		assert_eq!(Kitties::owner(1), Some(1));
		assert!(Kitties::kitties(1).is_some());

		System::assert_has_event(Event::Kitties(crate::Event::<Test>::KittyCreate(1, 1)));

	});
}

#[test]
fn test_transfer_kitties() {
	new_test_ext().execute_with(|| {
		setup_blocks(162);
		assert_eq!(System::block_number(), 162);

		const KITTY_ID_1:u32 = 1 ;
		const ACCOUNT_ID_1:u64 = 1 ;
		const ACCOUNT_ID_2:u64 = 2 ;

		// Stake before.
		assert_eq!(Balances::free_balance(1), 100);
		// Born a cat before test transfer.
		assert_ok!(Kitties::create(Origin::signed(ACCOUNT_ID_1)));
		// Stake after.
		assert_eq!(Balances::free_balance(1), 50);
		// Kitties count add 1
		assert_eq!(Kitties::kitties_count(), Some(1));
		// And owner is 1
		assert_eq!(Kitties::owner(KITTY_ID_1), Some(ACCOUNT_ID_1));
		// Transfer kitties. old owner transfer to new owner
		assert_ok!(Kitties::transfer(Origin::signed(ACCOUNT_ID_1), ACCOUNT_ID_2, KITTY_ID_1));
		// Test kitties owner.
		assert_eq!(Kitties::owner(KITTY_ID_1), Some(ACCOUNT_ID_2));
		// The quantity is still 1 after transmission.
		assert_eq!(Kitties::kitties_count(), Some(1));

		// Sending mutilple times will cause an error because owner is changed.
		assert_noop!(Kitties::transfer(Origin::signed(ACCOUNT_ID_1), ACCOUNT_ID_2, KITTY_ID_1), Error::<Test>::NotOwner);

		// T::AccountId, T::AccountId, T::KittyIndex
		System::assert_has_event(Event::Kitties(crate::Event::<Test>::KittyTransfer(ACCOUNT_ID_1, ACCOUNT_ID_2, KITTY_ID_1)));

	});
}

#[test]
fn test_bread_kitties() {
	new_test_ext().execute_with(|| {
		setup_blocks(162);
		assert_eq!(System::block_number(), 162);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;

		const KITTY_ID_1: u32 = 1;
		const KITTY_ID_2: u32 = 2;
		const KITTY_ID_3: u32 = 3;

		// Create kitties
		assert_ok!(Kitties::create(Origin::signed(ACCOUNT_ID_1)));
		assert_ok!(Kitties::create(Origin::signed(ACCOUNT_ID_2)));
		assert_eq!(Kitties::kitties_count(), Some(2));

		// Test kitty not exists.
		assert_noop!(Kitties::bread(Origin::signed(ACCOUNT_ID_3), KITTY_ID_1, KITTY_ID_3), Error::<Test>::InvalidKittyIndex);

		// Test kitty not exists.
		assert_noop!(Kitties::bread(Origin::signed(ACCOUNT_ID_3), KITTY_ID_3, KITTY_ID_2), Error::<Test>::InvalidKittyIndex);

		// Test kitty not same one.
		assert_noop!(Kitties::bread(Origin::signed(ACCOUNT_ID_3), KITTY_ID_1, KITTY_ID_1), Error::<Test>::SameParentIndex);

		// kitty1 + kitty2 = bron kitty3
		Kitties::bread(Origin::signed(ACCOUNT_ID_3), KITTY_ID_1, KITTY_ID_2);

		// kitty count is 3
		assert_eq!(Kitties::kitties_count(), Some(3));

		// kitty 3 owner is Origin(3)
		assert_eq!(Kitties::owner(KITTY_ID_3), Some(ACCOUNT_ID_3));

		//
		System::assert_has_event(Event::Kitties(crate::Event::<Test>::KittyCreate(ACCOUNT_ID_3, KITTY_ID_3)));

	});
}

// if balance import success this testing will be ok.
#[test]
fn test_balance_total () {
	new_test_ext().execute_with(|| {
		setup_blocks(162);
		//
		assert_eq!(Balances::free_balance(1), 100);
		assert_eq!(Balances::free_balance(2), 200);
		assert_eq!(Balances::free_balance(3), 300);

		// test lock
		// const ID_1: LockIdentifier = *b"1       ";
		// const ID_2: LockIdentifier = *b"2       ";

		// TODO:: can not use this test funciton .
		// <LockableCurrency<Balances>>::set_lock(ID_1, &1, 10, WithdrawReasons::all());
		// // Balances::
		// assert_eq!(Balances::free_balance(1), 90);
	});
}

#[test]
fn test_kitty_sell_and_buy () {
	new_test_ext().execute_with(|| {
		setup_blocks(162);

		const ACCOUNT_ID_1: u64 = 1;
		const ACCOUNT_ID_2: u64 = 2;
		const ACCOUNT_ID_3: u64 = 3;

		const KITTY_ID_1: u32 = 1;
		const KITTY_ID_2: u32 = 2;

		// Stake before.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 100);

		// Account 1 has created tow kitties by himself.
		assert_ok!(Kitties::create(Origin::signed(ACCOUNT_ID_1)));
		assert_ok!(Kitties::create(Origin::signed(ACCOUNT_ID_1)));

		// Stake after.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 0);

		// Check that the owner of these is him
		assert_eq!(Kitties::owner(KITTY_ID_1), Some(ACCOUNT_ID_1));
		assert_eq!(Kitties::owner(KITTY_ID_2), Some(ACCOUNT_ID_1));

		// get ACCOUNT_ID_1 sell list, now is empty.
		let sell_list = Kitties::sell_list(ACCOUNT_ID_1);
		assert_eq!(sell_list.len(), 0 , "sell list is empty.");

		// add KITTY_ID_2 to sell list.
		assert_ok!(Kitties::to_sell(Origin::signed(ACCOUNT_ID_1), KITTY_ID_2, 230));

		System::assert_has_event(Event::Kitties(crate::Event::<Test>::ToSellList(ACCOUNT_ID_1, KITTY_ID_2)));

		// the query owner is still him.
		assert_eq!(Kitties::owner(KITTY_ID_1), Some(ACCOUNT_ID_1));
		assert_eq!(Kitties::owner(KITTY_ID_2), Some(ACCOUNT_ID_1));

		// the query sell list of him, KITTY_ID_2 exists.
		let mut sell_list = Kitties::sell_list(ACCOUNT_ID_1);
		assert_eq!(sell_list.clone().len(), 1 , "KITTY_ID_2 exists. ");
		// let sell_list = Kitties::sell_list(ACCOUNT_ID_1);
		if let Some((sell_kitty_id, sell_balance)) = sell_list.pop() {
			assert_eq!(sell_kitty_id, KITTY_ID_2 );
			assert_eq!(sell_balance, 230 );
		} else {
			assert!(false, "This is impossible.");
		}

		// account_2 want to buy KITTY_ID_1, but KITTY_ID_1 has not sold yet.
		assert_noop!(Kitties::to_buy(Origin::signed(ACCOUNT_ID_2), ACCOUNT_ID_1, KITTY_ID_1), Error::<Test>::KittyHasNotSold);

		// account_2 has 200 yuan, try to buy KITTY_ID_2
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200);
		// But the balance is insufficient
		assert_noop!(Kitties::to_buy(Origin::signed(ACCOUNT_ID_2), ACCOUNT_ID_1, KITTY_ID_2), pallet_balances::Error::<Test>::InsufficientBalance);

		// account_2 want to buy KITTY_ID_1 and he is rich.
		assert_eq!(Balances::free_balance(ACCOUNT_ID_3), 300);
		assert_ok!(Kitties::to_buy(Origin::signed(ACCOUNT_ID_3), ACCOUNT_ID_1, KITTY_ID_2));

		// check balance and kitty's new owner .
		assert_eq!(Kitties::owner(KITTY_ID_2), Some(ACCOUNT_ID_3));

		// Unstake + income = 50 + 230
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), (50 + 230));

		System::assert_has_event(Event::Kitties(crate::Event::<Test>::MakeDeal(ACCOUNT_ID_3, ACCOUNT_ID_1, KITTY_ID_2)));
		System::assert_has_event(Event::Kitties(crate::Event::<Test>::KittyTransfer(ACCOUNT_ID_1, ACCOUNT_ID_3, KITTY_ID_2)));

		// The number of sell lists is empty
		let mut sell_list = Kitties::sell_list(ACCOUNT_ID_1);
		assert_eq!(sell_list.clone().len(), 0 , "Sell list is empty. ");

	});
}

#[test]
fn test_random() {
	new_test_ext().execute_with(|| {
		setup_blocks(162);
		assert_eq!(System::block_number(), 162);
		assert_eq!(RandomnessCollectiveFlip::random_seed(), RandomnessCollectiveFlip::random_seed());
		assert_ne!(RandomnessCollectiveFlip::random(b"random_1"), RandomnessCollectiveFlip::random(b"random_2"));
		let (random, known_since) = RandomnessCollectiveFlip::random_seed();
		assert_eq!(known_since, 162 - RANDOM_MATERIAL_LEN as u64);
		assert_ne!(random, sp_core::H256::zero());
		assert!(!RandomnessCollectiveFlip::random_material().contains(&random));
	});
}

fn setup_blocks(blocks: u64) {
	let mut parent_hash = System::parent_hash();
	for i in 1..(blocks + 1) {
		System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
		RandomnessCollectiveFlip::on_initialize(i);
		let header = System::finalize();
		parent_hash = header.hash();
		System::set_block_number(*header.number());
	}
}

