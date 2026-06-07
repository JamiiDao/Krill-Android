use krill_common::{ActivityMetadata, MinMaxParticipants};
use tai64::Tai64N;

#[derive(Debug, Default, Clone, PartialEq, Eq, uniffi::Record)]
pub struct RustTypeActivityMetadata {
    pub activity_id: String,
    pub creator: String,
    pub name: String,
    pub timestamp: String,
    pub spend: String,
    pub threshold: RustTypeMinMax,
    pub domain_or_ip: String,
}

impl From<(i32, ActivityMetadata, String)> for RustTypeActivityMetadata {
    fn from(values: (i32, ActivityMetadata, String)) -> Self {
        let (offset, metadata, domain_or_ip) = values;

        let as_tai64n = metadata.timestamp.parse().unwrap_or(Tai64N::UNIX_EPOCH);
        let timestamp = parse_tai64n(as_tai64n, offset);

        let spend = (metadata.spend as f64 / 1_000_000_000f64).to_string();

        Self {
            activity_id: metadata.as_hex(),
            creator: metadata.creator,
            name: metadata.name,
            timestamp,
            spend,
            threshold: metadata.threshold.into(),
            domain_or_ip,
        }
    }
}

pub fn parse_tai64n(tai64n_time: Tai64N, offset: i32) -> String {
    let duration = tai64n_time
        .duration_since(&tai64::Tai64N::UNIX_EPOCH)
        .unwrap_or_default();
    let timestamp = hifitime::Epoch::from_unix_seconds(duration.as_secs_f64());
    let tz = hifitime::Duration::from_seconds(offset as f64);
    let fmt = hifitime::efmt::Formatter::with_timezone(
        timestamp,
        tz,
        hifitime::efmt::consts::RFC2822_LONG,
    );

    format!("{fmt}")
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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
