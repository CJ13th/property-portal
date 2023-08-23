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
pub use types::{PropertyId, Property, Listing, ListingId, Tenancy, TenancyId, Offer, OfferId, OfferStatus};


use frame_support::traits::fungible;
pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::traits::{fungible, fungible::{MutateFreeze, Inspect as OtherInspect, Mutate}};
	use frame_support::dispatch::RawOrigin;
	use frame_support::traits::tokens::Preservation::Preserve;

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
		type MaxOffersPerApplicant: Get<u32>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId, Id = Self::RuntimeFreezeReason>
			+ fungible::freeze::Mutate<Self::AccountId>;

		type RuntimeFreezeReason: From<FreezeReason>;
	}

	#[pallet::composite_enum]
	pub enum FreezeReason {
		Offer(OfferId),
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
	// pub type ApplicantOffers<T: Config> = StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat, ListingId, OfferId>;
	pub type ApplicantOffers<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<OfferId, T::MaxOffersPerApplicant>>;

	#[pallet::storage]
	// All offers for a listing
	pub type ListingOffers<T: Config> = StorageMap<_, Blake2_128Concat, ListingId, BoundedVec<OfferId, T::MaxOffersPerListing>>;

	#[pallet::storage]
	// Used to generate new offer id's
	pub type OfferCounter<T: Config> = StorageValue<_, OfferId>;


	#[pallet::storage]
	// A structure to hold information about tenancies
	pub type Tenancies<T: Config> = StorageMap<_, Blake2_128Concat, PropertyId, Tenancy<T>>;

	
	

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewApplicantRegistered { applicant_id: T::AccountId },
		NewLandlordRegistered { landlord_id: T::AccountId },
		NewPropertyRegistered { address: T::Hash, postal_code: T::Hash },
		NewListingCreated {property_id: PropertyId, rental_price: u32, availability_date:BlockNumberFor<T>},
		NewOfferSubmitted {listing_id: ListingId, offer_price: u32, offer_start_date: BlockNumberFor<T>, offer_end_date: BlockNumberFor<T>, prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>},
		OfferAccepted {offer_id: OfferId},
		ApplicantSignedOffer {applicant_id: T::AccountId},
	}

	#[pallet::error]
	pub enum Error<T> {
		TooManyProperties,
		TooManyListings,
		TooManyOffers,
		LandlordNotVerified,
		PropertyDoesNotExist,
		ListingDoesNotExist,
		OfferDoesNotExist,
		// Not autorized to perform this action. 
		Unauthorized,
		InvalidOfferStartDate,
		TenantsIdsCannotBeEmpty,
		AllApplicantsMustBeVerified,
		TooManyOffersOnListing,
		TenancyAlreadyExists,
		MaxOffersForApplicantReached,
		InsufficientFundsForOffer,
		OfferExpired,
		OfferValidUntilMustBeFuture,
		OfferCannotBeAccepted,
		TooManyTenants,
		OfferNotFullySigned,
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
			let new_property_count = property_count + 1;
			// The id should actually be a combination of the address, postal code and landlord id hashed
			let new_property = Property::new(new_property_count, landlord_id, address, postal_code);
			
			Properties::<T>::insert(&new_property_count, new_property);

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
		pub fn submit_offer(origin: OriginFor<T>, listing_id: ListingId, offer_price: u32, offer_start_date: BlockNumberFor<T>, offer_end_date: BlockNumberFor<T>, prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>, valid_until: BlockNumberFor<T>) -> DispatchResult {
			let applicant_id = ensure_signed(origin)?;
			ensure!(VerifiedApplicants::<T>::contains_key(&applicant_id), Error::<T>::Unauthorized);
			ensure!(Listings::<T>::contains_key(&listing_id), Error::<T>::ListingDoesNotExist);
			ensure!(T::NativeBalance::total_balance(&applicant_id) >= offer_price.into(), Error::<T>::InsufficientFundsForOffer);
			let offer_listing = Listings::<T>::get(&listing_id).unwrap();
			let current_block_number =  frame_system::Pallet::<T>::block_number();
			ensure!(valid_until > current_block_number, Error::<T>::OfferValidUntilMustBeFuture); // Maybe add min? Don't want one block offers
			ensure!(offer_start_date >= current_block_number
					&& offer_start_date < offer_end_date
					&& offer_start_date >= offer_listing.availability_date, Error::<T>::InvalidOfferStartDate);
			
			ensure!(prospective_tenant_ids.len() > 0, Error::<T>::TenantsIdsCannotBeEmpty);
			// ensure!(prospective_tenant_ids.len() <= T::MaxNumberOfTenants::get(), Error::<T>::TooManyTenants); Not necessary?
			ensure!(&prospective_tenant_ids.iter().all(|applicant_id| VerifiedApplicants::<T>::contains_key(&applicant_id)), Error::<T>::AllApplicantsMustBeVerified);
			let offer_count = OfferCounter::<T>::get().unwrap_or_default();
			ensure!(offer_count.checked_add(1).is_some(), Error::<T>::TooManyOffers); // change to storage overflow
			let new_offer_id = offer_count + 1;

			let number_of_prospective_tenants = prospective_tenant_ids.len();
			let init_ids_and_sigs: Vec<(T::AccountId, bool)> = prospective_tenant_ids.clone().into_iter().map(|t_id| if number_of_prospective_tenants == 1 {(t_id, true)} else {if t_id == applicant_id {(t_id, true)} else {(t_id, false)}}).collect();
			let prospective_tenant_signatures = BoundedVec::try_from(init_ids_and_sigs).map_err(|_| Error::<T>::TooManyTenants)?; // should not be possible to err here
			let all_signed = if number_of_prospective_tenants == 1 { true } else { false };
			let new_offer = Offer::new(new_offer_id, offer_listing.property_id, offer_price, offer_start_date, offer_end_date, applicant_id.clone(), prospective_tenant_ids.clone(), prospective_tenant_signatures, valid_until, all_signed);
			// new_offer.clone() does not work??
			// let new_offer2 = Offer::new(new_offer_id, offer_listing.property_id, offer_price, offer_start_date, offer_end_date, prospective_tenant_ids.clone());
			// We should prevent people from making multiple offers on a property.
			let mut offers_on_listing = ListingOffers::<T>::get(&listing_id).unwrap_or(BoundedVec::new());
			offers_on_listing.try_push(new_offer_id).map_err(|_| Error::<T>::TooManyOffersOnListing)?;

			let mut all_applicant_offers = ApplicantOffers::<T>::get(&applicant_id).unwrap_or(BoundedVec::new());
			all_applicant_offers.try_push(new_offer_id).map_err(|_| Error::<T>::MaxOffersForApplicantReached)?;


			ListingOffers::<T>::insert(&listing_id, &offers_on_listing);
			ApplicantOffers::<T>::insert(&applicant_id, &all_applicant_offers);
			Offers::<T>::insert(&new_offer_id, &new_offer);

			T::NativeBalance::set_freeze(
				&FreezeReason::Offer(new_offer_id).into(),
				&applicant_id,
				offer_price.into(),
			);

			Self::deposit_event(Event::NewOfferSubmitted { listing_id, offer_price, offer_start_date, offer_end_date, prospective_tenant_ids });
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn accept_offer(origin: OriginFor<T>, offer_id: OfferId) -> DispatchResult {
			let landlord_id = ensure_signed(origin)?;
			ensure!(Offers::<T>::contains_key(&offer_id), Error::<T>::OfferDoesNotExist);
			let mut offer = Offers::<T>::get(&offer_id).unwrap();
			let current_block_number =  frame_system::Pallet::<T>::block_number();
			ensure!(current_block_number <= offer.valid_until, Error::<T>::OfferExpired);
			ensure!(offer.offer_status == OfferStatus::Pending, Error::<T>::OfferCannotBeAccepted);
			ensure!(offer.all_signed, Error::<T>::OfferNotFullySigned);
			ensure!(offer.offer_start_date > current_block_number, Error::<T>::InvalidOfferStartDate); // add a buffer time maybe? start date must be at least curr + 100 blocks?
			let property_id = offer.property_id;
			ensure!(Properties::<T>::contains_key(&property_id), Error::<T>::PropertyDoesNotExist);
			let property = Properties::<T>::get(property_id).unwrap();
			ensure!(property.landlord_id == landlord_id, Error::<T>::Unauthorized);
			ensure!(!Tenancies::<T>::contains_key(&property_id), Error::<T>::TenancyAlreadyExists);
			offer.offer_status = OfferStatus::Accepted;
			T::NativeBalance::thaw(&FreezeReason::Offer(offer_id).into(), &offer.lead_tenant);
			T::NativeBalance::transfer(&offer.lead_tenant, &landlord_id, offer.offer_price.into(), Preserve);
			Offers::<T>::insert(&offer_id, &offer);
			let new_tenancy = Tenancy::new(offer);
			Tenancies::<T>::insert(&property_id, new_tenancy);

			// Locked funds will be transferred to the landlord
			// need to start thinking about multiple tenants
			// e.g offer of 1000 accepted from 4 tenants
			// 250 locked for each, 250 from each transferred to landlord.
			// Also need a way to override this so that custom amounts can be locked/sent.
			// Two flat mates might share but one has ensuite and pay 100 more than the other.

			/*
			3 tenants 1000 rent
			333 = 1000.checked_div(3)
			1 = 1000 - (333 * 3)
			The lead tenant pays the extra 1.
			*/
			
			Self::deposit_event(Event::OfferAccepted {offer_id});
			// Self::deposit_event(Event::TenancyCreated {});
			Ok(())
		}
		

		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn sign_offer(origin: OriginFor<T>, offer_id: OfferId) -> DispatchResult {
			let applicant_id = ensure_signed(origin)?;
			ensure!(VerifiedApplicants::<T>::contains_key(&applicant_id), Error::<T>::Unauthorized);
			ensure!(Offers::<T>::contains_key(&offer_id), Error::<T>::OfferDoesNotExist);
			let mut offer = Offers::<T>::get(&offer_id).unwrap();
			let current_block_number =  frame_system::Pallet::<T>::block_number();
			ensure!(current_block_number <= offer.valid_until, Error::<T>::OfferExpired);
			ensure!(offer.offer_status == OfferStatus::Pending, Error::<T>::OfferCannotBeAccepted);
			let new_tenants = offer.prospective_tenant_signatures.into_iter().map(|(app_id, signed)| if app_id == applicant_id {(app_id, true)} else {(app_id, signed)}).collect::<Vec<(T::AccountId, bool)>>();
			let all_signed = new_tenants.iter().all(|(applicant_id, signed)| *signed == true);
			let updated_prospective_tenants = BoundedVec::try_from(new_tenants).map_err(|_| Error::<T>::TooManyTenants)?; // should never happen since we don't ever append 
			offer.prospective_tenant_signatures = updated_prospective_tenants;
			offer.all_signed = all_signed;
			
			Offers::<T>::insert(&offer_id, offer);


			/*
			Get the offer
			is the offer still valid? Still in the pending state
			Update the prospective tenants list
			If all tenants have signed then set the offer to fully_signed = true
			*/

			Self::deposit_event(Event::ApplicantSignedOffer {applicant_id});
			Ok(())
		}
	}


	


	
	impl<T: Config> Pallet<T> { 
		pub fn get_property(property_id: PropertyId) -> Option<Property<T>> {
			Properties::<T>::get(&property_id)
		}
	}
}
