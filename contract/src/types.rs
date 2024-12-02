use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near, Gas, NearToken};
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::str::FromStr;

/// Information about release.
#[derive(Debug, Default, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[near(serializers = [json, borsh])]
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
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[near(serializers = [json, borsh])]
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

impl DeploymentInfo {
    pub fn update(&mut self, hash: String, version: Version) {
        self.hash = hash;
        self.version = version.clone();
        self.upgrade_times
            .insert(near_sdk::env::block_timestamp(), version);
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
#[near(serializers = [json])]
pub struct Version(semver::Version);

impl FromStr for Version {
    type Err = semver::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl BorshDeserialize for Version {
    fn deserialize_reader<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let string = String::deserialize_reader(reader)?;
        string
            .parse()
            .map(Self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
    }
}

impl BorshSerialize for Version {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&near_sdk::borsh::to_vec(&self.0.to_string())?)
    }
}

impl Default for Version {
    fn default() -> Self {
        "0.0.1".parse().unwrap()
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_string())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FunctionCallArgs {
    pub function_name: String,
    pub arguments: Base64VecU8,
    pub amount: NearToken,
    pub gas: Gas,
}

#[derive(Debug)]
#[near(serializers = [borsh])]
pub struct UpgradeArgs {
    pub code: Vec<u8>,
    pub state_migration_gas: Option<u64>,
}

#[test]
fn test_version_borsh_serialize() {
    let actual: Version = "1.2.3-rc.2".parse().unwrap();
    let bytes = near_sdk::borsh::to_vec(&actual).unwrap();
    let expected = Version::try_from_slice(&bytes).unwrap();

    assert_eq!(actual, expected);
}
