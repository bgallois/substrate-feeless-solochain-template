# 🚫 pallet-feeless: Feeless Transactions with Rate Limiting for Substrate 🚫

Welcome to **pallet-feeless** — a plug-and-play solution to enable fully **feeless transactions** in any Substrate-based blockchain! ✨

Instead of charging transaction fees, this pallet uses a **rate-limiting system** to ensure fair usage and network security. It's perfect for applications where user experience, accessibility, and cost-efficiency are top priorities.

---

## 🧩 Introduction

Most blockchains rely on transaction fees to prevent spam and incentivize validators. While effective, this system can create friction, especially for:

- Microtransactions 💸  
- New users 🧑‍💻  
- Developers building free-to-use apps 🌍

This pallet removes transaction fees altogether—**even during congestion**—by using a **rate-limiting extrinsic extension**. This makes your blockchain:

- 🚫 Free from fees  
- 🔄 Predictable for users  
- 🛡️ Secure against spam  

---

## 💡 How It Works

Instead of fees, each account is subject to a **rate limit**:

- ⏱️ **Max Transactions per Period**: The number of transactions allowed per account in a set block window.
- 📦 **Max Size per Period**: The total size of allowed transactions during that window.

This is enforced via a **custom extrinsic extension** (`CheckRate`) that plugs into Substrate’s transaction validation pipeline alongside checks like:

- `CheckNonce`
- `CheckWeight`
- `CheckEra`
- ...

All validation and accounting are performed **before and after dispatch**, with minimal storage access to preserve performance and security.

---

## ⚙️ Runtime Integration

### 1. Define `AccountData` in `frame_system`

```rust ignore
type AccountData = pallet_feeless::AccountData<Balance, BlockNumber>;
```

This custom account type stores balance, last block, and rate data per account.

---

### 2. Set Up `AccountStore` in `pallet_balances`

```ignore
type AccountStore = Account;
```

Required to track the account’s storage location and enable rate tracking.

---

### 3. Configure `pallet_feeless`

Add your custom settings in the runtime:

```rust ignore
impl pallet_feeless::Config for Runtime {
    type MaxTxByPeriod = ConstU32<128>;     // Max transactions per period
    type MaxSizeByPeriod = ConstU32<1>;     // Max size in bytes per period
    type Period = ConstU32<5>;              // Length of the rate-limiting window (in blocks)
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();                   // Do not forget to generate and reference the weight after benchmarking
}
```

---

### 4. Update Your Transaction Extensions

Extend the runtime transaction validation to include `CheckRate`:

```rust ignore
pub type TxExtension = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    pallet_feeless::CheckRate<Runtime>,                    // 👈 Rate limit extension
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
    frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
);
```

---

### 5. Benchmarking and Payload Setup

In `benchmarking.rs` or similar:

```rust ignore
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
    pallet_feeless::CheckRate::<runtime::Runtime>::new(), // 👈 Add this
    frame_system::CheckWeight::<runtime::Runtime>::new(),
    pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
    frame_metadata_hash_extension::CheckMetadataHash::<runtime::Runtime>::new(false),
);

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
        (), // 👈 Add this
        (),
        (),
        None,
    ),
);
```

---

## ✅ Benefits

- 🚫 **No Transaction Fees**  
  Users never worry about price fluctuations.

- ⚖️ **Built-in Fairness**  
  Every account gets a rate limit, ensuring equal access.

- 🛠️ **Developer Friendly**  
  Build dApps without forcing users to hold or buy tokens.

- 🌐 **Accessible UX**  
  Lower onboarding friction and support for micro-use cases.

---

## ⚠️ Considerations

While this system improves user experience, keep in mind:

- ⚙️ **Fine-tuning required**: Limits must strike a balance between usability and protection.
- 🎯 **No validator fees**: Block rewards or other models must be used to incentivize validators.
- 🛡️ **Spam resistance**: Rate limits must be sufficient to deter Sybil attacks or multi-account spamming.

---

## 📦 Want to Try It?

Just include this pallet in your Substrate runtime and configure the limits to your needs. No additional infrastructure or fee logic is required!

---

## 📄 License

[MIT](./LICENSE) — open-source and ready to use in your blockchain projects.
