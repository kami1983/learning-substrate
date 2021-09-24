#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
pub use frame_support::{parameter_types};

/// Add test mock .
#[cfg(test)]
mod mock;
// Add test file.
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;

// pub const PROOF_MAX_LENGTH: u8 = 10;
//
//
// parameter_types! {
// 	pub const ProofMaxLength: u8 = PROOF_MAX_LENGTH / 2;
// }

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	pub use crate::weights::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type ProofMaxLength : Get<u8>;
		/// Information on runtime weights.
		type WeightInfo: WeightInfo;
	}
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub type Proofs <T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
        (T::AccountId, T::BlockNumber)
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ClaimCreated(T::AccountId, Vec<u8>),
		ClaimRevoked(T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ProofAlreadyExists,
		ClaimNotExist,
		NotClaimOwner,
		ProofLengthTooLong,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		// #[pallet::weight(0)]
		#[pallet::weight(T::WeightInfo::create_claim())]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			// Check clain lenth
			ensure!(claim.len() <= T::ProofMaxLength::get().into() , Error::<T>::ProofLengthTooLong);

			let sender = ensure_signed(origin)?;
			ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExists);
			Proofs::<T>::insert(&claim, (sender.clone(), frame_system::Pallet::<T>::block_number()));
			Self::deposit_event(Event::ClaimCreated(sender, claim));
			Ok(().into())
		}

		// #[pallet::weight(0)]
		#[pallet::weight(T::WeightInfo::revoke_claim())]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let (owner,_) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			Proofs::<T>::remove(&claim);
			Self::deposit_event(Event::ClaimRevoked(sender,claim));
			Ok(().into())
		}

		// #[pallet::weight(0)]
		#[pallet::weight(T::WeightInfo::transfer_claim())]
		pub fn transfer_claim(origin: OriginFor<T>, claim: Vec<u8>, dest: T::AccountId) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let(owner, _block_number) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;
			ensure!(owner == sender, Error::<T>::NotClaimOwner);
			Proofs::<T>::insert(&claim, (dest, frame_system::Pallet::<T>::block_number()));
			Ok(().into())
		}

	}
}
