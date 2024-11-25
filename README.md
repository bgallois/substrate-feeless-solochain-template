# 🚫 Fully Fee-Less Substrate Node 🚫

This repository provides a Substrate-based blockchain node that implements a **fully feeless transaction model**. In this implementation, all transactions are free, but additional mechanisms are in place to ensure fair usage and prevent abuse. ✨

# Introduction

In the [Duniter node implementation](https://git.duniter.org/nodes/rust/duniter-v2s/),the concept of a “semi-feeless” Substrate blockchain is implemented. The semi-feeless weight system ensures efficient feeless transactions while maintaining the balance of network resources. This system introduces fees during periods of congestion, allowing the network to remain secure and functional even under high load.

This node template implements a completely feeless blockchain system achieved through an extrinsic extension (similar to what is used to check, for example, that an account exists on the chain before doing anything (zero sender check)) that controls transaction submission rates.

## The Semi-Feeless Approach: A Quick Recap

Before discussing the new model’s details, let’s briefly revisit the previous system, the semi-feeless model implemented on Duniter.

In a typical blockchain network, fees are crucial in regulating transaction flow. Fees incentivize validators to include transactions in blocks and ensure spam attacks don’t overwhelm the system. However, excessive fees can hurt user experience and make microtransactions unfeasible. On the other hand, a fee-less system risks congestion by malicious actors overwhelming the network with spam or low-value transactions.

Duniter’s semi-feeless model implemented a dynamic system where transaction weights to fees are modulated to address this. When the network is not congested, transactions are free of fees. However, during high network load periods (when congestion threatens to slow down the blockchain), a fee mechanism kicks in to regulate transaction submission.

This model strikes a balance by providing an attractive user experience during off-peak times while ensuring the network remains protected during high-demand periods. However, this approach is partially fee-less, and the constant need for dynamic fee regulation highlighted an area for further improvement.

## Enter the Totally Feeless Blockchain

Fast-forward to today, when we’re exploring the next step: a completely feeless blockchain. In this new design, we remove transaction fees entirely, even during periods of network congestion. But how can a blockchain remain secure, scalable, and resistant to spam without relying on fees? The answer lies in rate-limiting transaction submissions through an extrinsic extension.

## How Rate-Limiting Solves the Problem

Rather than relying on fees to deter spam or congestion, we introduce a rate-limiting mechanism to control how frequently an account can submit transactions to the network. By imposing a transaction submission rate limit (a length limit for chain accepting data can also be implemented), we ensure that the network can handle the flood of transactions, regardless of whether they are spammy or legitimate.

This rate-limiting feature is implemented as an extrinsic extension on the Substrate chain. Several extensions are already in place, most by the frame system for checking the validity of a transaction (nonce, mortality, version, etc.) and one by the pallet transaction payment to compute and take the fee associated with the transaction. By adding a transaction submission rate limiter as an extrinsic extension, we can introduce this feature without compromising the fundamental operation of the blockchain.

Basically, the extension performs operations before and after the dispatch. These operations can include a weight that will be charged to the caller. Since there will be no fees, the pre-dispatch operation should be lightweight (at most, one storage access) to minimize its impact on the chain and avoid making it vulnerable to spam. This technique is similar to how Substrate checks if the sender’s balance is sufficient for a transaction (even when the balance is zero and no fee can be incurred) without compromising the chain security.

Each user account on the blockchain is given a defined transaction submission rate. For example, an account can only submit one transaction every few blocks, depending on the network’s configuration. This limit ensures that no account can flood the network with transactions in a short time, and the low footprint of the extension logic prevents a large group of accounts from overloading the chain with invalid transactions.

Since the rate-limiting mechanism handles congestion control, the network no longer needs to introduce fees based on load. This makes the blockchain truly feeless, even under stress. Users can continue to transact freely, knowing their actions will be limited only by their account’s rate and not by a fluctuating fee system.

## Benefits of a Feeless Blockchain with Rate-Limiting

- **No transaction fees:** Users no longer have to worry about transaction fees, making microtransactions and frequent interactions with the blockchain completely costless.
- **Lower barrier to entry:** Users can interact with the blockchain without needing to hold a large amount of native tokens to pay for variable and unpredictable transaction costs. The feeless blockchain is more predictable, and its price is more transparent for the user.

## Challenges and Considerations

While a feeless blockchain offers numerous advantages, it has its own challenges. A rigid rate-limiting system must ensure fairness. Without proper balance, it could inadvertently disadvantage certain users or applications that must quickly submit a large volume of transactions. More importantly, it can also reduce the incentive to participate in the network because, without fees, the validator rewards are diminished.

## How To Use It

The feeless logic is encapsulated in a pallet that needs to be integrated into the runtime.

1. **Add the pallet to the runtime:**

    ```rust
    impl pallet_feeless::Config for Runtime {
        type MaxTxByPeriod = ConstU32<1>;
        type Period = ConstU32<5>;
    }
    ```

2. **Use the custom `AccountData` and `AccountStore`:**

    ```rust
    impl frame_system::Config for Runtime {
        /// The data to be stored in an account.
        type AccountData = pallet_feeless::AccountData<Balance, BlockNumber>;
    }

    impl pallet_balances::Config for Runtime {
        type AccountStore = Account;
    }
    ```

3. **Add the extension:**

    ```rust
    /// The `TransactionExtension` to the basic transaction logic.
    pub type TxExtension = (
        frame_system::CheckNonZeroSender<Runtime>,
        frame_system::CheckSpecVersion<Runtime>,
        frame_system::CheckTxVersion<Runtime>,
        frame_system::CheckGenesis<Runtime>,
        frame_system::CheckEra<Runtime>,
        frame_system::CheckNonce<Runtime>,
        pallet_feeless::CheckRate<Runtime>,
        frame_system::CheckWeight<Runtime>,
        pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
        frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
    );
    ```

4. **In `/node/src/benchmarkings.rs`:**

    ```rust
    let raw_payload = runtime::SignedPayload::from_raw(
        call.clone(),
        tx_ext.clone(),
        (
            (),
            runtime::VERSION.spec_version,
            runtime::VERSION.transaction_version,
            genesis_hash,
            best_hash,
            (),
            (),
            (),
            (),
            None,
        ),
    );

    let tx_ext: runtime::TxExtension = (
        frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
        frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
        frame_system::CheckTxVersion::<runtime::Runtime>::new(),
        frame_system::CheckGenesis::<runtime::Runtime>::new(),
        frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
            period,
            best_block.saturated_into(),
        )),
        frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
        pallet_feeless::CheckRate::<runtime::Runtime>::new(),
        frame_system::CheckWeight::<runtime::Runtime>::new(),
        pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
        frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
    );
    ```

5. **Handling tips (Optional):**

    Currently, the feeless pallet does not incorporate logic to handle tips for validators to increase transaction priority and reward them. The user can remove the transaction payment extension and pallet if this functionality is unnecessary. However, if desired, it can be kept by mapping the weight to zero fee:

    ```rust
    impl pallet_transaction_payment::Config for Runtime {
        type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
        type LengthToFee = frame_support::weights::FixedFee<0, Balance>;
        type OnChargeTransaction = FungibleAdapter<Balances, ()>;
        type OperationalFeeMultiplier = ConstU8<0>;
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
        type WeightToFee = frame_support::weights::FixedFee<0, Balance>;
    }
    ```

## Conclusion

The shift from a semi-feeless system to a feeless blockchain is a step forward in blockchain design for Substrate blockchains, where the main incentive to participate in the network is not monetary.

By adding rate-limiting through an extrinsic extension, we’ve found a way to remove transaction fees while keeping the blockchain secure, efficient, and protected from spam. This approach gives users a free experience and offers a practical solution for blockchains without sacrificing security.

However, with no fees, the incentive for validators to contribute to the network is eliminated. Chains implementing this approach must introduce alternative incentive mechanisms to encourage validator participation.

## Future Enhancements

While the system is feeless, it could be configured to adjust rates based on network load. During periods of congestion, the blockchain can temporarily tighten the rate limits, further slowing down the flow of transactions from overactive accounts. Conversely, when the network is less congested, limits can be relaxed, allowing faster transaction processing without requiring fees.

If congestion rises, a more sophisticated system could introduce transaction prioritization based on factors like transaction size, account reputation, or tipping. However, the core idea remains: transactions

---

### TODO List
- [x] Custom `AccountData` and `AccountStore`
- [x] Rate limiter transaction extension
- [ ] Length limiter transaction extension
- [ ] Package in one standalone pallet
