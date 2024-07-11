use crate::Role;
use near_sdk::serde_json::json;
use near_workspaces::network::Sandbox;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract, Worker};

const FACTORY_OWNER: &str = "factory-owner";
const AURORA_FACTORY_CONTRACT_PATH: &str =
    "target/wasm32-unknown-unknown/release/aurora_controller_factory.wasm";
pub const INITIAL_BALANCE: NearToken = NearToken::from_near(200);

pub async fn crate_factory() -> anyhow::Result<(Account, Contract, Worker<Sandbox>)> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account().unwrap();
    let factory_owner = root
        .create_subaccount(FACTORY_OWNER)
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await
        .unwrap()
        .into_result()
        .unwrap();

    let wasm = std::fs::read(AURORA_FACTORY_CONTRACT_PATH)?;
    let contract = factory_owner.deploy(&wasm).await?.result;

    let result = factory_owner
        .call(factory_owner.id(), "new")
        .args_json(json!({"owner_id": factory_owner.id()}))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    grant_role(&contract, &factory_owner, factory_owner.id(), Role::DAO).await;
    grant_role(&contract, &factory_owner, factory_owner.id(), Role::Updater).await;

    Ok((factory_owner, contract, worker))
}

pub async fn grant_role(
    contract: &Contract,
    account: &Account,
    account_id: &AccountId,
    role: Role,
) {
    let result = account
        .call(contract.id(), "acl_grant_role")
        .args_json(json!({
            "role": role,
            "account_id": account_id
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());
}
