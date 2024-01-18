use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::IntoStorageKey;

/// Prefixes for the `near-sdk` collections that are used in the smart contract.
#[derive(BorshSerialize, BorshDeserialize)]
pub enum Prefix {
    Blobs,
    Deployments,
    Releases,
    LatestRelease,
}

impl IntoStorageKey for Prefix {
    fn into_storage_key(self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}
