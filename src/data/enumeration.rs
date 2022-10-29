use crate::data::ResponseType;
use serde::ser::{Impossible, SerializeTupleVariant};
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Cow;

// Helper enum for allowing serde deserialization to retain unknown values, and serialize arbitrary
// unknown values for enums.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Enum<T, Repr> {
    Known(T),
    Unknown(Repr),
}

type EnumStringInner<T> = Enum<T, Cow<'static, str>>;

impl EnumString<ResponseType> {
    /// Whether this response type is an event.
    ///
    /// More specifically, it returns `true` if the underlying event type enum is a known
    /// [`Event`](crate::data::Event) type, or if the string ends with `"Event"`.
    pub fn is_event(&self) -> bool {
        match &self.0 {
            Enum::Known(t) => t.is_event(),
            Enum::Unknown(s) => s.ends_with("Event"),
        }
    }
}

/// Wrapper type for an `enum` with a serialized string representation.
///
/// This allows for defining an `enum` with a set of known values, but still accept other arbitrary
/// string values when serializing/deserializing.
///
/// # Example
///
/// ```
/// use vtubestudio::data::{EnumString, ResponseType};
///
/// // Multiple representations of the same enum
/// let resp_enum = EnumString::new(ResponseType::VtsFolderInfoResponse);
/// let resp_str = EnumString::new_from_str("VTSFolderInfoResponse");
///
/// // Can be compared to the inner enum type and other `EnumString`s
/// assert_eq!(resp_enum, ResponseType::VtsFolderInfoResponse);
/// assert_eq!(resp_str, ResponseType::VtsFolderInfoResponse);
/// assert_eq!(resp_enum, resp_str);
///
/// // Get the string representation of the enum
/// assert_eq!(resp_enum.as_str(), "VTSFolderInfoResponse");
/// assert_eq!(resp_str.as_str(), "VTSFolderInfoResponse");
/// ```
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumString<T>(pub(crate) EnumStringInner<T>);

impl<T> EnumString<T> {
    /// Creates a new value from a known variant.
    pub const fn new(variant: T) -> Self {
        Self(EnumStringInner::Known(variant))
    }

    /// Creates a new value from a raw string.
    pub fn new_from_str<S>(value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(EnumStringInner::new_from_str(value))
    }

    /// Creates a new value from a `const` static string slice.
    pub const fn const_new_from_str(value: &'static str) -> Self {
        Self(EnumStringInner::Unknown(std::borrow::Cow::Borrowed(value)))
    }
}

impl<T> From<T> for EnumString<T>
where
    T: Serialize + PartialEq,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> PartialEq for EnumString<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.0 == rhs.0
    }
}

impl<T> PartialEq<T> for EnumString<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &T) -> bool {
        self.0 == rhs
    }
}

impl<T> EnumString<T>
where
    T: Serialize,
{
    /// Returns the string representation.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T> std::fmt::Display for EnumString<T>
where
    T: Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<T, Repr> Default for Enum<T, Repr>
where
    T: Default,
{
    fn default() -> Self {
        Self::Known(T::default())
    }
}

impl<T> PartialEq for EnumStringInner<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &Self) -> bool {
        use Enum::{Known, Unknown};

        match (self, rhs) {
            (Known(a), Known(b)) => a == b,
            (Known(_), Unknown(b)) => self.as_str() == b,
            (Unknown(a), Known(_)) => a == rhs.as_str(),
            (Unknown(_), Unknown(_)) => self.as_str() == rhs.as_str(),
        }
    }
}

impl<T> PartialEq<str> for EnumStringInner<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &str) -> bool {
        self.as_str() == rhs
    }
}

impl<T> PartialEq<T> for EnumStringInner<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &T) -> bool {
        use Enum::{Known, Unknown};

        match self {
            Known(value) => value == rhs,
            Unknown(value) => value == VariantName::extract(rhs),
        }
    }
}

impl<T> PartialEq<&T> for EnumStringInner<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, rhs: &&T) -> bool {
        self == *rhs
    }
}

impl<T> EnumStringInner<T> {
    pub fn new_from_str<S>(value: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self::Unknown(value.into())
    }
}

impl<T> EnumStringInner<T>
where
    T: Serialize,
{
    pub fn as_str(&self) -> &str {
        match self {
            Self::Known(value) => VariantName::extract(value),
            Self::Unknown(value) => value.as_ref(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct VariantName;
impl VariantName {
    pub(crate) fn extract<T: Serialize>(value: &T) -> &'static str {
        value.serialize(&mut VariantName).unwrap_or("unknown")
    }
}

#[derive(thiserror::Error, Debug)]
#[error("cannot extract name of variant")]
pub(crate) struct VariantNameError;

impl serde::ser::Error for VariantNameError {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        VariantNameError
    }
}

pub(crate) struct TupleVariantName {
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

// Verbose serializer implementation that just extracts the enum variant name.
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
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    type Result = std::result::Result<(), Box<dyn std::error::Error>>;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub enum LazuLight {
        #[serde(rename = "DaPomky")]
        Pomu,
        Elira,
        Finana,
    }

    impl Default for LazuLight {
        fn default() -> Self {
            LazuLight::Pomu
        }
    }

    type Nijisanji = EnumString<LazuLight>;

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

        assert_ne!(Nijisanji::new(LazuLight::Pomu), LazuLight::Finana);

        // Equality between wrapper struct and raw string, renamed
        assert_eq!(Nijisanji::new(LazuLight::Pomu).as_str(), "DaPomky");
        assert_ne!(Nijisanji::new(LazuLight::Pomu).as_str(), "Pomu");

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

    #[test]
    fn is_event() -> Result {
        assert!(EnumString::new(ResponseType::TestEvent).is_event());
        assert!(EnumString::new_from_str("CoolNewEvent").is_event());

        assert_eq!(
            EnumString::new(ResponseType::VtsFolderInfoResponse).is_event(),
            false
        );
        assert_eq!(
            EnumString::new_from_str("ExampleResponse").is_event(),
            false
        );

        Ok(())
    }
}
