use crate::enumeration::string::VariantName;
use crate::enumeration::Enum;
use serde::{Deserialize, Serialize};

pub(crate) type I32Enum<T> = Enum<T, i32>;

// Define a wrapper struct around `I32Enum` allowing for serializing/deserializing from a known
// set of variants, and also arbitrary string values.
macro_rules! define_i32_enum {
    (
        $(#[$meta:meta])*
        $name:ident,
        $type:ty
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $name(crate::enumeration::I32Enum<$type>);

        impl $name {
            /// Creates a new value from a known variant.
            pub const fn new(variant: $type) -> Self {
                Self(crate::enumeration::I32Enum::Known(variant))
            }

            /// Creates a new value from a raw ID.
            pub fn new_from_id(id: i32) -> Self {
                use std::convert::TryFrom;

                match <$type>::try_from(id) {
                    Ok(value) => Self::new(value),
                    Err(_) => Self(crate::enumeration::I32Enum::Unknown(id)),
                }
            }

            /// Returns the underlying `i32` representation.
            pub fn as_id(&self) -> i32 {
                match self.0 {
                    crate::enumeration::I32Enum::Known(variant) => variant.into(),
                    crate::enumeration::I32Enum::Unknown(id) => id,
                }
            }

            /// Returns the string representation.
            pub fn as_str(&self) -> Option<&str> {
                self.0.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(name) = self.as_str() {
                    write!(f, "Error ID {} ({})", self.as_id(), name)
                } else {
                    write!(f, "Error ID {}", self.as_id())
                }
            }
        }

        impl From<i32> for $name {
            fn from(id: i32) -> Self {
                Self::new_from_id(id)
            }
        }

        impl From<$type> for $name {
            fn from(value: $type) -> Self {
                Self::new(value)
            }
        }

        impl PartialEq<$name> for $type {
            fn eq(&self, rhs: &$name) -> bool {
                rhs.0 == self
            }
        }

        impl PartialEq<&$name> for $type {
            fn eq(&self, rhs: &&$name) -> bool {
                (*rhs).0 == self
            }
        }

        impl PartialEq<$type> for $name {
            fn eq(&self, rhs: &$type) -> bool {
                self.0 == rhs
            }
        }

        impl PartialEq<i32> for $name {
            fn eq(&self, rhs: &i32) -> bool {
                self.0.as_i32() == *rhs
            }
        }

        impl PartialEq<&i32> for $name {
            fn eq(&self, rhs: &&i32) -> bool {
                self.0.as_i32() == **rhs
            }
        }
    };
}

pub(crate) use define_i32_enum;

impl<T> PartialEq for I32Enum<T>
where
    T: Into<i32> + Copy,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.as_i32() == rhs.as_i32()
    }
}

impl<T> PartialEq<T> for I32Enum<T>
where
    T: PartialEq + Copy + Into<i32>,
{
    fn eq(&self, rhs: &T) -> bool {
        use Enum::{Known, Unknown};

        match self {
            Known(value) => value == rhs,
            Unknown(value) => *value == (*rhs).into(),
        }
    }
}

impl<T> PartialEq<&T> for I32Enum<T>
where
    T: PartialEq + Into<i32> + Copy,
{
    fn eq(&self, rhs: &&T) -> bool {
        self == *rhs
    }
}

impl<T> I32Enum<T>
where
    T: Into<i32> + Copy,
{
    pub fn as_i32(&self) -> i32 {
        match self {
            Self::Known(value) => (*value).into(),
            Self::Unknown(value) => *value,
        }
    }
}

impl<T> I32Enum<T>
where
    T: Serialize,
{
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Known(value) => Some(VariantName::extract(value)),
            Self::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde_repr::{Deserialize_repr, Serialize_repr};

    type Result = std::result::Result<(), Box<dyn std::error::Error>>;

    #[derive(
        Clone,
        Copy,
        Debug,
        Deserialize_repr,
        PartialEq,
        Serialize_repr,
        num_enum::IntoPrimitive,
        num_enum::TryFromPrimitive,
    )]
    #[repr(i32)]
    pub enum ServerError {
        InternalServerError = 500,
        NotImplemented = 501,
        BadGateway = 502,
        ServiceUnavailable = 503,
        GatewayTimeout = 504,
        HttpVersionNotSupported = 505,
        VariantAlsoNegotiates = 506,
        InsufficientStorage = 507,
        LoopDetected = 508,
        NotExtended = 510,
        NetworkAuthenticationRequired = 511,
        NetworkConnectTimeoutError = 599,
    }

    define_i32_enum!(HttpError, ServerError);

    #[test]
    fn new() {
        assert_eq!(
            HttpError::new(ServerError::InternalServerError),
            HttpError::new_from_id(500),
        );
    }

    #[test]
    fn serde() -> Result {
        assert_eq!(
            serde_json::to_value(HttpError::new(ServerError::BadGateway))?,
            json!(502),
        );

        assert_eq!(
            serde_json::from_value::<HttpError>(json!(502))?,
            HttpError::new(ServerError::BadGateway),
        );

        Ok(())
    }
}
