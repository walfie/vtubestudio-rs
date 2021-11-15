use crate::data::enumeration::EnumString;
use crate::data::{ApiError, Request, RequestType, Response, ResponseType};

use crate::error::{Error, UnexpectedResponseError};

use serde::de::IntoDeserializer;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::borrow::Cow;

/// The default `api_name` value in requests and responses.
pub const API_NAME: &'static str = "VTubeStudioPublicAPI";

/// The default `api_version` value in requests and responses.
pub const API_VERSION: &'static str = "1.0";

/// A VTube Studio API request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEnvelope {
    /// API name, typically `"VTubeStudioPublicAPI"`.
    pub api_name: Cow<'static, str>,
    /// API version, typically `"1.0"`.
    pub api_version: Cow<'static, str>,
    /// The original request ID.
    #[serde(rename = "requestID")]
    pub request_id: Option<String>,
    /// The request type.
    pub message_type: EnumString<RequestType>,
    /// The request data.
    pub data: Value,
}

impl Default for RequestEnvelope {
    fn default() -> Self {
        Self {
            api_name: Cow::Borrowed(API_NAME),
            api_version: Cow::Borrowed(API_VERSION),
            message_type: EnumString::new(RequestType::ApiStateRequest),
            request_id: None,
            data: Value::Null,
        }
    }
}

impl RequestEnvelope {
    /// Creates a request with the underlying typed data.
    pub fn new<Req: Request>(data: &Req) -> Result<Self, serde_json::Error> {
        let mut value = Self::default();
        value.set_data(data)?;
        Ok(value)
    }

    /// Sets the `data` field of a request.
    pub fn set_data<Req: Request>(&mut self, data: &Req) -> Result<(), serde_json::Error> {
        let data = serde_json::to_value(&data)?;
        self.message_type = Req::MESSAGE_TYPE.into();
        self.data = data;
        Ok(())
    }

    /// Sets the request ID.
    pub fn with_id<S: Into<Option<String>>>(mut self, id: S) -> Self {
        self.request_id = id.into();
        self
    }
}

/// A VTube Studio API response.
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseEnvelope {
    /// API name, typically `"VTubeStudioPublicAPI"`.
    pub api_name: Cow<'static, str>,
    /// API version, typically `"1.0"`.
    pub api_version: Cow<'static, str>,
    /// Unix timestamp (in milliseconds) of the response.
    pub timestamp: i64,
    /// The original request ID.
    pub request_id: String,
    /// Response data, which could be an [`ApiError`].
    pub data: Result<ResponseData, ApiError>,
}

const API_ERROR_MESSAGE_TYPE: &'static EnumString<ResponseType> =
    &EnumString::new(ResponseType::ApiError);

impl ResponseEnvelope {
    /// Creates a new response with the underlying typed data.
    pub fn new<Resp>(data: &Resp) -> Result<Self, serde_json::Error>
    where
        Resp: Response + Serialize,
    {
        let mut value = Self::default();
        value.set_data(data)?;
        Ok(value)
    }

    /// Sets the request ID.
    pub fn with_id(mut self, id: String) -> Self {
        self.request_id = id;
        self
    }

    /// Sets the `data` field of a response.
    pub fn set_data<Resp>(&mut self, data: &Resp) -> Result<(), serde_json::Error>
    where
        Resp: Response + Serialize,
    {
        let data = serde_json::to_value(data)?;
        self.data = Ok(ResponseData {
            message_type: Resp::MESSAGE_TYPE.into(),
            data,
        });
        Ok(())
    }

    /// The message type of this response.
    pub fn message_type(&self) -> &EnumString<ResponseType> {
        match &self.data {
            Ok(data) => &data.message_type,
            Err(_) => &API_ERROR_MESSAGE_TYPE,
        }
    }

    /// Attempts to parse the response into a the given [`Response`] type. This can return an error
    /// if the message type is an [`ApiError`] or isn't the expected type.
    pub fn parse<Resp: Response>(self) -> Result<Resp, Error> {
        let data = self.data?;

        if data.message_type == Resp::MESSAGE_TYPE {
            Ok(serde_json::from_value(data.data)?)
        } else {
            Err(UnexpectedResponseError {
                expected: Resp::MESSAGE_TYPE,
                received: data.message_type,
            }
            .into())
        }
    }

    /// Returns `true` if the message type is `APIError`.
    pub fn is_api_error(&self) -> bool {
        self.data.is_err()
    }

    /// Returns `true` if the message is an `APIError` with
    /// [`ErrorId::REQUEST_REQUIRES_AUTHENTICATION`](crate::data::ErrorId).
    pub fn is_unauthenticated_error(&self) -> bool {
        matches!(&self.data, Err(e) if e.is_unauthenticated())
    }
}

/// Response data wrapper for [`ResponseEnvelope`] (typically for non-error responses).
#[derive(Debug, Clone, PartialEq)]
pub struct ResponseData {
    /// The message type.
    pub message_type: EnumString<ResponseType>,
    /// The raw data.
    pub data: Value,
}

// Custom deserialize, to eagerly parse API errors.
impl<'de> Deserialize<'de> for ResponseEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawResponseEnvelope<'a> {
            #[serde(borrow)]
            pub api_name: Cow<'a, str>,
            #[serde(borrow)]
            pub api_version: Cow<'a, str>,
            pub timestamp: i64,
            #[serde(rename = "requestID")]
            pub request_id: String,
            pub message_type: EnumString<ResponseType>,
            pub data: Value,
        }

        let raw = RawResponseEnvelope::deserialize(deserializer)?;

        let data = if raw.message_type == ResponseType::ApiError {
            Err(ApiError::deserialize(raw.data.into_deserializer())
                .map_err(serde::de::Error::custom)?)
        } else {
            Ok(ResponseData {
                message_type: raw.message_type,
                data: raw.data,
            })
        };

        // Typically this will be "VTubeStudioPublicAPI, so we can possibly avoid allocating
        let api_name = if raw.api_name == API_NAME {
            Cow::Borrowed(API_NAME)
        } else {
            Cow::Owned(raw.api_name.into_owned())
        };

        // Typically this will be "1.0", so we can possibly avoid allocating
        let api_version = if raw.api_version == API_VERSION {
            Cow::Borrowed(API_VERSION)
        } else {
            Cow::Owned(raw.api_version.into_owned())
        };

        Ok(Self {
            api_name,
            api_version,
            timestamp: raw.timestamp,
            request_id: raw.request_id,
            data,
        })
    }
}

impl Serialize for ResponseEnvelope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RawResponseEnvelope<'a, T> {
            pub api_name: &'a str,
            pub api_version: &'a str,
            pub timestamp: i64,
            #[serde(rename = "requestID")]
            pub request_id: &'a str,
            pub message_type: &'a EnumString<ResponseType>,
            pub data: &'a T,
        }

        match &self.data {
            Ok(inner) => RawResponseEnvelope {
                api_name: &self.api_name,
                api_version: &self.api_version,
                timestamp: self.timestamp,
                request_id: &self.request_id,
                message_type: &inner.message_type,
                data: &inner.data,
            }
            .serialize(serializer),
            Err(e) => RawResponseEnvelope {
                api_name: &self.api_name,
                api_version: &self.api_version,
                timestamp: self.timestamp,
                request_id: &self.request_id,
                message_type: &ApiError::MESSAGE_TYPE,
                data: &e,
            }
            .serialize(serializer),
        }
    }
}

impl Default for ResponseEnvelope {
    fn default() -> Self {
        Self {
            api_name: API_NAME.into(),
            api_version: API_VERSION.into(),
            timestamp: 0,
            request_id: "".to_owned(),
            data: Ok(ResponseData {
                message_type: EnumString::const_new_from_str("UnknownResponse"),
                data: Value::Null,
            }),
        }
    }
}
