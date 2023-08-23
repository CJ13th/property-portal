use crate::{mock::*, Error, Event, Property};
use frame_support::{assert_noop, assert_ok, pallet_prelude::DispatchError, traits::{fungible, tokens::{fungible::freeze::Inspect, Preservation::Expendable}}, BoundedVec};
use sp_runtime::TokenError::Frozen;


#[test]
fn can_register_applicant() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(RealEstate::register_applicant(RuntimeOrigin::root(), 1));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::NewApplicantRegistered { applicant_id: 1 }.into());
	});
}

#[test]
fn can_register_landlord() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(RealEstate::register_landlord(RuntimeOrigin::root(), 1));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::NewLandlordRegistered { landlord_id: 1 }.into());
	});
}

#[test]
fn can_register_property() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.
		assert_ok!(RealEstate::register_property(RuntimeOrigin::root(), sp_core::H256::repeat_byte(1), sp_core::H256::repeat_byte(1), 1));
		// Assert that the correct event was deposited
		System::assert_last_event(Event::NewPropertyRegistered { address: sp_core::H256::repeat_byte(1), postal_code: sp_core::H256::repeat_byte(1) }.into());
	});
}

#[test]
fn balance_is_frozen_on_submit_offer() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		assert_ok!(RealEstate::register_applicant(RuntimeOrigin::root(), 1));
		let _ = <Balances as fungible::Mutate<_>>::mint_into(&1, 1000);
		assert_eq!(Balances::free_balance(&1), 1000);
		assert_ok!(RealEstate::register_property(RuntimeOrigin::root(), sp_core::H256::repeat_byte(1), sp_core::H256::repeat_byte(1), 2));
		assert_ok!(RealEstate::create_listing(RuntimeOrigin::signed(2), 1, 1000, 50));
		let mut tenants = BoundedVec::new();
		tenants.try_push((1)).unwrap();
		assert_ok!(RealEstate::submit_offer(RuntimeOrigin::signed(1), 1, 900, 51, 101, tenants, 100));

		assert_eq!(
			<Balances as fungible::Mutate<_>>::transfer(&1, &2, 101, Expendable),
			Err(DispatchError::Token(Frozen))
		);
	});
}


#[test]
fn funds_are_transferred_on_offer_accepted() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		assert_ok!(RealEstate::register_applicant(RuntimeOrigin::root(), 101));
		assert_ok!(RealEstate::register_applicant(RuntimeOrigin::root(), 102));
		let _ = <Balances as fungible::Mutate<_>>::mint_into(&101, 1000);
		assert_eq!(Balances::free_balance(&101), 1000);
		assert_ok!(RealEstate::register_property(RuntimeOrigin::root(), sp_core::H256::repeat_byte(1), sp_core::H256::repeat_byte(1), 2));
		assert_ok!(RealEstate::create_listing(RuntimeOrigin::signed(2), 1, 1000, 50));
		let mut tenants = BoundedVec::new();
		tenants.try_push((101)).unwrap();
		tenants.try_push((102)).unwrap();
		assert_ok!(RealEstate::submit_offer(RuntimeOrigin::signed(101), 1, 900, 51, 101, tenants, 100));
		assert_ok!(RealEstate::sign_offer(RuntimeOrigin::signed(102), 1));

		assert_eq!(Balances::free_balance(&2), 0);
		let p = Property {
			 property_id: 1,
			 landlord_id: 2,
			 address: sp_core::H256::repeat_byte(1),
			 postal_code: sp_core::H256::repeat_byte(1),
		};
		assert_eq!(RealEstate::get_property(1).unwrap(), p);
		assert_eq!(Balances::free_balance(&2), 0);
		assert_ok!(RealEstate::accept_offer(RuntimeOrigin::signed(2), 1));
		assert_eq!(Balances::free_balance(&2), 900);
	});
}

// Landlord can also be an applicant, but they should not be able to offer on their own property.