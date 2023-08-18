#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use types::{PropertyId, Property, Listing, Tenancy, Offer, OfferId};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type MaxNumberOfTenants: Get<u32>;
	}

	#[pallet::storage]
	// Applicants who have been referenced and are now able to submit offers
	pub type VerifiedApplicants<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	#[pallet::storage]
	// Landlords who have verified that they own the property and are able to create a listing;
	pub type VerifiedLandlords<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewApplicantRegistered { applicant_id: T::AccountId },
		NewLandlordRegistered { applicant_id: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_applicant(origin: OriginFor<T>, applicant_id: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			VerifiedApplicants::<T>::insert(&applicant_id, ());

			Self::deposit_event(Event::NewApplicantRegistered { applicant_id });
			Ok(())
		}
	}
}
