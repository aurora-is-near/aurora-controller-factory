use near_sdk::near;
use near_sdk::serde_json::json;
use near_sdk::Gas;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract};
use std::str::FromStr;

use super::utils;
use crate::tests::{BLOB_3_6_4, HASH_3_6_4};
use crate::types::FunctionCallArgs;

#[tokio::test]
async fn test_delegate_execution() {
    let (factory_owner, factory, contract_id) = create_factory().await;

    let result = factory_owner
        .call(factory.id(), "delegate_execution")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "receiver_id": &contract_id,
            "actions": vec![FunctionCallArgs {
                function_name: "set_owner".to_string(),
                arguments: near_sdk::borsh::to_vec(&contract_id).map(Into::into).unwrap(),
                amount: NearToken::from_near(0),
                gas: Gas::from_tgas(5)
            }]
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

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
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "receiver_id": &contract_id,
            "pause_method_name": "pause_contract"
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

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

#[tokio::test]
async fn test_delegate_pause_via_plugins() {
    let (_, factory, _) = create_factory().await;

    let result = factory
        .call("delegate_pause")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "receiver_id": &factory.id(),
            "pause_method_name": "pa_pause_feature",
            "pause_arguments": {
                "key": "ALL"
            }
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
}

async fn create_factory() -> (Account, Contract, AccountId) {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_6_4,
            "version": "3.6.4",
            "is_latest": true,
            "downgrade_hash": null
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "add_release_blob")
        .deposit(NearToken::from_yoctonear(1))
        .args(BLOB_3_6_4.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

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
    assert!(result.is_success(), "{result:#?}");

    (factory_owner, factory, contract_id)
}
