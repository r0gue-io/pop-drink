use frame_support::{
	derive_impl, parameter_types,
	sp_runtime::traits::{IdentifyAccount, Lazy, Verify},
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_contracts::{
	config_preludes::{
		CodeHashLockupDepositPercent, DefaultDepositLimit, DepositPerByte, DepositPerItem,
		MaxDelegateDependencies,
	},
	DefaultAddressGenerator, Frame, Schedule,
};
use pallet_nfts::PalletFeatures;
use scale::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;

type AccountId = u64;
type HashOf<T> = <T as frame_system::Config>::Hash;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Assets: pallet_assets::<Instance1>,
		Balances: pallet_balances,
		Contracts: pallet_contracts,
		Nfts: pallet_nfts::<Instance1>,
		Timestamp: pallet_timestamp,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type Block = frame_system::mocking::MockBlock<Test>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Test {}

impl pallet_contracts::Config for Test {
	type AddressGenerator = DefaultAddressGenerator;
	type ApiVersion = ();
	type CallFilter = ();
	// TestFilter;
	type CallStack = [Frame<Self>; 5];
	type ChainExtension = ();
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	// TestDebug;
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type Environment = ();
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type MaxCodeLen = ConstU32<{ 100 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type MaxStorageKeyLen = ConstU32<128>;
	type MaxTransientStorageSize = ConstU32<{ 1024 * 1024 }>;
	type Migrations = ();
	// crate::migration::codegen::BenchMigrations;
	type Randomness = Test;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Schedule = MySchedule;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ();
	// UnstableInterface;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = ();
	type WeightPrice = ();
	// Self;
	type Xcm = ();
}

type AssetsInstance = pallet_assets::Instance1;
#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig as pallet_assets::DefaultConfig)]
impl pallet_assets::Config<AssetsInstance> for Test {
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<u64>;
	type Freezer = ();
	type RuntimeEvent = RuntimeEvent;
}

type NftsInstance = pallet_nfts::Instance1;
impl pallet_nfts::Config<NftsInstance> for Test {
	type ApprovalsLimit = ConstU32<10>;
	type AttributeDepositBase = ConstU64<1>;
	type BlockNumberProvider = frame_system::Pallet<Test>;
	type CollectionApprovalDeposit = ConstU64<1>;
	type CollectionBalanceDeposit = ConstU64<1>;
	type CollectionDeposit = ConstU64<2>;
	type CollectionId = u32;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type Currency = Balances;
	type DepositPerByte = ConstU64<1>;
	type Features = Features;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type ItemDeposit = ConstU64<1>;
	type ItemId = u32;
	type KeyLimit = ConstU32<50>;
	type Locker = ();
	type MaxAttributesPerCall = ConstU32<2>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type MaxTips = ConstU32<10>;
	type MetadataDepositBase = ConstU64<1>;
	type OffchainPublic = Noop;
	type OffchainSignature = Noop;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type WeightInfo = ();
}

parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct Noop;

impl IdentifyAccount for Noop {
	type AccountId = AccountId;

	fn into_account(self) -> Self::AccountId {
		0
	}
}

impl Verify for Noop {
	type Signer = Noop;

	fn verify<L: Lazy<[u8]>>(
		&self,
		_msg: L,
		_signer: &<Self::Signer as IdentifyAccount>::AccountId,
	) -> bool {
		false
	}
}

parameter_types! {
	pub MySchedule: Schedule<Test> = {

		<Schedule<Test>>::default()
	};
}

impl frame_support::traits::Randomness<HashOf<Test>, BlockNumberFor<Test>> for Test {
	fn random(_subject: &[u8]) -> (HashOf<Test>, BlockNumberFor<Test>) {
		(Default::default(), Default::default())
	}
}
