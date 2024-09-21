use super::Error;
use std::{collections::HashMap, ops::Deref};
use wasi::http::types::{ErrorCode, Fields as WasiFields, HeaderError};

/// A type alias for [`Fields`] when used as HTTP headers.
pub type Headers = Fields;

/// A type alias for [`Fields`] when used as HTTP trailers.
pub type Trailers = Fields;

/// An HTTP Field name.
pub type FieldName = String;

/// An HTTP Field value.
pub type FieldValue = Vec<u8>;

/// Field entry.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct FieldEntry {
    /// Field key in original case.
    key: String,
    /// Field values.
    values: Vec<FieldValue>,
}

/// HTTP Fields which can be used as either trailers or headers.
#[derive(Clone, PartialEq, Eq)]
pub struct Fields(pub(crate) HashMap<FieldName, FieldEntry>);

impl Fields {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn contains(&self, key: &str) -> bool {
        self.0
            .get(key)
            .is_some_and(|entry| !entry.values.is_empty())
    }
    pub fn get(&self, key: &str) -> Option<&[FieldValue]> {
        if key.chars().any(|c| c.is_uppercase()) {
            self.0
                .get(&key.to_lowercase())
                .map(|entry| entry.values.deref())
        } else {
            self.0.get(key).map(|entry| entry.values.deref())
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Vec<FieldValue>> {
        if key.chars().any(|c| c.is_uppercase()) {
            self.0
                .get_mut(&key.to_lowercase())
                .map(|entry| entry.values.as_mut())
        } else {
            self.0.get_mut(key).map(|entry| entry.values.as_mut())
        }
    }
    pub fn insert(&mut self, key: String, values: Vec<FieldValue>) {
        self.0
            .insert(key.to_lowercase(), FieldEntry { key, values });
    }
    pub fn append(&mut self, key: String, value: FieldValue) {
        let entry: &mut FieldEntry = self.0.entry(key.to_lowercase()).or_insert(FieldEntry {
            key,
            values: Vec::with_capacity(1),
        });
        entry.values.push(value);
    }
    pub fn remove(&mut self, key: &str) -> Option<Vec<FieldValue>> {
        self.0.remove(key).map(|entry| entry.values)
    }
}

impl std::fmt::Debug for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        let mut entries: Vec<_> = self.0.values().collect();
        entries.sort_by_cached_key(|entry| entry.key.to_owned());
        for FieldEntry { key, values } in entries {
            match values.len() {
                0 => {
                    map.entry(key, &"");
                }
                1 => {
                    let value = values.iter().next().unwrap();
                    let value = String::from_utf8_lossy(value);
                    map.entry(key, &value);
                }
                _ => {
                    let values: Vec<_> =
                        values.iter().map(|v| String::from_utf8_lossy(v)).collect();
                    map.entry(key, &values);
                }
            }
        }
        map.finish()
    }
}

impl From<WasiFields> for Fields {
    fn from(wasi_fields: WasiFields) -> Self {
        let mut output = HashMap::new();
        for (key, value) in wasi_fields.entries() {
            let field_name = key.to_lowercase();
            let entry: &mut FieldEntry = output.entry(field_name).or_insert(FieldEntry {
                key,
                values: Vec::with_capacity(1),
            });
            entry.values.push(value);
        }
        Self(output)
    }
}

impl TryFrom<Fields> for WasiFields {
    type Error = Error;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let mut list = Vec::with_capacity(fields.0.values().map(|entry| entry.values.len()).sum());
        for FieldEntry { key, values } in fields.0.into_values() {
            for value in values {
                list.push((key.clone(), value));
            }
        }
        WasiFields::from_list(&list).map_err(|e| {
            let msg = match e {
                HeaderError::InvalidSyntax => "header has invalid syntax",
                HeaderError::Forbidden => "header key is forbidden",
                HeaderError::Immutable => "headers are immutable",
            };
            ErrorCode::InternalError(Some(msg.to_string()))
        })
    }
}
