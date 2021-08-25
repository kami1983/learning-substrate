use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_create_kitties() {
	new_test_ext().execute_with(|| {

		setup_blocks(162);
		assert_eq!(System::block_number(), 162);

		// Begin testiing, count = 0
		assert_eq!(Kitties::kitties_count(), None);
		// Create a kitty cat.
		assert_ok!(Kitties::create(Origin::signed(1)));

		// assert_eq!(Kitties::kitties(1), 0);
		// Dispatch a signed extrinsic.
		// assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
		// // Read pallet storage and assert an expected result.
		// assert_eq!(TemplateModule::something(), Some(42));
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