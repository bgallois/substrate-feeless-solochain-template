use crate::{mock::*, CheckRate};
use frame_support::assert_ok;
use frame_system::Call;
use sp_runtime::{generic::SignedPayload, testing::H256};

#[test]
fn test_rate() {
    new_test_ext().execute_with(|| {
        /*System::set_block_number(1);
        assert_ok!(System::remark(RuntimeOrigin::signed(1), "1".into(),));
        let tx_ext: TxExtension = (CheckRate::<Test>::new(),);
        let test = frame_system::mocking::MockUncheckedExtrinsic::<Test>::new_transaction(
            Call::<Test>::remark{remark: "1".into()},
            tx_ext,
        );*/ //TODO
    })
}
