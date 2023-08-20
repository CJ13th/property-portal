use crate::{mock::*, Error, Event};
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
fn balances_test() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		assert_ok!(RealEstate::register_applicant(RuntimeOrigin::root(), 1));
		let _ = <Balances as fungible::Mutate<_>>::mint_into(&1, 900);
		assert_eq!(Balances::free_balance(&1), 900);
		assert_ok!(RealEstate::register_property(RuntimeOrigin::root(), sp_core::H256::repeat_byte(1), sp_core::H256::repeat_byte(1), 2));
		assert_ok!(RealEstate::create_listing(RuntimeOrigin::signed(2), 1, 1000, 50));
		let mut tenants = BoundedVec::new();
		tenants.try_push(1).unwrap();
		assert_ok!(RealEstate::submit_offer(RuntimeOrigin::signed(1), 1, 900, 51, 101, tenants));


		assert_eq!(
			<Balances as fungible::Mutate<_>>::transfer(&1, &2, 50, Expendable),
			Err(DispatchError::Token(Frozen))
		);


	});
}

// Landlord can also be an applicant, but they should not be able to offer on their own property.