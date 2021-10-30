use paste::paste;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub api_name: Cow<'static, str>,
    pub api_version: Cow<'static, str>,
    #[serde(rename = "requestID")]
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub data: RequestData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub api_name: String,
    pub api_version: String,
    pub timestamp: i64,
    #[serde(rename = "requestID")]
    pub request_id: String,
    #[serde(flatten)]
    pub data: ResponseData,
}

pub trait ApiRequest {
    type Response: ApiResponse;
}

pub trait ApiResponse {}

macro_rules! define_request_response_pairs {
    ($({
        rust_name = $rust_name:ident,
        $(req_name = $req_name:literal,)?
        $(resp_name = $resp_name:literal,)?
        req = { $($req:tt)* },
        resp = { $($resp:tt)* },
    },)*) => {
        $(
            paste! {
                #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Request>] { $($req)* }

                impl From<[<$rust_name Request>]> for RequestData {
                    fn from(value: [<$rust_name Request>]) -> Self {
                        RequestData::[<$rust_name Request>](value)
                    }
                }

                impl ApiRequest for [<$rust_name Request>] {
                    type Response = [<$rust_name Response>];
                }

                impl std::convert::TryFrom<RequestData> for [<$rust_name Request>] {
                    type Error = RequestData;

                    fn try_from(value: RequestData) -> Result<Self, Self::Error> {
                        if let RequestData::[<$rust_name Request>](inner) = value {
                            Ok(inner)
                        } else {
                            Err(value)
                        }
                    }
                }

                #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Response>] { $($resp)* }

                impl ApiResponse for [<$rust_name Response>] {}

                impl From<[<$rust_name Response>]> for ResponseData {
                    fn from(value: [<$rust_name Response>]) -> Self {
                        ResponseData::[<$rust_name Response>](value)
                    }
                }

                impl std::convert::TryFrom<ResponseData> for [<$rust_name Response>] {
                    type Error = ResponseData;

                    fn try_from(value: ResponseData) -> Result<Self, Self::Error> {
                        if let ResponseData::[<$rust_name Response>](inner) = value {
                            Ok(inner)
                        } else {
                            Err(value)
                        }
                    }
                }

            }
        )*

        paste! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            #[serde(tag = "messageType", content = "data")]
            pub enum RequestData {
                $(
                    $(#[serde(rename = $req_name)])?
                    [<$rust_name Request>]( [<$rust_name Request>] ),
                )*
            }

            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            #[serde(tag = "messageType", content = "data")]
            pub enum ResponseData {
                $(
                    $(#[serde(rename = $resp_name)])?
                    [<$rust_name Response>]( [<$rust_name Response>] ),
                )*
            }

        }
    };
}

define_request_response_pairs!(
    {
        rust_name = ApiState,
        req_name = "APIStateRequest",
        resp_name = "APIStateResponse",
        req = {},
        resp = {
            pub active: bool,
            #[serde(rename = "vTubeStudioVersion")]
            pub vtubestudio_version: String,
            pub current_session_authenticated: bool,
        },
    },
    {
        rust_name = AuthenticationToken,
        req = {
            pub plugin_name: String,
            pub plugin_developer: String,
            pub plugin_icon: String,
        },
        resp = {
            pub authentication_token: String,
        },
    },
);

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request() {
        assert_eq!(
            serde_json::to_value(&Request {
                api_name: "VTubeStudioPublicAPI".into(),
                api_version: "1.0".into(),
                request_id: Some("MyIDWithLessThan64Characters".into()),
                data: ApiStateRequest {}.into(),
            })
            .unwrap(),
            json!({
                "apiName": "VTubeStudioPublicAPI",
                "apiVersion": "1.0",
                "requestID": "MyIDWithLessThan64Characters",
                "messageType": "APIStateRequest",
                "data": {}
            })
        )
    }

    #[test]
    fn response() {
        assert_eq!(
            serde_json::from_value::<Response>(json!({
                "apiName": "VTubeStudioPublicAPI",
                "apiVersion": "1.0",
                "timestamp": 1625405710728i64,
                "messageType": "APIStateResponse",
                "requestID": "MyIDWithLessThan64Characters",
                "data": {
                    "active": true,
                    "vTubeStudioVersion": "1.9.0",
                    "currentSessionAuthenticated": false
                }
            }))
            .unwrap(),
            Response {
                api_name: "VTubeStudioPublicAPI".into(),
                api_version: "1.0".into(),
                request_id: "MyIDWithLessThan64Characters".into(),
                timestamp: 1625405710728,
                data: ApiStateResponse {
                    active: true,
                    vtubestudio_version: "1.9.0".into(),
                    current_session_authenticated: false,
                }
                .into(),
            }
        )
    }

    #[test]
    fn request_response_pairs() {
        use std::convert::TryFrom;

        let resp = ApiStateResponse {
            active: true,
            vtubestudio_version: "1.9.0".into(),
            current_session_authenticated: false,
        };

        let resp_enum = ResponseData::from(resp.clone());

        assert_eq!(
            <ApiStateRequest as ApiRequest>::Response::try_from(resp_enum).unwrap(),
            resp
        );
    }
}
