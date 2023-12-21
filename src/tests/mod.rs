// Tests in this module are used `near-sdk`.
mod sdk;
// Tests in this module are used `near-workspaces-rs`.
mod workspace;

pub const HASH_3_4_0: &str = "9316bf4c7aa0913f26ef8eebdcb11f3c63bb88c65eb717abfec8ade1b707620c";
pub const HASH_3_5_0: &str = "45f97119f38321864f1815c0e8a88753086f5433f6681810faf049d73d7de4b1";
pub const BLOB_3_4_0: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.4.0.wasm");
pub const BLOB_3_5_0: &[u8] = include_bytes!("../../res/aurora-mainnet-silo-3.5.0.wasm");
