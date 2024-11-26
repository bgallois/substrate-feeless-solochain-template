use crate::{mock::*, CheckRate};
use frame_support::{
    assert_err, assert_ok, dispatch::DispatchInfo, pallet_prelude::InvalidTransaction,
};
use sp_runtime::{traits::DispatchTransaction, transaction_validity::TransactionValidityError};

#[test]
fn transaction_work() {
    new_test_ext().execute_with(|| {
        let info = DispatchInfo::default();
        let len = 0_usize;
        assert_ok!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            ))
        );
    })
}

#[test]
fn transaction_fail_after_quota() {
    new_test_ext().execute_with(|| {
        let info = DispatchInfo::default();
        let len = 0_usize;
        for _ in 0..<Test as crate::Config>::MaxTxByPeriod::get() {
            assert_ok!(CheckRate::<Test>::new().test_run(
                Some(1).into(),
                CALL,
                &info,
                len,
                0,
                |_| Ok(Default::default())
            ));
        }
        assert_err!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            )),
            TransactionValidityError::Invalid(InvalidTransaction::ExhaustsResources,)
        );
    })
}

#[test]
fn transaction_success_after_period() {
    new_test_ext().execute_with(|| {
        let info = DispatchInfo::default();
        let len = 0_usize;
        for _ in 0..<Test as crate::Config>::MaxTxByPeriod::get() {
            assert_ok!(CheckRate::<Test>::new().test_run(
                Some(1).into(),
                CALL,
                &info,
                len,
                0,
                |_| Ok(Default::default())
            ));
        }
        assert_err!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            )),
            TransactionValidityError::Invalid(InvalidTransaction::ExhaustsResources,)
        );
        System::set_block_number(<Test as crate::Config>::Period::get().into());
        assert_ok!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            ))
        );
    })
}

#[test]
fn too_big_fail() {
    new_test_ext().execute_with(|| {
        System::set_block_number(<Test as crate::Config>::Period::get().into());
        let info = DispatchInfo::default();
        let len = <Test as crate::Config>::MaxSizeByPeriod::get() as usize;
        assert_err!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            )),
            TransactionValidityError::Invalid(InvalidTransaction::ExhaustsResources,)
        );
    });
}

#[test]
fn small_success_until_limit() {
    new_test_ext().execute_with(|| {
        let info = DispatchInfo::default();
        let len = (<Test as crate::Config>::MaxSizeByPeriod::get() / 4) as usize;
        for _ in 0..3 {
            assert_ok!(CheckRate::<Test>::new().test_run(
                Some(1).into(),
                CALL,
                &info,
                len,
                0,
                |_| Ok(Default::default())
            ));
        }
        assert_err!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            )),
            TransactionValidityError::Invalid(InvalidTransaction::ExhaustsResources,)
        );
        System::set_block_number(<Test as crate::Config>::Period::get().into());
        assert_ok!(
            CheckRate::<Test>::new().test_run(Some(1).into(), CALL, &info, len, 0, |_| Ok(
                Default::default()
            ))
        );
    })
}
