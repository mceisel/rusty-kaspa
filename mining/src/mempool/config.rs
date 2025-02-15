use kaspa_consensus_core::constants::TX_VERSION;

pub(crate) const DEFAULT_MAXIMUM_TRANSACTION_COUNT: u64 = 1_000_000;

pub(crate) const DEFAULT_TRANSACTION_EXPIRE_INTERVAL_SECONDS: u64 = 60;
pub(crate) const DEFAULT_TRANSACTION_EXPIRE_SCAN_INTERVAL_SECONDS: u64 = 10;
pub(crate) const DEFAULT_ORPHAN_EXPIRE_INTERVAL_SECONDS: u64 = 60;
pub(crate) const DEFAULT_ORPHAN_EXPIRE_SCAN_INTERVAL_SECONDS: u64 = 10;

pub(crate) const DEFAULT_MAXIMUM_ORPHAN_TRANSACTION_MASS: u64 = 100_000;

// TODO: when rusty-kaspa nodes run most of the network, consider increasing this value
pub(crate) const DEFAULT_MAXIMUM_ORPHAN_TRANSACTION_COUNT: u64 = 50;

/// DEFAULT_MINIMUM_RELAY_TRANSACTION_FEE specifies the minimum transaction fee for a transaction to be accepted to
/// the mempool and relayed. It is specified in sompi per 1kg (or 1000 grams) of transaction mass.
pub(crate) const DEFAULT_MINIMUM_RELAY_TRANSACTION_FEE: u64 = 1000;

/// Standard transaction version range might be different from what consensus accepts, therefore
/// we define separate values in mempool.
/// However, currently there's exactly one transaction version, so mempool accepts the same version
/// as consensus.
pub(crate) const DEFAULT_MINIMUM_STANDARD_TRANSACTION_VERSION: u16 = TX_VERSION;
pub(crate) const DEFAULT_MAXIMUM_STANDARD_TRANSACTION_VERSION: u16 = TX_VERSION;

#[derive(Clone, Debug)]
pub struct Config {
    pub maximum_transaction_count: u64,
    pub transaction_expire_interval_daa_score: u64,
    pub transaction_expire_scan_interval_daa_score: u64,
    pub transaction_expire_scan_interval_milliseconds: u64,
    pub orphan_expire_interval_daa_score: u64,
    pub orphan_expire_scan_interval_daa_score: u64,
    pub maximum_orphan_transaction_mass: u64,
    pub maximum_orphan_transaction_count: u64,
    pub accept_non_standard: bool,
    pub maximum_mass_per_block: u64,
    pub minimum_relay_transaction_fee: u64,
    pub minimum_standard_transaction_version: u16,
    pub maximum_standard_transaction_version: u16,
}

impl Config {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        maximum_transaction_count: u64,
        transaction_expire_interval_daa_score: u64,
        transaction_expire_scan_interval_daa_score: u64,
        transaction_expire_scan_interval_seconds: u64,
        orphan_expire_interval_daa_score: u64,
        orphan_expire_scan_interval_daa_score: u64,
        maximum_orphan_transaction_mass: u64,
        maximum_orphan_transaction_count: u64,
        accept_non_standard: bool,
        maximum_mass_per_block: u64,
        minimum_relay_transaction_fee: u64,
        minimum_standard_transaction_version: u16,
        maximum_standard_transaction_version: u16,
    ) -> Self {
        Self {
            maximum_transaction_count,
            transaction_expire_interval_daa_score,
            transaction_expire_scan_interval_daa_score,
            transaction_expire_scan_interval_milliseconds: transaction_expire_scan_interval_seconds,
            orphan_expire_interval_daa_score,
            orphan_expire_scan_interval_daa_score,
            maximum_orphan_transaction_mass,
            maximum_orphan_transaction_count,
            accept_non_standard,
            maximum_mass_per_block,
            minimum_relay_transaction_fee,
            minimum_standard_transaction_version,
            maximum_standard_transaction_version,
        }
    }

    /// Build a default config.
    /// The arguments should be obtained from the current consensus [`kaspa_consensus_core::config::params::Params`] instance.
    pub fn build_default(target_milliseconds_per_block: u64, relay_non_std_transactions: bool, max_block_mass: u64) -> Self {
        Self {
            maximum_transaction_count: DEFAULT_MAXIMUM_TRANSACTION_COUNT,
            transaction_expire_interval_daa_score: DEFAULT_TRANSACTION_EXPIRE_INTERVAL_SECONDS * 1000 / target_milliseconds_per_block,
            transaction_expire_scan_interval_daa_score: DEFAULT_TRANSACTION_EXPIRE_SCAN_INTERVAL_SECONDS * 1000
                / target_milliseconds_per_block,
            transaction_expire_scan_interval_milliseconds: DEFAULT_TRANSACTION_EXPIRE_SCAN_INTERVAL_SECONDS * 1000,
            orphan_expire_interval_daa_score: DEFAULT_ORPHAN_EXPIRE_INTERVAL_SECONDS * 1000 / target_milliseconds_per_block,
            orphan_expire_scan_interval_daa_score: DEFAULT_ORPHAN_EXPIRE_SCAN_INTERVAL_SECONDS * 1000 / target_milliseconds_per_block,
            maximum_orphan_transaction_mass: DEFAULT_MAXIMUM_ORPHAN_TRANSACTION_MASS,
            maximum_orphan_transaction_count: DEFAULT_MAXIMUM_ORPHAN_TRANSACTION_COUNT,
            accept_non_standard: relay_non_std_transactions,
            maximum_mass_per_block: max_block_mass,
            minimum_relay_transaction_fee: DEFAULT_MINIMUM_RELAY_TRANSACTION_FEE,
            minimum_standard_transaction_version: DEFAULT_MINIMUM_STANDARD_TRANSACTION_VERSION,
            maximum_standard_transaction_version: DEFAULT_MAXIMUM_STANDARD_TRANSACTION_VERSION,
        }
    }
}
