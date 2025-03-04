// The serialization of the wasm file is needed, because the upgrade transaction
// from near-plugins is waiting for the wasm file to be serialized in `borsh`.
fn main() -> anyhow::Result<()> {
    let data = std::fs::read("res/aurora-controller-factory.wasm")?;
    let borsh = near_sdk::borsh::to_vec(&data)?;
    std::fs::write("res/aurora-controller-factory-borsh.wasm", borsh)?;

    Ok(())
}
