fn main() -> anyhow::Result<()> {
    let data =
        std::fs::read("target/wasm32-unknown-unknown/release/aurora_controller_factory.wasm")?;
    let borsh = near_sdk::borsh::to_vec(&data)?;
    std::fs::write("res/aurora-controller-factory.wasm", borsh)?;

    Ok(())
}
