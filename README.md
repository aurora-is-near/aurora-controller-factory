# Aurora Controller Factory

[![CI](https://github.com/aurora-is-near/aurora-controller-factory/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/aurora-is-near/aurora-controller-factory/actions/workflows/rust.yml)

The main purpose of the contract is to provide the possibility to move all ops-related
functions into a separate controller contract. Ops-related functions include: deployment,
upgrading, downgrading, and delegation of execution for some transactions. The controller
contract implements role-based access control using the [near-plugins].

### Useful commands

- Build: `cargo make build`
- Clippy: `cargo make clippy`
- Test: `cargo make test`

The `cargo make build` creates two `wasm` files in the `res` folder:

- `aurora-controller-factory.wasm` - the main contract file.
- `aurora-controller-factory-borsh.wasm` - the borsh serialized contract file for upgrading the contract via
  `near-plugins`.

Note: the `up_stage_code` transaction from `near-plugins` accepts code of the contract serialized by `borsh`.

### API

#### Modified transactions

```rust
/// Initializes a new controller contract.
#[init]
fn new(dao: Option<AccountId>) -> Self;

/// Attaches new full access key to the controller contract.
#[access_control_any(roles(Role::DAO))]
fn attach_full_access_key(&mut self, public_key: PublicKey) -> Promise;

/// Delegates an execution of actions to the specified receiver.
#[access_control_any(roles(Role::DAO))]
fn delegate_execution(&self, receiver_id: AccountId, actions: Vec<FunctionCallArgs>) -> Promise;

/// Pauses the contract with provided account id.
#[access_control_any(roles(Role::DAO, Role::Pauser))]
fn delegate_pause(&self, receiver_id: AccountId, pause_method_name: Option<String>) -> Promise;

/// Adds new contract release info.
#[access_control_any(roles(Role::DAO))]
fn add_release_info(
    &mut self,
    hash: String,
    version: Version,
    is_latest: bool,
    downgrade_hash: Option<String>,
    description: Option<String>,
);

/// Adds bytes of the contract smart contract to the corresponding release info.
fn add_release_blob(&mut self);

/// Adds new deployment info of previously deployed contract by not this controller contract.
#[access_control_any(roles(Role::DAO))]
fn add_deployment_info(&mut self, contract_id: &AccountId, deployment_info: &DeploymentInfo);

/// Marks the release with the hash: `hash` as latest.
#[access_control_any(roles(Role::DAO, Role::Releaser))]
fn set_latest_release(&mut self, hash: &String);

/// Removes the release info for the provided hash.
#[access_control_any(roles(Role::DAO))]
fn remove_release(&mut self, hash: &String);

/// Deploys a new contract on the release info that corresponds to the provided hash or the latest.
#[access_control_any(roles(Role::DAO, Role::Deployer))]
fn deploy(
    &self,
    new_contract_id: AccountId,
    init_method: String,
    init_args: Value,
    blob_hash: Option<String>,
) -> Promise;

/// Upgrades a contract with account id and provided or the latest hash.
#[access_control_any(roles(Role::DAO, Role::Updater))]
fn upgrade(&self, contract_id: AccountId, hash: Option<String>) -> Promise;

/// Upgrades a contract with account id and provided hash without checking version.
#[access_control_any(roles(Role::DAO))]
fn unrestricted_upgrade(&self, contract_id: AccountId, hash: String) -> Promise;

/// Downgrades the contract with account id.
#[access_control_any(roles(Role::DAO))]
fn downgrade(&self, contract_id: AccountId) -> Promise;
```

#### View methods

```rust
/// Returns version of the controller contract.
fn version(&self) -> &str;

/// Returns a list of existing releases for deployment.
fn get_releases(&self) -> Vec<ReleaseInfo>;

/// Returns a WASM code from the release that corresponds the provided hash.
fn get_release_blob(&self, hash: &String) -> Vec<u8>;

/// Returns a hash of the latest release.
fn get_latest_release_hash(&self) -> String;

/// Returns a WASM code of the latest release.
fn get_latest_release_blob(&self) -> Vec<u8>;

/// Returns a list of existing contract deployments.
fn get_deployments(&self) -> BTreeMap<AccountId, DeploymentInfo>;

/// Returns a contract deployment info for corresponding account id.
fn get_deployment(&self, account_id: AccountId) -> Option<DeploymentInfo>;
```

#### Callback

```rust
/// Callback which adds or modifies a deployment info after successful deployment or upgrading of new contract.
#[private]
pub fn update_deployment_info(&mut self, contract_id: &AccountId, deployment_info: &DeploymentInfo);
```

#### Types used in transactions

```rust
/// Represents information about release.
#[derive(Debug, Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct ReleaseInfo {
    /// `sha256` hash of the WASM contract.
    pub hash: String,
    /// Version of the contract.
    pub version: Version,
    /// Flag which displays whether WASM data was added or not.
    pub is_blob_exist: bool,
    /// `sha256` hash of the WASM data for downgrading the contract.
    pub downgrade_hash: Option<String>,
    /// Description of the release.
    pub description: Option<String>,
}

/// Deployment information of the deployed contract.
#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct DeploymentInfo {
    /// `sha256` hash of the WASM contract.
    pub hash: String,
    /// Version of the contract.
    pub version: Version,
    /// Time of the contract deployment.
    pub deployment_time: u64,
    /// Upgrades history.
    pub upgrade_times: BTreeMap<u64, Version>,
    /// Initial arguments used while deploying the contact.
    pub init_args: String,
}
```

[near-plugins]: https://github.com/aurora-is-near/near-plugins

### LICENSE

**Aurora Controller Factory** is under [CC0 1.0 Universal](LICENSE)
