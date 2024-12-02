use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::AccountId;

use super::utils;
use crate::tests::{BLOB_3_6_4, BLOB_3_7_0, HASH_3_6_4, HASH_3_7_0};
use crate::types::DeploymentInfo;

#[tokio::test]
async fn test_downgrade_contract() {
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

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_7_0,
            "version": "3.7.0",
            "is_latest": true,
            "downgrade_hash": HASH_3_6_4 // Allow to downgrade for version 3.6.4
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "add_release_blob")
        .deposit(NearToken::from_yoctonear(1))
        .args(BLOB_3_7_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let new_contract_id: AccountId = "aurora.factory-owner.test.near".parse().unwrap();

    let result = factory_owner
        .call(factory.id(), "deploy")
        .args_json(json!({
            "new_contract_id": new_contract_id.clone(),
            "init_method": "new",
            "init_args": json!({
                "chain_id": 1_313_161_559,
                "owner_id": factory_owner.id(),
                "upgrade_delay_blocks": 0,
                "key_manager": factory_owner.id(),
                "initial_hashchain": null
            })
        }))
        .max_gas()
        .deposit(NearToken::from_near(25))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success());

    let deployments: Vec<DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(deployments[0].version, "3.7.0".parse().unwrap());

    let result = factory_owner
        .call(factory.id(), "downgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "contract_id": new_contract_id.clone(),
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.6.4");

    let deployments: Vec<DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(deployments[0].version, "3.6.4".parse().unwrap());
}
