#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;


use frame_support::{
	dispatch::DispatchResult, traits::IsSubType,
	weights::{DispatchClass, ClassifyDispatch, WeighData, Weight, PaysFee, Pays},
};
use sp_runtime::SaturatedConversion;

struct WeightForSetDummy<T: pallet_balances::Config>(BalanceOf<T>);

// 给 WeightForSetDummy 做一个  WeighData 的实现
impl<T: pallet_balances::Config> WeighData<(&BalanceOf<T>,)> for WeightForSetDummy<T>
{
	fn weigh_data(&self, target: (&BalanceOf<T>,)) -> Weight {

		let multiplier = self.0;
		// *target.0 is the amount passed into the extrinsic
		let cents = *target.0 / <BalanceOf<T>>::from(MILLICENTS);
		(cents * multiplier).saturated_into::<Weight>()
	}
}

/// A type alias for the balance type from this pallet's point of view.
type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
const MILLICENTS: u32 = 1_000_000_000;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use codec::{Encode, Decode};
	use sp_io::hashing::blake2_128;
	use sp_std::vec::Vec;

	use sp_runtime::{
		RuntimeDebug, DispatchError, ArithmeticError,
		traits::{
			self, CheckedAdd, CheckedSub, AtLeast32Bit, AtLeast32BitUnsigned, BadOrigin, BlockNumberProvider, Bounded,
			CheckEqual, Dispatchable, Hash, Lookup, LookupError, MaybeDisplay, MaybeMallocSizeOf,
			MaybeSerializeDeserialize, Member, One, Saturating, SimpleBitOps, StaticLookup, Zero,
		},
	};

	// use sp_io::misc::{Balance, WithdrawReasons, ExistenceRequirement};

	use sp_std::{cmp, result, mem, fmt::Debug, ops::BitOr};
	use frame_support::traits::{Currency, LockableCurrency, ReservableCurrency};
	use frame_benchmarking::frame_support::dispatch::PostDispatchInfo;
	use frame_benchmarking::frame_support::sp_runtime::DispatchErrorWithPostInfo;

	// use sp_runtime::app_crypto::sp_core::blake2_128;
	// use sp_core::hashing::blake2_128;

	#[derive(Encode, Decode)]
	pub struct Kitty (pub [u8; 16]);

	// Define iden
	// type KittyIndex = u32;


    // use balance
	// type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// type BalanceOf<T> = <T as pallet_balances::Config>::Balance;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


	const MILLICENTS: u32 = 1_000_000_000;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type KittyIndex: Parameter + Member + MaybeSerializeDeserialize + Debug + Default + MaybeDisplay + AtLeast32Bit + Copy;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;
		// TODO:: 这里不会。

		type MaxStakeBalance: Get<BalanceOf<Self>>;

	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn kitties_count)]
	pub type KittiesCount<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sell_list)]
	pub type SellList<T: Config> =  StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(T::KittyIndex, T::Balance)>, ValueQuery>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		KittyCreate(T::AccountId, T::KittyIndex),
		KittyTransfer(T::AccountId, T::AccountId, T::KittyIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		KittiesCountOverflow,
		NotOwner,
		SameParentIndex,
		InvalidKittyIndex,
		KittyHasNotSold,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// stake some balance
			T::Currency::reserve(&who, T::MaxStakeBalance::get());

			let mut kitty_count = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != u32::max_value(), Error::<T>::KittiesCountOverflow) ;
					id
				},
				None => {
					0
				}
			};

			// Add count
			kitty_count += 1;
			// Add kitty id
			let kitty_id : T::KittyIndex = kitty_count.into() ;
			// // 获取 dna
			let dna = Self::random_value(&who);
			Kitties::<T>::insert(kitty_id, Some(Kitty(dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			// Set to kittie count.
			KittiesCount::<T>::put(kitty_count);
			// Emit event
			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			// let a:Vec<u8> = Vec::new();
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(origin: OriginFor<T>, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::to_transfer(who.clone(), new_owner.clone(), kitty_id.clone())
		}

		#[pallet::weight(0)]
		pub fn bread(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(kitty_id_1 != kitty_id_2, Error::<T>::SameParentIndex);
			let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyIndex)?;
			let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyIndex)?;

			let mut kitty_count = match Self::kitties_count() {
				Some(id) => {
					ensure!(id != u32::max_value(), Error::<T>::KittiesCountOverflow) ;
					id
				},
				None => {
					0
				}
			};

			let dna_1 = kitty1.0;
			let dna_2 = kitty2.0;

			let selector = Self::random_value(&who);
			let mut new_dna = [0u8; 16];

			for i in 0..dna_1.len() {
				new_dna[i] = (selector[i] & dna_1[i]) | (!selector[i] & dna_2[i]);
			}

			// Sum kitty_count
			kitty_count += 1;
			// Add kitty id
			let kitty_id : T::KittyIndex = kitty_count.into() ;
			Kitties::<T>::insert(kitty_id, Some(Kitty(new_dna)));
			Owner::<T>::insert(kitty_id, Some(who.clone()));
			KittiesCount::<T>::put(kitty_count );
			Self::deposit_event(Event::KittyCreate(who, kitty_id));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn to_sell(origin: OriginFor<T>, kitty_id: T::KittyIndex, balance: T::Balance ) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let mut sell_list = SellList::<T>::get(who.clone()) ;
			for (tmp_index, (tmp_kitty_id, _)) in sell_list.clone().iter().enumerate() {
				if tmp_kitty_id == &kitty_id {
					// Del old.
					sell_list.remove(tmp_index);
				}
			}
			// update or set new price .
			sell_list.push((kitty_id,balance) );
			SellList::<T>::insert(who, sell_list);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn to_buy(origin: OriginFor<T>, dest: <T::Lookup as StaticLookup>::Source, kitty_id: T::KittyIndex ) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let owner_id = <T::Lookup as StaticLookup>::lookup(dest.clone())?;

			// 1. first check whether kitty is being sold, if not display an Error.
			let mut sell_list = SellList::<T>::get(owner_id.clone()) ;
			let mut kitty_is_sold = false;
			for (_, (tmp_kitty_id, tmp_balance)) in sell_list.clone().iter().enumerate() {
				if tmp_kitty_id == &kitty_id {
					kitty_is_sold = true;
					// <pallet_balances::Pallet<T>>::reserved_balance()
					match <pallet_balances::Pallet<T>>::transfer(origin.clone(), dest.clone(), tmp_balance.clone())  {
						Ok(_) => {
							// TODO:: It should be judged that if the transmission fails, you need to refund the money.
							// Un stake.
							T::Currency::unreserve(&owner_id, T::MaxStakeBalance::get());
							return Self::to_transfer(owner_id.clone(), who.clone(), kitty_id.clone());
						}
						Err(e) => {
							return Err(e.error) ;
						}
					}
				}
			}
			// not exists on the list
			if false == kitty_is_sold {return Err(Error::<T>::KittyHasNotSold)?}

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn random_value(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			// [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
			payload.using_encoded(blake2_128)
		}

		pub fn to_transfer(owner: T::AccountId, new_owner: T::AccountId, kitty_id: T::KittyIndex) -> DispatchResult {
			// println!("{:?},{:?},{:?}",Some(owner.clone()), Owner::<T>::get(kitty_id.clone()), kitty_id);
			ensure!(Some(owner.clone()) == Owner::<T>::get(kitty_id.clone()), Error::<T>::NotOwner) ;
			Owner::<T>::insert(kitty_id, Some(new_owner.clone()));
			Self::deposit_event(Event::KittyTransfer(owner, new_owner, kitty_id));
			Ok(())
		}
	}
}
