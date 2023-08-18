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
pub use types::{PropertyId, Property, Listing, ListingId, Tenancy, Offer, OfferId, OfferStatus};

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
		type MaxNumberOfAgents: Get<u32>;
		type MaxOffersPerListing: Get<u32>;
	}

	#[pallet::storage]
	// Applicants who have been referenced and are now able to submit offers
	pub type VerifiedApplicants<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	#[pallet::storage]
	// Landlords who have verified that they own the property and are able to create a listing;
	pub type VerifiedLandlords<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

	#[pallet::storage]
	// A structure to contain unique property id's
	pub type Properties<T: Config> = StorageMap<_, Blake2_128Concat, PropertyId, Property<T>>;

	#[pallet::storage]
	// Used to generate new property id's
	pub type PropertyCounter<T: Config> = StorageValue<_, PropertyId>;

	#[pallet::storage]
	// Property listings
	pub type Listings<T: Config> = StorageMap<_, Blake2_128Concat, PropertyId, Listing<T>>;

	#[pallet::storage]
	// Used to generate new listing id's
	pub type ListingCounter<T: Config> = StorageValue<_, ListingId>;

	#[pallet::storage]
	// Offers on properties
	pub type Offers<T: Config> = StorageMap<_, Blake2_128Concat, OfferId, Offer<T>>;

	#[pallet::storage]
	// Offers on listings by applicant
	pub type ApplicantOffers<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ListingId, Offer<T>>;

	#[pallet::storage]
	// All offers for a listing
	pub type ListingOffers<T: Config> = StorageMap<_, Blake2_128Concat, ListingId, BoundedVec<Offer<T>, T::MaxOffersPerListing>>;

	#[pallet::storage]
	// Used to generate new offer id's
	pub type OfferCounter<T: Config> = StorageValue<_, OfferId>;

	

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewApplicantRegistered { applicant_id: T::AccountId },
		NewLandlordRegistered { landlord_id: T::AccountId },
		NewPropertyRegistered { address: T::Hash, postal_code: T::Hash },
		NewListingCreated {property_id: PropertyId, rental_price: u32, availability_date:BlockNumberFor<T>},
		NewOfferSubmitted {listing_id: ListingId, offer_price: u32, offer_start_date: BlockNumberFor<T>, offer_end_date: BlockNumberFor<T>, prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>},
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyProperties,
		TooManyListings,
		TooManyOffers,
		LandlordNotVerified,
		PropertyDoesNotExist,
		ListingDoesNotExist,
		// Not autorized to perform this action. 
		Unauthorized,
		InvalidOfferStartDate,
		TenantsIdsCannotBeEmpty,
		AllApplicantsMustBeVerified,
		TooManyOffersOnListing,

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

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_landlord(origin: OriginFor<T>, landlord_id: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			VerifiedLandlords::<T>::insert(&landlord_id, ());

			Self::deposit_event(Event::NewLandlordRegistered { landlord_id });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_property(origin: OriginFor<T>, address: T::Hash, postal_code: T::Hash, landlord_id: T::AccountId ) -> DispatchResult {
			ensure_root(origin)?;

			let property_count = PropertyCounter::<T>::get().unwrap_or_default();
			ensure!(property_count.checked_add(1).is_some(), Error::<T>::TooManyProperties);
			// The id should actually be a combination of the address, postal code and landlord id hashed
			let new_property = Property::new(property_count, landlord_id, address, postal_code);
			
			Properties::<T>::insert(&property_count + 1, new_property);

			Self::deposit_event(Event::NewPropertyRegistered { address, postal_code });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_listing(origin: OriginFor<T>, property_id: PropertyId, rental_price: u32, availability_date: BlockNumberFor<T>) -> DispatchResult {
			// Only landlords and their agents should be able to list properties
			let landlord_id = ensure_signed(origin)?;
			ensure!(Properties::<T>::contains_key(&property_id), Error::<T>::PropertyDoesNotExist);

			let property = Properties::<T>::get(property_id).unwrap();
			ensure!(property.landlord_id == landlord_id, Error::<T>::Unauthorized);

			let listing_count = ListingCounter::<T>::get().unwrap_or_default();
			ensure!(listing_count.checked_add(1).is_some(), Error::<T>::TooManyListings);

			let new_listing_id = listing_count + 1;
			let new_listing = property.create_listing(new_listing_id, rental_price, availability_date, landlord_id);
			
			Listings::<T>::insert(new_listing_id, new_listing);

			Self::deposit_event(Event::NewListingCreated { property_id, rental_price, availability_date });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn submit_offer(origin: OriginFor<T>, listing_id: ListingId, offer_price: u32, offer_start_date: BlockNumberFor<T>, offer_end_date: BlockNumberFor<T>, prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>) -> DispatchResult {
			let applicant_id = ensure_signed(origin)?;
			ensure!(VerifiedApplicants::<T>::contains_key(&applicant_id), Error::<T>::Unauthorized);
			ensure!(Listings::<T>::contains_key(&listing_id), Error::<T>::ListingDoesNotExist);
			let offer_listing = Listings::<T>::get(&listing_id).unwrap();
			let current_block_number =  frame_system::Pallet::<T>::block_number();
			ensure!(offer_start_date >= current_block_number
					&& offer_start_date > offer_end_date
					&& offer_start_date >= offer_listing.availability_date, Error::<T>::InvalidOfferStartDate);
			
			ensure!(prospective_tenant_ids.len() > 0, Error::<T>::TenantsIdsCannotBeEmpty);
			ensure!(&prospective_tenant_ids.iter().all(|applicant_id| VerifiedApplicants::<T>::contains_key(&applicant_id)), Error::<T>::AllApplicantsMustBeVerified);
			
			let offer_count = OfferCounter::<T>::get().unwrap_or_default();
			ensure!(offer_count.checked_add(1).is_some(), Error::<T>::TooManyOffers); // change to storage overflow
			let new_offer_id = offer_count + 1;
			let new_offer = Offer::new(new_offer_id, offer_listing.property_id, offer_price, offer_start_date, offer_end_date, prospective_tenant_ids.clone());
			// We should prevent people from making multiple offers on a property.
			let mut offers_on_listing = ListingOffers::<T>::get(&listing_id).unwrap_or(BoundedVec::new());
			ensure!(!offers_on_listing.is_full(), Error::<T>::TooManyOffersOnListing);
			offers_on_listing.push(new_offer);
			ListingOffers::<T>::insert(&listing_id, offers_on_listing);
			ApplicantOffers::<T>::insert(&applicant_id, &listing_id, &new_offer);

			Self::deposit_event(Event::NewOfferSubmitted { listing_id, offer_price, offer_start_date, offer_end_date, prospective_tenant_ids });
			Ok(())
		}
	}
}
