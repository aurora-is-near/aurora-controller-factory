use near_sdk::serde::Serialize;
use std::fmt::{Display, Formatter};

const EVENT_PRODUCER: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde", rename_all = "snake_case")]
pub enum Event {
    AddBlob,
    AddReleaseInfo,
    AddDeploymentInfo,
    UpdateDeploymentInfo,
    AttachFullAccessKey,
    Deploy,
    DelegatedPause,
    DelegatedExecution,
    SetLatestReleaseInfo,
    RemoveReleaseInfo,
    Upgrade,
    UnrestrictedUpgrade,
    Downgrade,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
struct EventMetadata<'a, T> {
    producer: &'static str,
    version: &'static str,
    event: Event,
    metadata: Option<&'a T>,
}

impl<'a, T> EventMetadata<'a, T>
where
    T: Serialize,
{
    /// Create new event metadata
    const fn new(event: Event, metadata: &'a T) -> Self {
        Self {
            producer: EVENT_PRODUCER,
            version: VERSION,
            event,
            metadata: Some(metadata),
        }
    }

    /// Emit the log with event metadata on chain.
    fn emit(&self) {
        near_sdk::log!("EVENT_JSON:{}", self);
    }
}

impl<T> Display for EventMetadata<'_, T>
where
    T: Serialize,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            near_sdk::serde_json::to_string(self).unwrap_or_default()
        )
    }
}

/// Emit the log with event metadata on chain.
pub fn emit<T: Serialize>(event: Event, metadata: &T) {
    EventMetadata::new(event, metadata).emit();
}

#[test]
fn test_stringify_event_metadata() {
    use crate::types::ReleaseInfo;

    let release_info = ReleaseInfo {
        hash: "9316bf4c7aa0913f26ef8eebdcb11f3c63bb88c65eb717abfec8ade1b707620c".to_string(),
        version: "3.5.0".parse().unwrap(),
        is_blob_exist: false,
        downgrade_hash: None,
        description: Some("Aurora SILO 3.5.0".to_string()),
    };
    let event_metadata = EventMetadata::new(Event::AddReleaseInfo, &release_info);

    assert_eq!(
        event_metadata.to_string(),
        r#"{"producer":"aurora-controller-factory","version":"0.2.1","event":"add_release_info","metadata":{"hash":"9316bf4c7aa0913f26ef8eebdcb11f3c63bb88c65eb717abfec8ade1b707620c","version":"3.5.0","is_blob_exist":false,"downgrade_hash":null,"description":"Aurora SILO 3.5.0"}}"#
    );
}
