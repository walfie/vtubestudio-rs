mod number;
mod string;

#[allow(dead_code, unused_imports, unused_macros)] // TODO
pub(crate) use self::number::{define_i32_enum, I32Enum};
pub(crate) use self::string::{define_string_enum, StringEnum};

use serde::{Deserialize, Serialize};

// Helper enum for allowing serde deserialization to retain unknown values, and serialize arbitrary
// string values for enums. This is meant to be used inside the `define_string_enum` macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Enum<T, Repr> {
    Known(T),
    Unknown(Repr),
}
