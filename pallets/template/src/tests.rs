use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};


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

// Landlord can also be an applicant, but they should not be able to offer on their own property.