use bitcode::{Decode, Encode};
use krill_common::{ActivityInfo, ActivityMetadata, ActivityState, MinMaxParticipants};

use crate::RustFfiError;

#[derive(Debug, Default, Clone, PartialEq, Eq, uniffi::Record, Encode, Decode)]
pub struct RustTypeActivityMetadata {
    pub activity_id: String,
    pub creator: String,
    pub name: String,
    pub timestamp: String,
    pub spend: String,
    pub threshold: RustTypeMinMax,
    pub domain_or_ip: String,
}

impl TryFrom<(i32, ActivityMetadata, String)> for RustTypeActivityMetadata {
    type Error = RustFfiError;

    fn try_from(values: (i32, ActivityMetadata, String)) -> Result<Self, Self::Error> {
        let (offset, metadata, domain_or_ip) = values;

        let timestamp = metadata.timestamp.to_rfc_2822_long(offset)?;

        let spend = (metadata.spend as f64 / 1_000_000_000f64).to_string();

        Ok(Self {
            activity_id: metadata.as_hex(),
            creator: metadata.creator,
            name: metadata.name,
            timestamp,
            spend,
            threshold: metadata.threshold.into(),
            domain_or_ip,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record, Encode, Decode)]
pub struct RustTypeMinMax {
    pub min: u16,
    pub max: u16,
}

impl Default for RustTypeMinMax {
    fn default() -> Self {
        Self { min: 2, max: 2 }
    }
}

impl From<MinMaxParticipants> for RustTypeMinMax {
    fn from(value: MinMaxParticipants) -> Self {
        Self {
            min: value.min,
            max: value.max,
        }
    }
}

impl From<RustTypeMinMax> for MinMaxParticipants {
    fn from(value: RustTypeMinMax) -> Self {
        Self {
            min: value.min,
            max: value.max,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, uniffi::Record, Encode, Decode)]
pub struct RustTypeActivityInfo {
    pub activity_state: RustTypeActivityState,
    pub activity_id: String,
    pub creator: String,
    pub name: String,
    pub timestamp: String,
    pub spend: String,
    pub threshold: RustTypeMinMax,
}

impl TryFrom<(i32, ActivityInfo)> for RustTypeActivityInfo {
    type Error = RustFfiError;

    fn try_from(values: (i32, ActivityInfo)) -> Result<Self, Self::Error> {
        let (zone, info) = values;

        Ok(Self {
            activity_state: info.metadata.state.clone().into(),
            activity_id: info.metadata.as_hex(),
            creator: info.metadata.creator,
            name: info.metadata.name,
            timestamp: info.metadata.timestamp.to_rfc_2822_long(zone)?,
            spend: (info.metadata.spend as f64 / 1_000_000_000f64).to_string(),
            threshold: info.metadata.threshold.into(),
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, uniffi::Enum, Encode, Decode)]
pub enum RustTypeActivityState {
    #[default]
    DkgRound1,
    DkgRound2,
    DkgFinalized,
    SignalSigning,
    SigningRound1,
    SigningRound2,
    SigningAggregate,
}

impl RustTypeActivityState {
    pub fn stringify(&self) -> &'static str {
        match self {
            Self::DkgRound1 => "Round 1 Key Sharing",
            Self::DkgRound2 => "Round 2 Key Sharing",
            Self::DkgFinalized => "Key Sharing finalized",
            Self::SignalSigning => "Relay Signaled Signing Event",
            Self::SigningRound1 => "Threshold Signing Round 1",
            Self::SigningRound2 => "Threshold Signing Round 2",
            Self::SigningAggregate => "Threshold Signing Finalized",
        }
    }
}

impl From<ActivityState> for RustTypeActivityState {
    fn from(state: ActivityState) -> Self {
        match state {
            ActivityState::DkgRound1 => Self::DkgRound1,
            ActivityState::DkgRound2 => Self::DkgRound2,
            ActivityState::DkgFinalized => Self::DkgFinalized,
            ActivityState::SignalSigning => Self::SignalSigning,
            ActivityState::SigningRound1 => Self::SigningRound1,
            ActivityState::SigningRound2 => Self::SigningRound2,
            ActivityState::SigningAggregate => Self::SigningAggregate,
        }
    }
}

impl From<RustTypeActivityState> for ActivityState {
    fn from(state: RustTypeActivityState) -> Self {
        match state {
            RustTypeActivityState::DkgRound1 => Self::DkgRound1,
            RustTypeActivityState::DkgRound2 => Self::DkgRound2,
            RustTypeActivityState::DkgFinalized => Self::DkgFinalized,
            RustTypeActivityState::SignalSigning => Self::SignalSigning,
            RustTypeActivityState::SigningRound1 => Self::SigningRound1,
            RustTypeActivityState::SigningRound2 => Self::SigningRound2,
            RustTypeActivityState::SigningAggregate => Self::SigningAggregate,
        }
    }
}
