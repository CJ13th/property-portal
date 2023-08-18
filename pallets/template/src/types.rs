use crate::{Config};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::*;
use frame_support::pallet_prelude::*;

pub type PropertyId = u128;
pub type OfferId = u128;

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
pub struct Property<T: Config> {
    pub property_id: PropertyId,
    pub landlord_id: T::AccountId,
    pub address: T::Hash,
    pub postal_code: T::Hash,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
pub struct Listing<T: Config> {
    pub property_id: PropertyId,
    pub rental_price: u32,
    pub availability_date: BlockNumberFor<T>,
    pub lister: T::AccountId
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
pub struct Tenancy<T: Config> {
    pub property_id: PropertyId,
    pub rental_price: u32,
    pub start_date: BlockNumberFor<T>,
    pub end_date: BlockNumberFor<T>,
}

#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone)]
pub struct Offer<T: Config> {
    pub offer_id: OfferId,
    pub property_id: PropertyId,
    pub offer_price: u32,
    pub offer_start_date: BlockNumberFor<T>,
    pub offer_end_date: BlockNumberFor<T>,
    pub prospective_tenant_ids: BoundedVec<T::AccountId, T::MaxNumberOfTenants>
}
