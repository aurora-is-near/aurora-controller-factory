use crate::event::Event;
use crate::types::{DeploymentInfo, FunctionCallArgs, ReleaseInfo, Version};
use near_plugins::{
    access_control, access_control_any, AccessControlRole, AccessControllable, Ownable, Pausable,
    Upgradable,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, Value};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, Gas, PanicOnDefault, Promise, PromiseResult,
    PublicKey,
};

mod event;
mod keys;
#[cfg(test)]
mod tests;
pub mod types;
pub mod utils;

/// Gas needed for initialization deployed contract.
const NEW_GAS: Gas = Gas(Gas::ONE_TERA.0 * 100);

/// Gas needed to upgrade contract.
const UPGRADE_GAS: Gas = Gas(Gas::ONE_TERA.0 * 230);

/// Gas needed to call the `add_deployment` callback.
const ADD_DEPLOYMENT_GAS: Gas = Gas(Gas::ONE_TERA.0 * 5);

/// Amount of gas used by `delegate_pause` in the controller contract
/// without taking into account the gas consumed by the promise.
const OUTER_DELEGATE_PAUSE_GAS: Gas = Gas(Gas::ONE_TERA.0 * 10);

/// Allowed pause methods.
const ALLOWED_PAUSE_METHODS: &[&str] = &["pause_contract", "pa_pause_feature"];

macro_rules! panic {
    ($($args:tt)*) => {
        env::panic_str(&format!("{}", format_args!($($args)*)))
    };
}

/// ACL Roles of the contract.
#[derive(AccessControlRole, Deserialize, Serialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Role {
    DAO,
    Deployer,
    Pauser,
    Releaser,
    Updater,
}

///
#[near_bindgen]
#[derive(
    Debug, BorshDeserialize, BorshSerialize, Ownable, PanicOnDefault, Pausable, Upgradable,
)]
#[ownable]
#[access_control(role_type(Role))]
#[upgradable(access_control_roles(
    code_stagers(Role::DAO),
    code_deployers(Role::DAO),
    duration_initializers(Role::DAO),
    duration_update_stagers(Role::DAO),
    duration_update_appliers(Role::DAO),
))]
#[pausable(manager_roles(Role::DAO, Role::Pauser))]
pub struct AuroraControllerFactory {
    releases: UnorderedMap<String, ReleaseInfo>,
    blobs: UnorderedMap<String, Vec<u8>>,
    deployments: UnorderedMap<AccountId, DeploymentInfo>,
    latest: LazyOption<ReleaseInfo>,
}

#[near_bindgen]
impl AuroraControllerFactory {
    /// Initializes a new controller contract.
    ///
    /// # Panics
    ///
    /// The function panics if the state of the contract is already exist.
    #[must_use]
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let mut contract = Self {
            releases: UnorderedMap::new(keys::Prefix::Releases),
            blobs: UnorderedMap::new(keys::Prefix::Blobs),
            deployments: UnorderedMap::new(keys::Prefix::Deployments),
            latest: LazyOption::new(keys::Prefix::LatestRelease, None),
        };

        contract.owner_set(Some(owner_id));
        contract.acl_init_super_admin(env::predecessor_account_id());
        contract
    }

    /// Returns version of the controller contract.
    #[must_use]
    pub const fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    /// Attaches new full access key to the controller contract.
    #[access_control_any(roles(Role::DAO))]
    pub fn attach_full_access_key(&mut self, public_key: PublicKey) -> Promise {
        event::emit(
            Event::AttachFullAccessKey,
            json!({"public_key": &public_key}),
        );
        Promise::new(env::current_account_id()).add_full_access_key(public_key)
    }

    /// Delegates an execution of actions to the specified receiver.
    #[access_control_any(roles(Role::DAO))]
    pub fn delegate_execution(
        &self,
        receiver_id: AccountId,
        actions: Vec<FunctionCallArgs>,
    ) -> Promise {
        event::emit(
            Event::DelegatedExecution,
            json!({
                "receiver_id": &receiver_id,
                "actions": &actions
            }),
        );
        actions
            .into_iter()
            .fold(Promise::new(receiver_id), |promise, action| {
                promise.function_call(
                    action.function_name,
                    action.arguments.into(),
                    action.amount.into(),
                    Gas(action.gas.0),
                )
            })
    }

    /// Pauses the contract with provided account id.
    #[access_control_any(roles(Role::DAO, Role::Pauser))]
    pub fn delegate_pause(
        &self,
        receiver_id: AccountId,
        pause_method_name: Option<String>,
    ) -> Promise {
        let function_name = match pause_method_name {
            Some(method) if ALLOWED_PAUSE_METHODS.contains(&method.as_str()) => method,
            Some(method) => panic!("pause method: {method} is not allowed"),
            None => "pause_contract".to_string(), // Aurora Engine pause method name is used by default.
        };
        let gas = env::prepaid_gas() - OUTER_DELEGATE_PAUSE_GAS;

        event::emit(
            Event::DelegatedPause,
            json!({
                "receiver_id": &receiver_id,
                "pause_method_name": &function_name
            }),
        );

        Promise::new(receiver_id).function_call(function_name, vec![], 0, gas)
    }

    /// Adds new contract release info.
    #[access_control_any(roles(Role::DAO))]
    pub fn add_release_info(
        &mut self,
        hash: String,
        version: Version,
        is_latest: bool,
        downgrade_hash: Option<String>,
        description: Option<String>,
    ) {
        assert!(
            self.releases.get(&hash).is_none(),
            "release info for hash: {hash} is already exist"
        );

        let release_info = ReleaseInfo {
            hash: hash.clone(),
            version,
            is_blob_exist: false,
            downgrade_hash,
            description,
        };

        self.releases.insert(&hash, &release_info);

        if is_latest {
            self.set_latest_release(&hash);
        }

        event::emit(Event::AddReleaseInfo, release_info);
    }

    /// Adds bytes of the contract smart contract to the corresponding release info.
    pub fn add_release_blob(&mut self) {
        let blob = env::input().unwrap_or_else(|| panic!("no blob's bytes were provided"));
        let hash = utils::hash_256(&blob);
        let mut release_info = self.releases.get(&hash).unwrap_or_else(|| {
            panic!("release info doesn't exist for the hash: {hash}");
        });

        release_info.is_blob_exist = true;
        self.releases.insert(&hash, &release_info);
        self.blobs.insert(&hash, &blob);

        event::emit(Event::AddBlob, json!({"blob_hash": &hash}));
    }

    /// Marks the release with the hash: `hash` as latest.
    #[access_control_any(roles(Role::DAO, Role::Releaser))]
    pub fn set_latest_release(&mut self, hash: &String) {
        let new_latest = self.releases.get(hash).unwrap_or_else(|| {
            panic!("release info doesn't exist for hash: {hash}");
        });

        if let Some(current_latest) = self.latest.get() {
            assert!(
                current_latest.version < new_latest.version,
                "version of new latest should be higher than previous"
            );
        }

        self.latest.set(&new_latest);
        event::emit(Event::SetLatestReleaseInfo, new_latest);
    }

    /// Removes the release info for hash: `hash`.
    #[access_control_any(roles(Role::DAO))]
    pub fn remove_release(&mut self, hash: &String) {
        let release_info = self.releases.remove(hash).unwrap_or_else(|| {
            panic!("release info doesn't exist for hash: {hash}");
        });
        self.blobs.remove(hash);
        event::emit(Event::RemoveReleaseInfo, release_info);
    }

    /// Returns a list of existing releases for deployment.
    #[must_use]
    pub fn get_releases(&self) -> Vec<ReleaseInfo> {
        self.releases.values_as_vector().to_vec()
    }

    /// Returns a WASM code from the release that corresponds the provided hash.
    #[must_use]
    pub fn get_release_blob(&self, hash: &String) -> Vec<u8> {
        self.blobs
            .get(hash)
            .unwrap_or_else(|| panic!("blob doesn't exist for release info with hash: {hash}"))
    }

    /// Returns a hash of the latest release.
    #[must_use]
    pub fn get_latest_release_hash(&self) -> String {
        self.latest.get().map_or_else(
            || panic!("the latest release hash hasn't been set yet"),
            |r| r.hash,
        )
    }

    /// Returns a WASM code of the latest release.
    #[must_use]
    pub fn get_latest_release_blob(&self) -> Vec<u8> {
        let latest_hash = self.get_latest_release_hash();
        self.get_release_blob(&latest_hash)
    }

    /// Deploys a new contract on the release info that corresponds to the provided hash.
    #[access_control_any(roles(Role::DAO, Role::Deployer))]
    pub fn deploy(
        &self,
        new_contract_id: AccountId,
        init_method: String,
        init_args: Value,
        blob_hash: Option<String>,
    ) -> Promise {
        // Check that the `new_contract_id` wasn't used for another contract before.
        assert!(
            self.deployments.get(&new_contract_id).is_none(),
            "{new_contract_id} is already deployed"
        );

        let blob_hash = blob_hash
            .or_else(|| self.latest.get().map(|r| r.hash))
            .unwrap_or_else(|| panic!("no custom hash nor the latest was provided"));
        let release_info = self
            .releases
            .get(&blob_hash)
            .unwrap_or_else(|| panic!("no release info for hash: {}", &blob_hash));
        let event_metadata =
            json!({"contract_id": &new_contract_id, "release_info": &release_info});
        let code = self
            .blobs
            .get(&blob_hash)
            .unwrap_or_else(|| panic!("blob doesn't exist for hash: {}", &blob_hash));
        let init_args_bytes = near_sdk::serde_json::to_vec(&init_args)
            .unwrap_or_else(|e| panic!("bad format of the init args: {e}"));

        event::emit(Event::Deploy, event_metadata);

        let block_time = env::block_timestamp();
        let deployment_info = DeploymentInfo {
            hash: blob_hash,
            version: release_info.version.clone(),
            deployment_time: block_time,
            upgrade_times: [(block_time, release_info.version)].into(),
            init_args: near_sdk::serde_json::to_string(&init_args).unwrap_or_default(),
        };

        Promise::new(new_contract_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(env::attached_deposit())
            .deploy_contract(code)
            .function_call(init_method, init_args_bytes, 0, NEW_GAS)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(ADD_DEPLOYMENT_GAS)
                    .update_deployment_info(new_contract_id, &deployment_info),
            )
    }

    /// Adds new deployment info of previously deployed contract.
    /// E.g. the contract which has been deployed by not this controller contract.
    #[access_control_any(roles(Role::DAO))]
    pub fn add_deployment_info(
        &mut self,
        contract_id: &AccountId,
        deployment_info: &DeploymentInfo,
    ) {
        self.deployments.insert(contract_id, deployment_info);
        event::emit(
            Event::AddDeploymentInfo,
            json!({"contract_id": contract_id, "deployment_info": deployment_info}),
        );
    }

    /// Callback which adds new deployment info after successful deployment of new contract.
    #[private]
    pub fn update_deployment_info(
        &mut self,
        contract_id: &AccountId,
        deployment_info: &DeploymentInfo,
    ) {
        let result = env::promise_result(0);

        if matches!(result, PromiseResult::Successful(_)) {
            self.deployments.insert(contract_id, deployment_info);
            event::emit(
                Event::UpdateDeploymentInfo,
                json!({"contract_id": contract_id, "deployment_info": deployment_info}),
            );
        }
    }

    /// Returns a list of existing contract deployments.
    #[must_use]
    pub fn get_deployments(&self) -> Vec<DeploymentInfo> {
        self.deployments.values_as_vector().to_vec()
    }

    /// Upgrades a contract with account id and provided or the latest hash.
    #[access_control_any(roles(Role::DAO, Role::Updater))]
    pub fn upgrade(&self, contract_id: AccountId, hash: Option<String>) -> Promise {
        self.upgrade_internal(contract_id, hash, false, Event::Upgrade)
    }

    /// Upgrades a contract with account id and provided hash without checking version.
    #[access_control_any(roles(Role::DAO))]
    pub fn unrestricted_upgrade(&self, contract_id: AccountId, hash: String) -> Promise {
        self.upgrade_internal(contract_id, Some(hash), true, Event::UnrestrictedUpgrade)
    }

    /// Downgrades the contract with account id.
    #[access_control_any(roles(Role::DAO))]
    pub fn downgrade(&self, contract_id: AccountId) -> Promise {
        let mut deployment_info = self.deployments.get(&contract_id).unwrap_or_else(|| {
            panic!("contract with account id: {contract_id} hasn't been deployed")
        });
        let release_info = self.releases.get(&deployment_info.hash).unwrap_or_else(|| {
            panic!(
                "release info doesn't exist for hash: {}",
                &deployment_info.hash
            )
        });
        let downgrade_hash = release_info
            .downgrade_hash
            .unwrap_or_else(|| panic!("release info doesn't include downgrade hash"));
        let downgrade_release_info = self
            .releases
            .get(&downgrade_hash)
            .unwrap_or_else(|| panic!("no release info for hash: {downgrade_hash}"));
        let event_metadata =
            json!({"contract_id": &contract_id, "release_info": &downgrade_release_info});
        let blob = self.blobs.get(&downgrade_hash).unwrap_or_else(|| {
            panic!(
                "blob doesn't exist for hash: {downgrade_hash} and version: {}",
                release_info.version
            )
        });

        event::emit(Event::Downgrade, event_metadata);
        deployment_info.update(downgrade_hash, downgrade_release_info.version);
        upgrade_promise(contract_id, blob, &deployment_info)
    }
}

impl AuroraControllerFactory {
    fn upgrade_internal(
        &self,
        contract_id: AccountId,
        hash: Option<String>,
        skip_version_check: bool,
        event: Event,
    ) -> Promise {
        let hash = hash
            .or_else(|| self.latest.get().map(|r| r.hash))
            .unwrap_or_else(|| panic!("no latest nor custom hash was provided for upgrading"));
        let release_info = self
            .releases
            .get(&hash)
            .unwrap_or_else(|| panic!("no release info for hash: {hash}"));

        let mut deployment_info = self.deployments.get(&contract_id).unwrap_or_else(|| {
            panic!("contract with account id: {contract_id} hasn't been deployed")
        });

        assert!(
            release_info.version > deployment_info.version || skip_version_check,
            "upgradable version: {} should be higher than the deployed version: {}",
            release_info.version,
            deployment_info.version
        );

        let event_metadata = json!({"contract_id": &contract_id, "release_info": &release_info});
        let blob = self.blobs.get(&hash).unwrap_or_else(|| {
            panic!(
                "blob doesn't exist for hash: {hash} and version: {}",
                release_info.version
            )
        });

        event::emit(event, event_metadata);
        deployment_info.update(hash, release_info.version);
        upgrade_promise(contract_id, blob, &deployment_info)
    }
}

#[ext_contract(ext_self)]
pub trait ExtAuroraControllerFactory {
    /// Callback which adds or overwrites deployment info after successful contract deployment,
    /// upgrading or downgrading.
    fn update_deployment_info(&mut self, contract_id: AccountId, deployment_info: &DeploymentInfo);
}

fn upgrade_promise(
    contract_id: AccountId,
    blob: Vec<u8>,
    deployment_info: &DeploymentInfo,
) -> Promise {
    Promise::new(contract_id.clone())
        .function_call("upgrade".to_string(), blob, 0, UPGRADE_GAS)
        .then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(ADD_DEPLOYMENT_GAS)
                .update_deployment_info(contract_id, deployment_info),
        )
}
