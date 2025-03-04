use near_sdk::{near, BorshStorageKey};

/// Prefixes for the `near-sdk` collections that are used in the smart contract.
#[derive(BorshStorageKey)]
#[near(serializers = [borsh])]
pub enum Prefix {
    Blobs,
    Deployments,
    Releases,
    LatestRelease,
}
