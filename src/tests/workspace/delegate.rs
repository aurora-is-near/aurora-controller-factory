use super::utils;
use crate::tests::{BLOB_3_4_0, HASH_3_4_0};
use crate::types::FunctionCallArgs;
use near_sdk::borsh::{self, BorshSerialize};
use near_sdk::serde_json::json;
use near_sdk::Gas;
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
                arguments: contract_id.try_to_vec().map(Into::into).unwrap(),
                amount: 0,
                gas: Gas::ONE_TERA * 5
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

#[derive(BorshSerialize)]
struct SetOwner {
    new_owner: near_sdk::AccountId,
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
            SetOwner {
                new_owner: "new_owner".parse().unwrap(),
            }
            .try_to_vec()
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
