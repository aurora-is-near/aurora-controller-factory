use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::AccountId;
use std::collections::BTreeMap;

use super::utils;
use crate::tests::{BLOB_3_6_4, BLOB_3_7_0, HASH_3_6_4, HASH_3_7_0, MIGRATION_GAS};
use crate::types::DeploymentInfo;

#[tokio::test]
async fn test_upgrade_contract() {
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
        .deposit(NearToken::from_near(25))
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
            "downgrade_hash": null
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

    let result = factory_owner
        .call(factory.id(), "upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_7_0, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.7.0");
}

#[tokio::test]
async fn test_upgrade_contract_to_previous_version() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_7_0,
            "version": "3.7.0",
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
        .deposit(NearToken::from_near(25))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_6_4,
            "version": "3.6.4",
            "is_latest": false,
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
        .call(factory.id(), "upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_6_4, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_failure()); // Couldn't be upgraded to the previous version.

    // Check that the version hasn't been changed.
    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.7.0");
}

#[tokio::test]
async fn test_unrestricted_upgrade_contract() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_7_0,
            "version": "3.7.0",
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
        .deposit(NearToken::from_near(25))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_6_4,
            "version": "3.6.4",
            "is_latest": false,
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
        .call(factory.id(), "unrestricted_upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_6_4, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
    // Check that the version has been changed to 3.6.4.
    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.6.4");
}

#[tokio::test]
async fn test_upgrade_contract_with_small_gas_for_migration() {
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
        .deposit(NearToken::from_near(25))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let deployments_info: BTreeMap<AccountId, DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_7_0,
            "version": "3.7.0",
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
        .args(BLOB_3_7_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((
            &new_contract_id,
            HASH_3_7_0,
            near_gas::NearGas::from_gas(1).as_gas(), // 1 Tas too small amount for migration
        ))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success()); // But the promise for the state migration will be failed.
                                  // So, the upgrade won't be successful.

    // Check that the deployment into hasn't been changed.
    let result: BTreeMap<AccountId, DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(deployments_info, result);

    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.6.4");
}
