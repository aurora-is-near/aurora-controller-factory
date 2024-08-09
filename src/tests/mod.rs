// Tests in this module are used `near-sdk`.
mod sdk;
// Tests in this module are used `near-workspaces-rs`.
mod workspace;

pub const HASH_3_4_0: &str = "c8f8468675bc1de2b12eb6a11819be20e91a1ae169fa6f10d997a6fd19f84bf9";
pub const HASH_3_5_0: &str = "85b781eabb3b39e7f975bd803c6c7e80fe8dff78724c6c204a7eaf9f0cefbcbf";
pub const BLOB_3_4_0: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.4.0.wasm");
pub const BLOB_3_5_0: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.5.0.wasm");

pub const MIGRATION_GAS: u64 = 50_000_000_000_000; // 50 TGas
