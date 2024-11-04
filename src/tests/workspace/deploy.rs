use near_sdk::serde_json::json;
use near_workspaces::types::NearToken;
use near_workspaces::AccountId;
use std::collections::BTreeMap;

use super::utils;
use crate::tests::{BLOB_3_6_4, BLOB_3_7_0, HASH_3_6_4, HASH_3_7_0, MIGRATION_GAS};
use crate::types::DeploymentInfo;

#[tokio::test]
async fn test_deploy_contract() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();
    let version: String = factory_owner
        .view(factory.id(), "version")
        .await
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(&version, env!("CARGO_PKG_VERSION"));

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
        .max_gas()
        .deposit(NearToken::from_near(25))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let deployments: Vec<DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(deployments.len(), 1);

    let result = factory_owner.view(&new_contract_id, "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.6.4");
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_deploy_more_than_one_contract() {
    let (factory_owner, factory, worker) = utils::crate_factory().await.unwrap();

    let version: String = factory_owner
        .view(factory.id(), "version")
        .await
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(&version, env!("CARGO_PKG_VERSION"));

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

    let new_1_contract_id: AccountId = "aurora-1.factory-owner.test.near".parse().unwrap();
    let init_args_1 = json!({
        "chain_id": 1_313_161_559,
        "owner_id": factory_owner.id(),
        "upgrade_delay_blocks": 0,
        "key_manager": factory_owner.id(),
        "initial_hashchain": null
    });
    let result = factory_owner
        .call(factory.id(), "deploy")
        .args_json(json!({
            "new_contract_id": new_1_contract_id.clone(),
            "init_method": "new",
            "init_args": init_args_1
        }))
        .max_gas()
        .deposit(NearToken::from_near(25))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
    let deploy_time_1 = worker
        .view_block()
        .block_hash(result.unwrap().outcome().block_hash)
        .await
        .unwrap()
        .timestamp();

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

    let new_2_contract_id: AccountId = "aurora-2.factory-owner.test.near".parse().unwrap();
    let init_args_2 = json!({
        "chain_id": 1_313_161_559,
        "owner_id": factory_owner.id(),
        "upgrade_delay_blocks": 0,
        "key_manager": factory_owner.id(),
        "initial_hashchain": null
    });
    let result = factory_owner
        .call(factory.id(), "deploy")
        .args_json(json!({
            "new_contract_id": new_2_contract_id.clone(),
            "blob_hash": HASH_3_7_0,
            "init_method": "new",
            "init_args": &init_args_2
        }))
        .max_gas()
        .deposit(NearToken::from_near(25))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");
    let deploy_time_2 = worker
        .view_block()
        .block_hash(result.unwrap().outcome().block_hash)
        .await
        .unwrap()
        .timestamp();

    let deployments: Vec<DeploymentInfo> = factory_owner
        .view(factory.id(), "get_deployments")
        .await
        .unwrap()
        .json()
        .unwrap();
    assert_eq!(deployments.len(), 2);
    assert_eq!(
        deployments,
        vec![
            DeploymentInfo {
                hash: HASH_3_6_4.to_string(),
                version: "3.6.4".parse().unwrap(),
                deployment_time: deploy_time_1,
                upgrade_times: [(deploy_time_1, "3.6.4".parse().unwrap())].into(),
                init_args: near_sdk::serde_json::to_string(&init_args_1).unwrap(),
            },
            DeploymentInfo {
                hash: HASH_3_7_0.to_string(),
                version: "3.7.0".parse().unwrap(),
                deployment_time: deploy_time_2,
                upgrade_times: [(deploy_time_2, "3.7.0".parse().unwrap())].into(),
                init_args: near_sdk::serde_json::to_string(&init_args_2).unwrap(),
            }
        ]
    );
}

#[tokio::test]
async fn test_add_deployment_info_for_existed_contract() {
    let (factory_owner, factory, worker) = utils::crate_factory().await.unwrap();

    // Deploy silo contract 3.6.4 manually.
    let silo_contract = {
        let contract_id = worker
            .root_account()
            .unwrap()
            .create_subaccount("silo")
            .initial_balance(utils::INITIAL_BALANCE)
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();
        let result = contract_id.deploy(BLOB_3_6_4).await.unwrap();
        assert!(result.is_success(), "{:?}", result.details);
        let silo_contract = result.unwrap();

        let result = silo_contract
            .call("new")
            .args_json(json!({
                "chain_id": 1_313_161_559,
                "owner_id": factory_owner.id(),
                "upgrade_delay_blocks": 0,
                "key_manager": factory_owner.id(),
                "initial_hashchain": null
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:#?}");

        silo_contract
    };

    // Adding deployment info of previously deployed silo contract to the controller contract.
    {
        let deployment_info = DeploymentInfo {
            hash: HASH_3_6_4.to_string(),
            version: "3.6.4".parse().unwrap(),
            deployment_time: 0,
            upgrade_times: BTreeMap::default(),
            init_args: String::default(),
        };

        let result = factory_owner
            .call(factory.id(), "add_deployment_info")
            .deposit(NearToken::from_yoctonear(1))
            .args_json(json!({
                "contract_id": silo_contract.id(),
                "deployment_info": &deployment_info
            }))
            .transact()
            .await
            .unwrap();
        assert!(result.is_success(), "{result:#?}");
    }

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
        .args_json(json!({
            "contract_id": silo_contract.id(),
            "state_migration_gas": MIGRATION_GAS
        }))
        .max_gas()
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    // Check that the version has been changed to 3.7.0.
    let result = factory_owner.view(silo_contract.id(), "get_version").await;
    let version = String::from_utf8(result.unwrap().result).unwrap();
    assert_eq!(version.trim_end(), "3.7.0");
}
