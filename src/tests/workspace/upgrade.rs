use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::AccountId;

use super::utils;
use crate::tests::{BLOB_3_4_0, BLOB_3_5_0, HASH_3_4_0, HASH_3_5_0, MIGRATION_GAS};

#[tokio::test]
async fn test_upgrade_contract() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_4_0,
            "version": "3.4.0",
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
        .args(BLOB_3_4_0.to_vec())
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
            "hash": HASH_3_5_0,
            "version": "3.5.0",
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
        .args(BLOB_3_5_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_5_0, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.5.0");
}

#[tokio::test]
async fn test_upgrade_contract_to_previous_version() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_5_0,
            "version": "3.5.0",
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
        .args(BLOB_3_5_0.to_vec())
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
            "hash": HASH_3_4_0,
            "version": "3.4.0",
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
        .args(BLOB_3_4_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_4_0, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_failure()); // Couldn't be upgraded to the previous version.

    // Check that the version hasn't been changed.
    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.5.0");
}

#[tokio::test]
async fn test_unrestricted_upgrade_contract() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": HASH_3_5_0,
            "version": "3.5.0",
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
        .args(BLOB_3_5_0.to_vec())
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
            "hash": HASH_3_4_0,
            "version": "3.4.0",
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
        .args(BLOB_3_4_0.to_vec())
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "unrestricted_upgrade")
        .deposit(NearToken::from_yoctonear(1))
        .args_json((&new_contract_id, HASH_3_4_0, MIGRATION_GAS))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
    // Check that the version has been changed to 3.4.0.
    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.4.0");
}
