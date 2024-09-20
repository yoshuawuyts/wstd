use super::Error;
use std::{borrow::Cow, collections::HashMap, ops::Deref};
use wasi::http::types::{ErrorCode, Fields as WasiFields, HeaderError};

/// A type alias for [`Fields`] when used as HTTP headers.
pub type Headers = Fields;

/// A type alias for [`Fields`] when used as HTTP trailers.
pub type Trailers = Fields;

/// An HTTP Field name.
pub type FieldName = Cow<'static, str>;

/// An HTTP Field value.
pub type FieldValue = Vec<u8>;

/// HTTP Fields which can be used as either trailers or headers.
#[derive(Clone, PartialEq, Eq)]
pub struct Fields(pub(crate) HashMap<FieldName, Vec<FieldValue>>);

impl Fields {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn contains(&self, k: &FieldName) -> bool {
        self.0.get(k).is_some_and(|v| !v.is_empty())
    }
    pub fn get(&self, k: &FieldName) -> Option<&[FieldValue]> {
        self.0.get(k).map(|f| f.deref())
    }
    pub fn get_mut(&mut self, k: &FieldName) -> Option<&mut Vec<FieldValue>> {
        self.0.get_mut(k)
    }
    pub fn insert(&mut self, k: FieldName, v: Vec<FieldValue>) {
        self.0.insert(k, v);
    }
    pub fn append(&mut self, k: FieldName, v: FieldValue) {
        match self.0.get_mut(&k) {
            Some(vals) => vals.push(v),
            None => {
                self.0.insert(k, vec![v]);
            }
        }
    }
    pub fn remove(&mut self, k: &FieldName) -> Option<Vec<FieldValue>> {
        self.0.remove(k)
    }
}

impl std::fmt::Debug for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by_cached_key(|(k, _)| k.to_owned());
        for (key, values) in entries {
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
            let field_name = key.into();
            let field_list: &mut Vec<_> = output.entry(field_name).or_default();
            field_list.push(value);
        }
        Self(output)
    }
}

impl TryFrom<Fields> for WasiFields {
    type Error = Error;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let mut list = Vec::with_capacity(fields.0.capacity());
        for (name, values) in fields.0.into_iter() {
            for value in values {
                list.push((name.clone().into_owned(), value));
            }
        }
        Ok(WasiFields::from_list(&list).map_err(|e| {
            let msg = match e {
                HeaderError::InvalidSyntax => "header has invalid syntax",
                HeaderError::Forbidden => "header key is forbidden",
                HeaderError::Immutable => "headers are immutable",
            };
            ErrorCode::InternalError(Some(msg.to_string()))
        })?)
    }
}
