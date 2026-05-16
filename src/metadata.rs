use serde::{Deserialize, Serialize};

/// Simplified `AVMetadataItem` view suitable for asset/player inspection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataItem {
    /// Fully-qualified metadata identifier when available.
    pub identifier: Option<String>,
    /// Metadata key-space raw value.
    pub key_space: Option<String>,
    /// Common-key raw value.
    pub common_key: Option<String>,
    /// String value when the metadata item is textual.
    pub string_value: Option<String>,
    /// Numeric value when the metadata item is numeric.
    pub number_value: Option<f64>,
    /// Underlying data type UTI.
    pub data_type: Option<String>,
    /// Best-effort human-readable description of the value.
    pub value_description: Option<String>,
}
