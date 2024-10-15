// Tests in this module are used `near-sdk`.
mod sdk;
// Tests in this module are used `near-workspaces-rs`.
mod workspace;

pub const HASH_3_6_4: &str = "b7f368ff6aeb0e98ede5e5116f6462704ed97e512bf909a2aa59f0ebfb9716cb";
pub const HASH_3_7_0: &str = "4c6d9305a7694deaf78fabc8f15896b8073507da283103f46ed509ed8a2bb6b0";
pub const BLOB_3_6_4: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.6.4.wasm");
pub const BLOB_3_7_0: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.7.0.wasm");

pub const MIGRATION_GAS: u64 = 50_000_000_000_000; // 50 TGas
