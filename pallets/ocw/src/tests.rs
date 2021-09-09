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

	// price_oracle_response(&mut offchain_state.write());
	//
	t.execute_with(|| {
		let block_number: u64 = 51;
		// set block number, try to call submit_number_signed on offchain worker
		// setup_blocks(block_number);
		// assert_eq!(System::block_number(), block_number);

		// Call submit_number_signed submit current block_number on chain.
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

	// price_oracle_response(&mut offchain_state.write());

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

// Mock dot return data.
fn get_dot_json() -> &'static str {
	"{\"data\":{\"id\":\"polkadot\",\"rank\":\"10\",\"symbol\":\"DOT\",\"name\":\"Polkadot\",\"supply\":\"1028068538.5689500000000000\",\"maxSupply\":null,\"marketCapUsd\":\"28465580367.6970634793593043\",\"volumeUsd24Hr\":\"1346610126.5750374659937759\",\"priceUsd\":\"27.6884072411364311\",\"changePercent24Hr\":\"0.5268368449774714\",\"vwap24Hr\":\"27.6695623833381634\",\"explorer\":\"https://polkascan.io/polkadot\"},\"timestamp\":1631146070953}"
}

// Test price extract.
#[test]
fn parse_price_works() {
	assert_eq!((27, Permill::from_parts(688407)), DemoOcw::parse_price_of_dot(get_dot_json()))
}

// #[test]
// fn it_aggregates_the_price() {
// 	sp_io::TestExternalities::default().execute_with(|| {
// 		assert_eq!(Example::average_price(), None);
//
// 		assert_ok!(Example::submit_price(Origin::signed(Default::default()), 27));
// 		assert_eq!(Example::average_price(), Some(27));
//
// 		assert_ok!(Example::submit_price(Origin::signed(Default::default()), 43));
// 		assert_eq!(Example::average_price(), Some(35));
// 	});
// }

// #[test]
// fn should_make_http_call_and_parse_result() {
// 	let (offchain, state) = testing::TestOffchainExt::new();
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
//
// 	price_oracle_response(&mut state.write());
//
// 	t.execute_with(|| {
// 		// when
// 		let price = Example::fetch_price().unwrap();
// 		// then
// 		assert_eq!(price, 15523);
// 	});
// }

// #[test]
// fn knows_how_to_mock_several_http_calls() {
// 	let (offchain, state) = testing::TestOffchainExt::new();
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
//
// 	{
// 		let mut state = state.write();
// 		state.expect_request(testing::PendingRequest {
// 			method: "GET".into(),
// 			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
// 			response: Some(br#"{"USD": 1}"#.to_vec()),
// 			sent: true,
// 			..Default::default()
// 		});
//
// 		state.expect_request(testing::PendingRequest {
// 			method: "GET".into(),
// 			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
// 			response: Some(br#"{"USD": 2}"#.to_vec()),
// 			sent: true,
// 			..Default::default()
// 		});
//
// 		state.expect_request(testing::PendingRequest {
// 			method: "GET".into(),
// 			uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
// 			response: Some(br#"{"USD": 3}"#.to_vec()),
// 			sent: true,
// 			..Default::default()
// 		});
// 	}
//
//
// 	t.execute_with(|| {
// 		let price1 = Example::fetch_price().unwrap();
// 		let price2 = Example::fetch_price().unwrap();
// 		let price3 = Example::fetch_price().unwrap();
//
// 		assert_eq!(price1, 100);
// 		assert_eq!(price2, 200);
// 		assert_eq!(price3, 300);
// 	})
//
// }

// #[test]
// fn should_submit_signed_transaction_on_chain() {
// 	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
//
// 	let (offchain, offchain_state) = testing::TestOffchainExt::new();
// 	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
// 	let keystore = KeyStore::new();
// 	SyncCryptoStore::sr25519_generate_new(
// 		&keystore,
// 		crate::crypto::Public::ID,
// 		Some(&format!("{}/hunter1", PHRASE))
// 	).unwrap();
//
//
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
// 	t.register_extension(TransactionPoolExt::new(pool));
// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
//
// 	price_oracle_response(&mut offchain_state.write());
//
// 	t.execute_with(|| {
// 		// when
// 		Example::fetch_price_and_send_signed().unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		assert!(pool_state.read().transactions.is_empty());
// 		let tx = Extrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature.unwrap().0, 0);
// 		assert_eq!(tx.call, Call::Example(crate::Call::submit_price(15523)));
// 	});
// }

// #[test]
// fn should_submit_unsigned_transaction_on_chain_for_any_account() {
// 	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
// 	let (offchain, offchain_state) = testing::TestOffchainExt::new();
// 	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
//
// 	let keystore = KeyStore::new();
//
// 	SyncCryptoStore::sr25519_generate_new(
// 		&keystore,
// 		crate::crypto::Public::ID,
// 		Some(&format!("{}/hunter1", PHRASE))
// 	).unwrap();
//
// 	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
// 		.get(0)
// 		.unwrap()
// 		.clone();
//
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
// 	t.register_extension(TransactionPoolExt::new(pool));
// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
//
// 	price_oracle_response(&mut offchain_state.write());
//
// 	let price_payload = PricePayload {
// 		block_number: 1,
// 		price: 15523,
// 		public: <Test as SigningTypes>::Public::from(public_key),
// 	};
//
// 	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
// 	t.execute_with(|| {
// 		// when
// 		Example::fetch_price_and_send_unsigned_for_any_account(1).unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		let tx = Extrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature, None);
// 		if let Call::Example(crate::Call::submit_price_unsigned_with_signed_payload(body, signature)) = tx.call {
// 			assert_eq!(body, price_payload);
//
// 			let signature_valid = <PricePayload<
// 				<Test as SigningTypes>::Public,
// 				<Test as frame_system::Config>::BlockNumber
// 					> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&price_payload, signature);
//
// 			assert!(signature_valid);
// 		}
// 	});
// }

// #[test]
// fn should_submit_unsigned_transaction_on_chain_for_all_accounts() {
// 	const PHRASE: &str = "news slush supreme milk chapter athlete soap sausage put clutch what kitten";
// 	let (offchain, offchain_state) = testing::TestOffchainExt::new();
// 	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
//
// 	let keystore = KeyStore::new();
//
// 	SyncCryptoStore::sr25519_generate_new(
// 		&keystore,
// 		crate::crypto::Public::ID,
// 		Some(&format!("{}/hunter1", PHRASE))
// 	).unwrap();
//
// 	let public_key = SyncCryptoStore::sr25519_public_keys(&keystore, crate::crypto::Public::ID)
// 		.get(0)
// 		.unwrap()
// 		.clone();
//
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
// 	t.register_extension(TransactionPoolExt::new(pool));
// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
//
// 	price_oracle_response(&mut offchain_state.write());
//
// 	let price_payload = PricePayload {
// 		block_number: 1,
// 		price: 15523,
// 		public: <Test as SigningTypes>::Public::from(public_key),
// 	};
//
// 	// let signature = price_payload.sign::<crypto::TestAuthId>().unwrap();
// 	t.execute_with(|| {
// 		// when
// 		Example::fetch_price_and_send_unsigned_for_all_accounts(1).unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		let tx = Extrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature, None);
// 		if let Call::Example(crate::Call::submit_price_unsigned_with_signed_payload(body, signature)) = tx.call {
// 			assert_eq!(body, price_payload);
//
// 			let signature_valid = <PricePayload<
// 				<Test as SigningTypes>::Public,
// 				<Test as frame_system::Config>::BlockNumber
// 					> as SignedPayload<Test>>::verify::<crypto::TestAuthId>(&price_payload, signature);
//
// 			assert!(signature_valid);
// 		}
// 	});
// }

// #[test]
// fn should_submit_raw_unsigned_transaction_on_chain() {
// 	let (offchain, offchain_state) = testing::TestOffchainExt::new();
// 	let (pool, pool_state) = testing::TestTransactionPoolExt::new();
//
// 	let keystore = KeyStore::new();
//
// 	let mut t = sp_io::TestExternalities::default();
// 	t.register_extension(OffchainWorkerExt::new(offchain));
// 	t.register_extension(TransactionPoolExt::new(pool));
// 	t.register_extension(KeystoreExt(Arc::new(keystore)));
//
// 	price_oracle_response(&mut offchain_state.write());
//
// 	t.execute_with(|| {
// 		// when
// 		Example::fetch_price_and_send_raw_unsigned(1).unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		assert!(pool_state.read().transactions.is_empty());
// 		let tx = Extrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature, None);
// 		assert_eq!(tx.call, Call::Example(crate::Call::submit_price_unsigned(1, 15523)));
// 	});
// }

// fn price_oracle_response(state: &mut testing::OffchainState) {
// 	state.expect_request(testing::PendingRequest {
// 		method: "GET".into(),
// 		uri: "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD".into(),
// 		response: Some(br#"{"USD": 155.23}"#.to_vec()),
// 		sent: true,
// 		..Default::default()
// 	});
// }

// #[test]
// fn parse_price_works() {
// 	let test_data = vec![
// 		("{\"USD\":6536.92}", Some(653692)),
// 		("{\"USD\":65.92}", Some(6592)),
// 		("{\"USD\":6536.924565}", Some(653692)),
// 		("{\"USD\":6536}", Some(653600)),
// 		("{\"USD2\":6536}", None),
// 		("{\"USD\":\"6432\"}", None),
// 	];
//
// 	for (json, expected) in test_data {
// 		assert_eq!(expected, Example::parse_price(json));
// 	}
// }



fn setup_blocks(blocks: u64) {
	let mut parent_hash = System::parent_hash();
	for i in 1..(blocks + 1) {
		System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
		let header = System::finalize();
		parent_hash = header.hash();
	}
}
