use near_plugins::{
    access_control, access_control_any, AccessControlRole, AccessControllable, Pausable, Upgradable,
};
use near_sdk::borsh::BorshDeserialize;
use near_sdk::collections::LazyOption;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, Value};
use near_sdk::store::IterableMap;
use near_sdk::{
    env, ext_contract, near, require, AccountId, Gas, NearToken, PanicOnDefault, Promise,
    PromiseResult, PublicKey,
};

use crate::event::Event;
use crate::types::{DeploymentInfo, FunctionCallArgs, ReleaseInfo, UpgradeArgs, Version};

mod event;
mod keys;
#[cfg(test)]
mod tests;
pub mod types;
pub mod utils;

/// Gas needed for initialization deployed contract.
const NEW_GAS: Gas = Gas::from_tgas(100);

/// Gas needed to upgrade contract (except a gas for the migration state).
const UPGRADE_GAS: Gas = Gas::from_tgas(130);

/// Gas needed to upgrade contract (except a gas for the migration state).
const UPGRADE_GAS_NO_MIGRATION_GAS: Gas = Gas::from_tgas(180);

/// Gas needed to call the `add_deployment` callback.
const ADD_DEPLOYMENT_GAS: Gas = Gas::from_tgas(5);

/// Amount of gas used by `delegate_pause` in the controller contract
/// without taking into account the gas consumed by the promise.
const OUTER_DELEGATE_PAUSE_GAS: Gas = Gas::from_tgas(10);

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

/// Controller contract for deploying and upgrading contracts.
#[derive(PanicOnDefault, Pausable, Upgradable)]
#[access_control(role_type(Role))]
#[upgradable(access_control_roles(
    code_stagers(Role::DAO),
    code_deployers(Role::DAO),
    duration_initializers(Role::DAO),
    duration_update_stagers(Role::DAO),
    duration_update_appliers(Role::DAO),
))]
#[pausable(manager_roles(Role::DAO, Role::Pauser))]
#[near(contract_state)]
pub struct AuroraControllerFactory {
    releases: IterableMap<String, ReleaseInfo>,
    blobs: IterableMap<String, Vec<u8>>,
    deployments: IterableMap<AccountId, DeploymentInfo>,
    latest: LazyOption<ReleaseInfo>,
}

#[near]
impl AuroraControllerFactory {
    /// Initializes a new controller contract.
    ///
    /// # Panics
    ///
    /// The function panics if the state of the contract is already exist.
    #[must_use]
    #[init]
    #[allow(clippy::use_self)]
    pub fn new(dao: Option<AccountId>) -> Self {
        let mut contract = Self {
            releases: IterableMap::new(keys::Prefix::Releases),
            blobs: IterableMap::new(keys::Prefix::Blobs),
            deployments: IterableMap::new(keys::Prefix::Deployments),
            latest: LazyOption::new(keys::Prefix::LatestRelease, None),
        };

        require!(
            contract.acl_init_super_admin(env::current_account_id()),
            "Failed to init Super Admin role"
        );

        // Optionally grant `Role::DAO`.
        if let Some(account_id) = dao {
            let res = contract.acl_grant_role(Role::DAO.into(), account_id);
            require!(Some(true) == res, "Failed to grant DAO role");
        }

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
            &json!({"public_key": &public_key}),
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
            &json!({
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
                    action.amount,
                    action.gas,
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
        let gas = env::prepaid_gas().saturating_sub(OUTER_DELEGATE_PAUSE_GAS);

        event::emit(
            Event::DelegatedPause,
            &json!({
                "receiver_id": &receiver_id,
                "pause_method_name": &function_name
            }),
        );

        Promise::new(receiver_id).function_call(function_name, vec![], NearToken::from_near(0), gas)
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
        require!(
            self.releases.get(&hash).is_none(),
            "release info for the hash is already exist"
        );

        let release_info = ReleaseInfo {
            hash: hash.clone(),
            version,
            is_blob_exist: false,
            downgrade_hash,
            description,
        };

        event::emit(Event::AddReleaseInfo, &release_info);
        self.releases.insert(hash.clone(), release_info);

        if is_latest {
            self.set_latest_release(&hash);
        }
    }

    /// Adds bytes of the contract smart contract to the corresponding release info.
    pub fn add_release_blob(&mut self) {
        let blob = env::input().unwrap_or_else(|| panic!("no blob's bytes were provided"));
        let hash = utils::hash_256(&blob);
        let release_info = self.releases.get_mut(&hash).unwrap_or_else(|| {
            panic!("release info doesn't exist for the hash: {hash}");
        });

        event::emit(Event::AddBlob, &json!({"blob_hash": &hash}));

        release_info.is_blob_exist = true;
        self.blobs.insert(hash, blob);
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

        self.latest.set(new_latest);
        event::emit(Event::SetLatestReleaseInfo, new_latest);
    }

    /// Removes the release info for hash: `hash`.
    #[access_control_any(roles(Role::DAO))]
    pub fn remove_release(&mut self, hash: &String) {
        let release_info = self.releases.remove(hash).unwrap_or_else(|| {
            panic!("release info doesn't exist for hash: {hash}");
        });
        self.blobs.remove(hash);
        event::emit(Event::RemoveReleaseInfo, &release_info);
    }

    /// Returns a list of existing releases for deployment.
    #[must_use]
    pub fn get_releases(&self) -> Vec<ReleaseInfo> {
        self.releases.values().cloned().collect()
    }

    /// Returns a WASM code from the release that corresponds the provided hash.
    #[must_use]
    pub fn get_release_blob(&self, hash: &String) -> Vec<u8> {
        self.blobs
            .get(hash)
            .cloned()
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
        require!(
            self.deployments.get(&new_contract_id).is_none(),
            format!("{new_contract_id} is already deployed")
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

        event::emit(Event::Deploy, &event_metadata);

        let block_time = env::block_timestamp();
        let deployment_info = DeploymentInfo {
            hash: blob_hash,
            version: release_info.version.clone(),
            deployment_time: block_time,
            upgrade_times: [(block_time, release_info.version.clone())].into(),
            init_args: near_sdk::serde_json::to_string(&init_args).unwrap_or_default(),
        };

        Promise::new(new_contract_id.clone())
            .create_account()
            .add_full_access_key(env::signer_account_pk())
            .transfer(env::attached_deposit())
            .deploy_contract(code.clone())
            .function_call(
                init_method,
                init_args_bytes,
                NearToken::from_near(0),
                NEW_GAS,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(ADD_DEPLOYMENT_GAS)
                    .update_deployment_info(new_contract_id, deployment_info),
            )
    }

    /// Adds new deployment info of previously deployed contract.
    /// E.g. the contract which has been deployed by not this controller contract.
    #[access_control_any(roles(Role::DAO))]
    pub fn add_deployment_info(&mut self, contract_id: AccountId, deployment_info: DeploymentInfo) {
        event::emit(
            Event::AddDeploymentInfo,
            &json!({"contract_id": contract_id, "deployment_info": deployment_info}),
        );
        self.deployments.insert(contract_id, deployment_info);
    }

    /// Callback which adds new deployment info after successful deployment of new contract.
    #[private]
    pub fn update_deployment_info(
        &mut self,
        contract_id: AccountId,
        deployment_info: DeploymentInfo,
    ) {
        let result = env::promise_result(0);

        if matches!(result, PromiseResult::Successful(_)) {
            event::emit(
                Event::UpdateDeploymentInfo,
                &json!({"contract_id": contract_id, "deployment_info": deployment_info}),
            );
            self.deployments.insert(contract_id, deployment_info);
        }
    }

    /// Returns a list of existing contract deployments.
    #[must_use]
    pub fn get_deployments(&self) -> Vec<DeploymentInfo> {
        self.deployments.values().cloned().collect()
    }

    /// Upgrades a contract with account id and provided or the latest hash.
    #[access_control_any(roles(Role::DAO, Role::Updater))]
    pub fn upgrade(
        &mut self,
        contract_id: AccountId,
        hash: Option<String>,
        state_migration_gas: Option<u64>,
    ) -> Promise {
        self.upgrade_internal(
            contract_id,
            hash,
            false,
            state_migration_gas,
            Event::Upgrade,
        )
    }

    /// Upgrades a contract with account id and provided hash without checking version.
    #[access_control_any(roles(Role::DAO))]
    pub fn unrestricted_upgrade(
        &mut self,
        contract_id: AccountId,
        hash: String,
        state_migration_gas: Option<u64>,
    ) -> Promise {
        self.upgrade_internal(
            contract_id,
            Some(hash),
            true,
            state_migration_gas,
            Event::UnrestrictedUpgrade,
        )
    }

    /// Downgrades the contract with account id.
    #[access_control_any(roles(Role::DAO))]
    pub fn downgrade(&mut self, contract_id: AccountId) -> Promise {
        let mut deployment_info =
            self.deployments
                .get(&contract_id)
                .cloned()
                .unwrap_or_else(|| {
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
            .clone()
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

        event::emit(Event::Downgrade, &event_metadata);
        deployment_info.update(downgrade_hash, downgrade_release_info.version.clone());

        let args = UpgradeArgs {
            code: blob.clone(),
            state_migration_gas: None,
        };

        Self::upgrade_promise(contract_id, args, deployment_info)
    }
}

impl AuroraControllerFactory {
    fn upgrade_internal(
        &self,
        contract_id: AccountId,
        hash: Option<String>,
        skip_version_check: bool,
        state_migration_gas: Option<u64>,
        event: Event,
    ) -> Promise {
        let hash = hash
            .or_else(|| self.latest.get().map(|r| r.hash))
            .unwrap_or_else(|| panic!("no latest nor custom hash was provided for upgrading"));
        let release_info = self
            .releases
            .get(&hash)
            .unwrap_or_else(|| panic!("no release info for hash: {hash}"));

        let mut deployment_info =
            self.deployments
                .get(&contract_id)
                .cloned()
                .unwrap_or_else(|| {
                    panic!("contract with account id: {contract_id} hasn't been deployed")
                });

        require!(
            release_info.version > deployment_info.version || skip_version_check,
            format!(
                "upgradable version: {} should be higher than the deployed version: {}",
                release_info.version, deployment_info.version
            )
        );

        let event_metadata = json!({"contract_id": &contract_id, "release_info": &release_info});
        let blob = self.blobs.get(&hash).unwrap_or_else(|| {
            panic!(
                "blob doesn't exist for hash: {hash} and version: {}",
                release_info.version
            )
        });

        event::emit(event, &event_metadata);
        deployment_info.update(hash, release_info.version.clone());

        let args = UpgradeArgs {
            code: blob.clone(),
            state_migration_gas,
        };

        Self::upgrade_promise(contract_id, args, deployment_info)
    }

    fn upgrade_promise(
        contract_id: AccountId,
        args: UpgradeArgs,
        deployment_info: DeploymentInfo,
    ) -> Promise {
        ext_aurora::ext(contract_id.clone())
            .with_static_gas(
                args.state_migration_gas
                    .map_or(UPGRADE_GAS_NO_MIGRATION_GAS, |gas| {
                        UPGRADE_GAS.saturating_add(Gas::from_gas(gas))
                    }),
            )
            .upgrade(args.code, args.state_migration_gas)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(ADD_DEPLOYMENT_GAS)
                    .update_deployment_info(contract_id, deployment_info),
            )
    }
}

#[ext_contract(ext_aurora)]
pub trait ExtAurora {
    fn upgrade(
        &mut self,
        #[serializer(borsh)] code: Vec<u8>,
        #[serializer(borsh)] state_migration_gas: Option<u64>,
    );
}
