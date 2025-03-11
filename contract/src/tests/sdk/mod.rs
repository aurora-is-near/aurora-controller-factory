use near_sdk::{AccountId, NearToken};

use crate::types::ReleaseInfo;
use crate::AuroraControllerFactory;

#[macro_use]
mod macros;

#[test]
fn test_controller_version() {
    set_env!(predecessor_account_id: predecessor_account_id());
    let contract = AuroraControllerFactory::new(dao());
    assert_eq!(contract.version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn test_adding_blob() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        input: vec![1; 256],
        attached_deposit: NearToken::from_yoctonear(1),
    );
    let mut contract = AuroraControllerFactory::new(dao());

    contract.add_release_info(
        "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string(),
        "1.0.0".parse().unwrap(),
        true,
        None,
        None,
    );
    contract.add_release_blob();

    let releases = contract.get_releases();
    assert_eq!(
        releases,
        vec![ReleaseInfo {
            hash: "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string(),
            version: "1.0.0".parse().unwrap(),
            is_blob_exist: true,
            downgrade_hash: None,
            description: None
        }]
    );

    set_env!(
        predecessor_account_id: predecessor_account_id(),
        input: vec![2; 256],
        attached_deposit: NearToken::from_yoctonear(1),
    );

    contract.add_release_info(
        "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921".to_string(),
        "1.0.1".parse().unwrap(),
        true,
        Some("2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string()),
        None,
    );
    contract.add_release_blob();

    let releases = contract.get_releases();
    assert_eq!(
        releases,
        vec![
            ReleaseInfo {
                hash: "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234"
                    .to_string(),
                version: "1.0.0".parse().unwrap(),
                is_blob_exist: true,
                ..Default::default()
            },
            ReleaseInfo {
                hash: "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921"
                    .to_string(),
                version: "1.0.1".parse().unwrap(),
                is_blob_exist: true,
                downgrade_hash: Some(
                    "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string()
                ),
                description: None
            }
        ]
    );
}

#[test]
#[should_panic = "release info doesn't exist for the hash"]
fn test_adding_blob_without_adding_hash() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        input: vec![1; 256],
        attached_deposit: NearToken::from_yoctonear(1),
    );

    let mut contract = AuroraControllerFactory::new(dao());
    contract.add_release_blob();
}

#[test]
fn test_check_latest_release() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        input: vec![1; 256],
        attached_deposit: NearToken::from_yoctonear(1),
    );
    let mut contract = AuroraControllerFactory::new(dao());

    contract.add_release_info(
        "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string(),
        "1.0.0".parse().unwrap(),
        true,
        None,
        None,
    );
    contract.add_release_blob();

    let latest_hash = contract.get_latest_release_hash();
    assert_eq!(
        &latest_hash,
        "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234"
    );

    set_env!(
        predecessor_account_id: predecessor_account_id(),
        input: vec![2; 256],
        attached_deposit: NearToken::from_yoctonear(1),
    );

    contract.add_release_info(
        "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921".to_string(),
        "1.0.1".parse().unwrap(),
        true,
        Some("2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string()),
        None,
    );
    contract.add_release_blob();

    let latest_hash = contract.get_latest_release_hash();
    assert_eq!(
        &latest_hash,
        "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921"
    );
}

#[test]
#[should_panic = "the latest release hash hasn't been set yet"]
fn test_check_latest_release_hash_without_adding() {
    set_env!(predecessor_account_id: predecessor_account_id());
    let contract = AuroraControllerFactory::new(dao());
    let _ = contract.get_latest_release_hash();
}

#[test]
#[should_panic = "version of new latest should be higher than previous"]
fn test_set_latest_with_lower_version() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        attached_deposit: NearToken::from_yoctonear(1),
    );
    let mut contract = AuroraControllerFactory::new(dao());

    contract.add_release_info(
        "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string(),
        "1.0.0".parse().unwrap(),
        false,
        None,
        None,
    );
    contract.add_release_info(
        "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921".to_string(),
        "1.0.1".parse().unwrap(),
        true,
        Some("2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string()),
        None,
    );

    contract.set_latest_release(
        &"2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_owned(),
    );
}

#[test]
#[should_panic = "version of new latest should be higher than previous"]
fn test_add_latest_with_lower_version() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        attached_deposit: NearToken::from_yoctonear(1),
    );

    let mut contract = AuroraControllerFactory::new(dao());

    contract.add_release_info(
        "f5c22e35d04167e37913e7963ce033b1f3d17a924a4e6fe5fc95af1224051921".to_string(),
        "1.0.1".parse().unwrap(),
        true,
        None,
        None,
    );

    contract.add_release_info(
        "2661920f2409dd6c8adeb0c44972959f232b6429afa913845d0fd95e7e768234".to_string(),
        "1.0.0".parse().unwrap(),
        true,
        None,
        None,
    );
}

#[test]
#[should_panic = "pause method: some_pause_method is not allowed"]
fn test_use_not_allowed_pause_method() {
    set_env!(
        predecessor_account_id: predecessor_account_id(),
        attached_deposit: NearToken::from_yoctonear(1),
    );

    let mut contract = AuroraControllerFactory::new(dao());
    contract.delegate_pause(new_engine(), Some("some_pause_method".to_string()), None);
}

fn dao() -> Option<AccountId> {
    "alice.near".parse().ok()
}

fn predecessor_account_id() -> AccountId {
    "alice.near".parse().unwrap()
}

fn new_engine() -> AccountId {
    "new_engine".parse().unwrap()
}
