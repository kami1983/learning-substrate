//! Benchmarking setup for pallet-poe
//! Need Build with: cargo build --release --features runtime-benchmarks
//! Need Test with: cargo test -p pallet-poe --features runtime-benchmarks
//! Need Make weights.rs with: ./target/release/node-template benchmark --chain dev --execution=wasm --wasm-execution=compiled --pallet pallet-poe --extrinsic '*' --steps 20 --repeat 50 --template=.maintain/frame-weight-template.hbs --output=./pallets/poe/src/weights.rs

use super::*;


#[allow(unused)]
use crate::Pallet as Poe;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, account};
use frame_system::RawOrigin;

benchmarks! {
	create_claim {
		// let s in claim_list;
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller.clone()), "HASH".as_bytes().to_vec())
	verify {
		assert_eq!(Proofs::<T>::get("HASH".as_bytes().to_vec()), Some((caller, frame_system::Pallet::<T>::block_number())));
	}

	revoke_claim {
		// let s in claim_list;
		let caller: T::AccountId = whitelisted_caller();
		// add proofs
		Proofs::<T>::insert("HASH".as_bytes().to_vec(), (caller.clone(), frame_system::Pallet::<T>::block_number()));
		assert_eq!(Proofs::<T>::get("HASH".as_bytes().to_vec()), Some((caller.clone(), frame_system::Pallet::<T>::block_number())));

	}: _(RawOrigin::Signed(caller.clone()), "HASH".as_bytes().to_vec())
	verify {
		assert_eq!(Proofs::<T>::get("HASH".as_bytes().to_vec()), None);
	}

	transfer_claim {
		// let s in claim_list;
		let caller1: T::AccountId = account::<T::AccountId>("whitelisted_caller1", 0, 0);
		let caller2: T::AccountId = account::<T::AccountId>("whitelisted_caller2", 0, 1);
		// let caller3: T::AccountId = account::<T::AccountId>("whitelisted_caller3", 0, 2);

		// add proofs
		Proofs::<T>::insert("HASH".as_bytes().to_vec(), (caller1.clone(), frame_system::Pallet::<T>::block_number()));
		assert_eq!(Proofs::<T>::get("HASH".as_bytes().to_vec()), Some((caller1.clone(), frame_system::Pallet::<T>::block_number())));
	}: _(RawOrigin::Signed(caller1.clone()), "HASH".as_bytes().to_vec(), caller2.clone())
	verify {
		assert_eq!(Proofs::<T>::get("HASH".as_bytes().to_vec()), Some((caller2.clone(), frame_system::Pallet::<T>::block_number())));
	}
}
impl_benchmark_test_suite!(Poe, crate::mock::new_test_ext(), crate::mock::Test);
