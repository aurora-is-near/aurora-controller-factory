use super::utils;
use crate::tests::{BLOB_3_6_4, HASH_3_6_4};
use crate::types::ReleaseInfo;
use near_sdk::{serde_json::json, NearToken};

#[tokio::test]
async fn test_add_new_release() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921",
            "version": "1.0.1",
            "is_latest": true,
            "downgrade_hash": null
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_success(), "{result:#?}");

    let result = factory_owner
        .call(factory.id(), "add_release_info")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "hash": "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234",
            "version": "1.0.0",
            "is_latest": true,
            "downgrade_hash": null
        }))
        .transact()
        .await
        .unwrap();
    assert!(result.is_failure());

    let releases: Vec<ReleaseInfo> = factory_owner
        .call(factory.id(), "get_releases")
        .view()
        .await
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(releases.len(), 1);
}

#[tokio::test]
#[should_panic(expected = "HostError(GasLimitExceeded")]
async fn test_get_release_blob() {
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

    let blob = factory_owner
        .call(factory.id(), "get_release_blob")
        .args_json(json!({ "hash": HASH_3_6_4 }))
        .view()
        .await
        .unwrap()
        .result;

    assert_eq!(blob, BLOB_3_6_4);
}
