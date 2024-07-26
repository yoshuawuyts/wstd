use std::{borrow::Cow, collections::HashMap};
use wasi::http::types::{Fields as WasiFields, HeaderError};

/// A type alias for [`Fields`] when used as HTTP headers.
pub type Headers = Fields;

/// A type alias for [`Fields`] when used as HTTP trailers.
pub type Trailers = Fields;

/// An HTTP Field name.
pub type FieldName = Cow<'static, str>;

/// An HTTP Field value.
pub type FieldValue = Vec<u8>;

/// HTTP Fields which can be used as either trailers or headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fields(pub(crate) HashMap<FieldName, Vec<FieldValue>>);

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
    type Error = HeaderError;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let mut list = Vec::with_capacity(fields.0.capacity());
        for (name, values) in fields.0.into_iter() {
            for value in values {
                list.push((name.clone().into_owned(), value));
            }
        }
        Ok(WasiFields::from_list(&list)?)
    }
}
