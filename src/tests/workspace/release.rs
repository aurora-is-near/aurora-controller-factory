use near_sdk::serde_json::json;

use super::utils;
use crate::types::ReleaseInfo;

#[tokio::test]
async fn test_add_new_release() {
    let (factory_owner, factory, _) = utils::crate_factory().await.unwrap();

    let result = factory_owner
        .call(factory.id(), "add_release_info")
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
