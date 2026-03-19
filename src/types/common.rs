use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// API sometimes returns numeric IDs as JSON strings (e.g. `"337048"`).
pub fn deserialize_i64_from_string_or_number<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| serde::de::Error::custom("expected i64-compatible JSON number")),
        serde_json::Value::String(s) => s.parse().map_err(serde::de::Error::custom),
        _ => Err(serde::de::Error::custom(
            "expected string or number for i64 field",
        )),
    }
}

/// Represents a hex-encoded integer value.
/// Deserializes from hex string (e.g. "0x1a2b") or integer.
/// Serializes to hex string.
///
/// Uses String internally to support 252-bit Stark field elements
/// which exceed the range of i128.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HexValue(String);

impl HexValue {
    /// Create from an i128 value (for small values like collateral IDs).
    pub fn new(value: i128) -> Self {
        Self(format!("{:#x}", value))
    }

    /// Create from a hex string (e.g. "0x1a2b").
    pub fn from_hex_string(hex: String) -> Self {
        Self(hex)
    }

    pub fn to_hex_string(&self) -> String {
        self.0.clone()
    }
}

impl From<i128> for HexValue {
    fn from(v: i128) -> Self {
        Self::new(v)
    }
}

impl From<i64> for HexValue {
    fn from(v: i64) -> Self {
        Self::new(v as i128)
    }
}

impl From<u64> for HexValue {
    fn from(v: u64) -> Self {
        Self::new(v as i128)
    }
}

impl Serialize for HexValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for HexValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s: serde_json::Value = Deserialize::deserialize(deserializer)?;
        match s {
            serde_json::Value::String(hex_str) => Ok(HexValue(hex_str)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(HexValue::new(i as i128))
                } else {
                    Err(serde::de::Error::custom("invalid number for HexValue"))
                }
            }
            _ => Err(serde::de::Error::custom(
                "expected hex string or integer for HexValue",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementSignatureModel {
    pub r: HexValue,
    pub s: HexValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    pub count: i64,
}
