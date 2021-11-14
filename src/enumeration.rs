use serde::ser::{Impossible, SerializeTupleVariant};
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;

macro_rules! define_string_enum {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $name(crate::enumeration::StringEnum<$type>);

        impl $name {
            /// Returns the string representation.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Creates a new value from a known variant.
            pub fn new(variant: $type) -> Self {
                Self(crate::enumeration::StringEnum::new(variant))
            }

            /// Creates a new value from a raw string.
            pub fn new_from_str<S>(value: S) -> Self
            where
                S: Into<Cow<'static, str>>,
            {
                Self(crate::enumeration::StringEnum::new_from_str(value))
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

        impl PartialEq<str> for $name {
            fn eq(&self, rhs: &str) -> bool {
                self.0.as_str() == rhs
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, rhs: &&str) -> bool {
                self.0.as_str() == *rhs
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringEnum<T> {
    Known(T),
    Unknown(Cow<'static, str>),
}

impl<T> PartialEq for StringEnum<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        use StringEnum::{Known, Unknown};

        match (self, rhs) {
            (Known(a), Known(b)) => a == b,
            (Known(_), Unknown(b)) => self.as_str() == b,
            (Unknown(a), Known(_)) => a == rhs.as_str(),
            (Unknown(_), Unknown(_)) => self.as_str() == rhs.as_str(),
        }
    }
}

impl<T> PartialEq<T> for StringEnum<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &T) -> bool {
        use StringEnum::{Known, Unknown};

        match self {
            Known(value) => value == rhs,
            Unknown(value) => value == VariantName::extract(rhs),
        }
    }
}

impl<T> PartialEq<&T> for StringEnum<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &&T) -> bool {
        self == *rhs
    }
}

impl<T> StringEnum<T>
where
    T: Serialize,
{
    pub fn new(variant: T) -> Self {
        Self::Known(variant)
    }

    pub fn new_from_str<S>(value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::Unknown(value.into())
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(value) => VariantName::extract(value),
            Self::Unknown(value) => value.as_ref(),
        }
    }
}

#[derive(Debug)]
struct VariantName;
impl VariantName {
    pub fn extract<T: Serialize>(value: &T) -> &'static str {
        value.serialize(&mut VariantName).unwrap_or("unknown")
    }
}

#[derive(thiserror::Error, Debug)]
#[error("cannot extract name of variant")]
struct VariantNameError;

impl serde::ser::Error for VariantNameError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        VariantNameError
    }
}

struct TupleVariantName {
    name: &'static str,
}

impl SerializeTupleVariant for TupleVariantName {
    type Ok = &'static str;
    type Error = VariantNameError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize + ?Sized,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.name)
    }
}

impl<'a> Serializer for &'a mut VariantName {
    type Ok = &'static str;
    type Error = VariantNameError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = TupleVariantName;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(name)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(name)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(variant)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(TupleVariantName { name: variant })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(VariantNameError)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(VariantNameError)
    }

    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized,
    {
        Err(VariantNameError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    type Result = std::result::Result<(), Box<dyn std::error::Error>>;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum LazuLight {
        #[serde(rename = "DaPomky")]
        Pomu,
        Elira,
        Finana,
    }

    define_string_enum!(Nijisanji, LazuLight);

    #[test]
    fn from() {
        assert_eq!(
            Nijisanji::new(LazuLight::Pomu),
            Nijisanji::from(LazuLight::Pomu)
        );
    }

    #[test]
    fn partial_eq() -> Result {
        // Wrapper struct equality
        assert_eq!(
            Nijisanji::new(LazuLight::Pomu),
            Nijisanji::new(LazuLight::Pomu),
        );

        assert_ne!(
            Nijisanji::new(LazuLight::Pomu),
            Nijisanji::new(LazuLight::Elira),
        );

        // Equality between wrapper struct and unwrapped struct
        assert_eq!(Nijisanji::new(LazuLight::Pomu), LazuLight::Pomu);

        assert_eq!(LazuLight::Pomu, Nijisanji::new(LazuLight::Pomu));

        assert_ne!(Nijisanji::new(LazuLight::Pomu), LazuLight::Finana);

        assert_ne!(LazuLight::Pomu, Nijisanji::new(LazuLight::Finana));

        // Equality between wrapper struct and raw string, renamed
        assert_eq!(Nijisanji::new(LazuLight::Pomu), "DaPomky");
        assert_ne!(Nijisanji::new(LazuLight::Pomu), "Pomu");

        // Equality between wrapper struct constructed different ways
        assert_eq!(
            Nijisanji::new_from_str("DaPomky"),
            Nijisanji::new(LazuLight::Pomu),
        );

        assert_eq!(
            Nijisanji::new(LazuLight::Pomu),
            Nijisanji::new_from_str("DaPomky"),
        );

        assert_eq!(
            Nijisanji::new_from_str("DaPomky"),
            Nijisanji::new_from_str("DaPomky"),
        );

        // Equality of `as_str`
        assert_eq!(Nijisanji::new(LazuLight::Elira).as_str(), "Elira");

        // Allow creation of custom values
        assert_eq!(
            Nijisanji::new_from_str("Petra"),
            Nijisanji::new_from_str("Petra"),
        );

        Ok(())
    }

    #[test]
    fn serialize() -> Result {
        assert_eq!(
            serde_json::to_value(Nijisanji::new(LazuLight::Pomu))?,
            json!("DaPomky"),
        );

        assert_eq!(
            serde_json::to_value(Nijisanji::new_from_str("DaPomky"))?,
            json!("DaPomky"),
        );

        assert_eq!(
            serde_json::to_value(Nijisanji::new_from_str("Oliver"))?,
            json!("Oliver"),
        );

        Ok(())
    }

    #[test]
    fn deserialize() -> Result {
        assert_eq!(
            serde_json::from_value::<Nijisanji>(json!("DaPomky"))?,
            Nijisanji::new(LazuLight::Pomu),
        );

        assert_eq!(
            serde_json::from_value::<Nijisanji>(json!("DaPomky"))?,
            Nijisanji::new_from_str("DaPomky"),
        );

        assert_eq!(
            serde_json::from_value::<Nijisanji>(json!("Oliver"))?,
            Nijisanji::new_from_str("Oliver"),
        );

        Ok(())
    }
}
