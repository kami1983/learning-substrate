// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::*;
use crate as demo_offchain_worker;
use std::sync::Arc;
use sp_arithmetic::per_things::Permill;

// use codec::Decode;
use frame_support::{assert_ok, parameter_types};
use sp_core::{
	H256,
	offchain::{OffchainWorkerExt, TransactionPoolExt, testing},
	sr25519::Signature,
};

use sp_keystore::{
	{KeystoreExt, SyncCryptoStore},
	testing::KeyStore,
};

use sp_runtime::{
	RuntimeAppPublic,
	testing::{Header, TestXt},
	traits::{
		BlakeTwo256, IdentityLookup, Extrinsic as ExtrinsicT,
		IdentifyAccount, Verify,
	},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// For testing the module, we construct a mock runtime.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		DemoOcw: demo_offchain_worker::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
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
	type AccountId = sp_core::sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

type Extrinsic = TestXt<Call, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test where
	Call: From<LocalCall>,
{
	type OverarchingCall = Call;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(Call, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

parameter_types! {
	pub const GracePeriod: u64 = 5;
	pub const UnsignedInterval: u64 = 128;
	pub const UnsignedPriority: u64 = 1 << 20;
}

impl Config for Test {
	type Event = Event;
	type AuthorityId = crypto::TestAuthId;
	type Call = Call;
}

#[test]
fn test_submit_number_signed() {
	// create keystore for sign transaction.
	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";

	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let keystore = KeyStore::new();
	SyncCryptoStore::sr25519_generate_new(
		&keystore,
		crate::crypto::Public::ID,
		Some(&format!("{}/hunter1", PHRASE))
	).unwrap();


	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	//
	t.execute_with(|| {
		let block_number: u64 = 51;

		// this function will be called by offchain_worker.
		DemoOcw::offchain_signed_tx(block_number).unwrap();

		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature.unwrap().0, 0);
		//
		assert_eq!(tx.call, Call::DemoOcw(crate::Call::submit_number_signed(block_number)));
	});
}

#[test]
fn test_submit_number_unsigned() {
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let keystore = KeyStore::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	t.execute_with(|| {
		// when
		DemoOcw::offchain_unsigned_tx(1).unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		assert!(pool_state.read().transactions.is_empty());
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		assert_eq!(tx.call, Call::DemoOcw(crate::Call::submit_number_unsigned(1)));
	});
}

// Test price extract, will decode json to (u64, Permill) struct
#[test]
fn parse_price_works() {
	assert_eq!((27, Permill::from_parts(688407)), DemoOcw::parse_price_of_dot(get_dot_json()))
}

// Test fetch price (fetch_price_info)
#[test]
fn test_fetch_price_by_http_request() {

	let (offchain, state) = testing::TestOffchainExt::new();
	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));

	price_http_response(&mut state.write());

	t.execute_with(|| {
		// when
		let body = DemoOcw::fetch_dot_price_by_http().unwrap();
		let body_str = sp_std::str::from_utf8(&body);
		// println!("body_str = {:?}", body_str);
		// then body has been obtained.
		assert_eq!(body_str, Ok(get_dot_json()));
	});
}

#[test]
fn test_price_data_on_the_chain() {
	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let keystore = KeyStore::new();
	SyncCryptoStore::sr25519_generate_new(
		&keystore,
		crate::crypto::Public::ID,
		Some(&format!("{}/hunter1", PHRASE))
	).unwrap();

	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
		.get(0)
		.unwrap()
		.clone();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	price_http_response(&mut offchain_state.write());

	let price_payload = PricePayload {
		price_cell: (27, Permill::from_parts(688407)),
		public: <Test as SigningTypes>::Public::from(public_key),
	};

	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
	t.execute_with(|| {
		// when
		DemoOcw::fetch_price_info().unwrap();
		// then
		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature, None);
		if let Call::DemoOcw(crate::Call::submit_price_unsigned_with_signed_payload(body, signature)) = tx.call {
			// assert payload struct is what we want.
			assert_eq!(body, price_payload);
			// assert signature is right.
			let signature_valid = <PricePayload<
				<Test as SigningTypes>::Public
			> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&price_payload, signature);
			// be fine.
			assert!(signature_valid);
		}
	});
}

#[test]
fn test_10_price_stored_cycles() {
	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
	let (offchain, offchain_state) = testing::TestOffchainExt::new();
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
	let keystore = KeyStore::new();
	SyncCryptoStore::sr25519_generate_new(
		&keystore,
		crate::crypto::Public::ID,
		Some(&format!("{}/hunter1", PHRASE))
	).unwrap();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt(Arc::new(keystore)));

	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
	t.execute_with(|| {

		// when insert 11 price
		DemoOcw::append_dot_price_by_round((1,Permill::from_parts(1)));
		DemoOcw::append_dot_price_by_round((2,Permill::from_parts(2)));
		DemoOcw::append_dot_price_by_round((3,Permill::from_parts(3)));
		DemoOcw::append_dot_price_by_round((4,Permill::from_parts(4)));
		DemoOcw::append_dot_price_by_round((5,Permill::from_parts(5)));
		DemoOcw::append_dot_price_by_round((6,Permill::from_parts(6)));
		DemoOcw::append_dot_price_by_round((7,Permill::from_parts(7)));
		DemoOcw::append_dot_price_by_round((8,Permill::from_parts(8)));
		DemoOcw::append_dot_price_by_round((9,Permill::from_parts(9)));
		DemoOcw::append_dot_price_by_round((10,Permill::from_parts(10)));
		DemoOcw::append_dot_price_by_round((11,Permill::from_parts(11)));

		// get price queue list.
		let mut price_deque = DemoOcw::prices();
		assert_eq!(price_deque.len(), 10, "10 store loops, so there are only 10 elements at most.");

		// The for loop starts at 2 because 1 has been kicked out.
		for i in 2..12u64 {
			let assert_price = (i,Permill::from_parts(i.try_into().unwrap()));
			// assert_eq!(assert_price, price_deque[(i-2) as usize], "");
			assert_eq!(assert_price, price_deque.pop_front().unwrap(),"The element of the first element starts from 2.");
		}

		// After the assertion, the circular list is empty and all assertions are made.
		assert_eq!(price_deque.len(), 0, "All assertions are made.");

	});
}


fn price_http_response(state: &mut testing::OffchainState) {
	let headers:Vec<(String, String)> = vec![("User-Agent".to_string(), HTTP_DOT_PRICE_HEADER_USER_AGENT.to_string())];
	state.expect_request(testing::PendingRequest {
		method: "GET".into(),
		headers,
		uri: "https://api.coincap.io/v2/assets/polkadot".into(),
		response: Some(get_dot_json().as_bytes().to_vec()),
		sent: true,
		..Default::default()
	});
}

// Mock dot return data.
fn get_dot_json() -> &'static str {
	"{\"data\":{\"id\":\"polkadot\",\"rank\":\"10\",\"symbol\":\"DOT\",\"name\":\"Polkadot\",\"supply\":\"1028068538.5689500000000000\",\"maxSupply\":null,\"marketCapUsd\":\"28465580367.6970634793593043\",\"volumeUsd24Hr\":\"1346610126.5750374659937759\",\"priceUsd\":\"27.6884072411364311\",\"changePercent24Hr\":\"0.5268368449774714\",\"vwap24Hr\":\"27.6695623833381634\",\"explorer\":\"https://polkascan.io/polkadot\"},\"timestamp\":1631146070953}"
}

fn setup_blocks(blocks: u64) {
	let mut parent_hash = System::parent_hash();
	for i in 1..(blocks + 1) {
		System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
		let header = System::finalize();
		parent_hash = header.hash();
	}
}
