/// Simplified `AVMetadataItem` view suitable for asset/player inspection.
#[derive(Debug, Clone, PartialEq)]
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

use crate::error::AVPlayerError;

impl crate::binary::FromValue for MetadataItem {
    fn from_value(value: &crate::binary::Value) -> Result<Self, AVPlayerError> {
        Ok(Self {
            identifier: value.field(0)?,
            key_space: value.field(1)?,
            common_key: value.field(2)?,
            string_value: value.field(3)?,
            number_value: value.field(4)?,
            data_type: value.field(5)?,
            value_description: value.field(6)?,
        })
    }
}

impl crate::binary::ToValue for MetadataItem {
    fn to_value(&self) -> crate::binary::Value {
        crate::binary::Value::Array(vec![
            crate::binary::ToValue::to_value(&self.identifier),
            crate::binary::ToValue::to_value(&self.key_space),
            crate::binary::ToValue::to_value(&self.common_key),
            crate::binary::ToValue::to_value(&self.string_value),
            crate::binary::ToValue::to_value(&self.number_value),
            crate::binary::ToValue::to_value(&self.data_type),
            crate::binary::ToValue::to_value(&self.value_description),
        ])
    }
}
