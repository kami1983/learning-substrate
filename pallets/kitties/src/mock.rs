pub use crate as pallet_kitties;
use pallet_randomness_collective_flip;
use frame_system as system;

pub use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Header as _, IdentityLookup},
};

pub use frame_support::{
	parameter_types,
	traits::{OnInitialize, Randomness},
};

use frame_system::{limits, Config};


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Kitties: pallet_kitties::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);



parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights
			::simple_max(1024);
	pub BlockLength: limits::BlockLength = limits::BlockLength
			::max(2 * 1024);
	pub const SS58Prefix: u8 = 42;
	pub static ExistentialDeposit: u64 = 0;
	pub const MaxLocks: u32 = 1000;
}


impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::AllowAll;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const Deposit: u64 = 50;
}

impl pallet_kitties::Config for Test {
	type Event = Event;
	type Randomness = RandomnessCollectiveFlip;
	type KittyIndex = u32;
	type Currency = Balances;
	type MaxStakeBalance = Deposit;
	// type MaxStakeBalance = u64;
}



impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// use pallet_kitties::;

pub const RANDOM_MATERIAL_LEN: u32 = 81;
pub use sp_core::H256;
use frame_benchmarking::frame_support::traits::{OnUnbalanced, StoredMap};
use pallet_balances::{NegativeImbalance, WeightInfo, AccountData};
use frame_benchmarking::frame_support::pallet_prelude::{IsType, Member, Get};
use frame_benchmarking::frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use frame_benchmarking::frame_support::Parameter;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test>{
		balances: vec![(1, 100), (2, 200), (3, 300)],
	}.assimilate_storage(&mut t).unwrap();
	t.into()
}
