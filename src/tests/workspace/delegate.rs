use super::utils;
use crate::tests::{BLOB_3_4_0, HASH_3_4_0};
use crate::types::FunctionCallArgs;
use near_gas::NearGas;
use near_sdk::near;
use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use std::str::FromStr;

#[tokio::test]
async fn test_delegate_execution() {
    let (factory_owner, factory, contract_id) = create_factory().await;

    let result = factory_owner
        .call(factory.id(), "delegate_execution")
        .args_json(json!({
            "receiver_id": &contract_id,
            "actions": vec![FunctionCallArgs {
                function_name: "set_owner".to_string(),
                arguments: near_sdk::borsh::to_vec(&contract_id).map(Into::into).unwrap(),
                amount: NearToken::from_near(0),
                gas: NearGas::from_tgas(5)
            }]
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    let bytes = factory_owner
        .call(&contract_id, "get_owner")
        .view()
        .await
        .unwrap()
        .result;
    let owner = AccountId::from_str(&String::from_utf8(bytes).unwrap()).unwrap();
    assert_eq!(owner, contract_id);
}

#[near(serializers = [borsh])]
struct SetOwner {
    new_owner: AccountId,
}

#[tokio::test]
async fn test_delegate_pause() {
    let (factory_owner, factory, contract_id) = create_factory().await;

    let result = factory_owner
        .call(factory.id(), "delegate_pause")
        .args_json(json!({
            "receiver_id": &contract_id,
            "pause_method_name": "pause_contract"
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    dbg!(&result);
    assert!(result.is_success());

    let result = factory_owner
        .call(&contract_id, "set_owner")
        .args_borsh(
            near_sdk::borsh::to_vec(&SetOwner {
                new_owner: "new_owner".parse().unwrap(),
            })
            .unwrap(),
        )
        .transact()
        .await
        .unwrap();
    assert!(result.is_failure());
}

async fn create_factory() -> (Account, Contract, AccountId) {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .args_json(json!({
            "hash": HASH_3_4_0,
            "version": "3.4.0",
            "is_latest": true,
            "downgrade_hash": null
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    let result = factory_owner
        .call(factory.id(), "add_release_blob")
        .args(BLOB_3_4_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    let contract_id: AccountId = "aurora-1.factory-owner.test.near".parse().unwrap();
    let init_args = json!({
        "chain_id": 1_313_161_559,
        "owner_id": factory_owner.id(),
        "upgrade_delay_blocks": 0,
        "key_manager": factory_owner.id(),
        "initial_hashchain": null
    });
    let result = factory_owner
        .call(factory.id(), "deploy")
        .args_json(json!({
            "new_contract_id": contract_id.clone(),
            "init_method": "new",
            "init_args": init_args,
        }))
        .max_gas()
        .deposit(NearToken::from_near(25))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    (factory_owner, factory, contract_id)
}
