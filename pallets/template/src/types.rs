use crate::{Config};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;

pub type PropertyId = u128;
pub type ListingId = u128;
pub type OfferId = u128;
pub type TenancyId = u128;


#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
#[scale_info(skip_type_params(T))]
pub struct Property<T: Config> {
    pub property_id: PropertyId,
    pub landlord_id: T::AccountId,
    // pub assigned_agents: BoundedVec<T::AccountId, T::MaxNumberOfAgents>,
    pub address: T::Hash,
    pub postal_code: T::Hash,
}

impl<T: Config> Property<T> {
    pub fn new(property_id: PropertyId, landlord_id: T::AccountId, address: T::Hash, postal_code: T::Hash) -> Property<T> {
        Property {
            property_id,
            landlord_id,
            address,
            postal_code
        }
    }

    pub fn create_listing(self, listing_id: ListingId, rental_price: u32, availability_date: BlockNumberFor<T>, lister: T::AccountId) -> Listing<T> {
        Listing {
            listing_id,
            property_id: self.property_id,
            rental_price, 
            availability_date,
            lister
        }
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
#[scale_info(skip_type_params(T))]
pub struct Listing<T: Config> {
    pub listing_id: ListingId,
    pub property_id: PropertyId,
    pub rental_price: u32,
    pub availability_date: BlockNumberFor<T>,
    pub lister: T::AccountId
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
#[scale_info(skip_type_params(T))]
pub struct Tenancy<T: Config> {
    pub property_id: PropertyId,
    pub rental_price: u32,
    pub start_date: BlockNumberFor<T>,
    pub end_date: BlockNumberFor<T>,
    pub tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>,
}

impl<T: Config> Tenancy<T> {
    pub fn new(offer: Offer<T>) -> Tenancy<T> {
        Tenancy {
            property_id: offer.property_id,
            rental_price: offer.offer_price,
            start_date: offer.offer_start_date,
            end_date: offer.offer_end_date,
            tenant_ids: offer.prospective_tenant_ids,
        }
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
#[scale_info(skip_type_params(T))]
pub struct Offer<T: Config> {
    pub offer_id: OfferId,
    pub property_id: PropertyId,
    pub offer_price: u32,
    pub offer_start_date: BlockNumberFor<T>,
    pub offer_end_date: BlockNumberFor<T>,
    pub lead_tenant: T::AccountId,
    pub prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>,
    pub offer_status: OfferStatus,
}

impl<T: Config> Offer<T> {
    pub fn new(offer_id: OfferId, property_id: PropertyId, offer_price: u32, offer_start_date: BlockNumberFor<T>, offer_end_date: BlockNumberFor<T>, lead_tenant: T::AccountId, prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>) -> Offer<T> {
        Offer {
            offer_id,
            property_id,
            offer_price,
            offer_start_date,
            offer_end_date,
            lead_tenant,
            prospective_tenant_ids,
            offer_status: OfferStatus::Pending,
        }
    }
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
pub enum OfferStatus {
    Cancelled,
    Pending,
    Accepted,
    Rejected
}