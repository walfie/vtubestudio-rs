use paste::paste;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::convert::TryFrom;

pub const API_NAME: &'static str = "VTubeStudioPublicAPI";
pub const API_VERSION: &'static str = "1.0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEnvelope {
    pub api_name: Cow<'static, str>,
    pub api_version: Cow<'static, str>,
    #[serde(rename = "requestID")]
    pub request_id: Option<String>,
    #[serde(flatten)]
    pub data: RequestData,
}

impl RequestEnvelope {
    pub fn new(data: RequestData) -> Self {
        Self {
            api_name: Cow::Borrowed(API_NAME),
            api_version: Cow::Borrowed(API_VERSION),
            request_id: None,
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseEnvelope {
    pub api_name: String,
    pub api_version: String,
    pub timestamp: i64,
    #[serde(rename = "requestID")]
    pub request_id: String,
    #[serde(flatten)]
    pub data: ResponseData,
}

pub trait Request: Into<RequestData> {
    const MESSAGE_TYPE: &'static str;
    type Response: Response;
}

pub trait Response:
    DeserializeOwned + Into<ResponseData> + TryFrom<ResponseData, Error = ResponseData> + Send + 'static
{
    const MESSAGE_TYPE: &'static str;
}

macro_rules! first_expr {
    ($value:expr) => {
        $value
    };
    ($value:expr, $_:expr) => {
        $value
    };
}

macro_rules! impl_enum {
    ($enum:ident, $variant:ident) => {
        impl From<$variant> for $enum {
            fn from(value: $variant) -> Self {
                $enum::$variant(value)
            }
        }

        impl std::convert::TryFrom<$enum> for $variant {
            type Error = $enum;

            fn try_from(value: $enum) -> Result<Self, Self::Error> {
                if let $enum::$variant(inner) = value {
                    Ok(inner)
                } else {
                    Err(value)
                }
            }
        }

        impl<'a> std::convert::TryFrom<&'a $enum> for &'a $variant {
            type Error = ();

            fn try_from(value: &'a $enum) -> Result<Self, Self::Error> {
                if let $enum::$variant(inner) = value {
                    Ok(inner)
                } else {
                    Err(())
                }
            }
        }
    };
}

macro_rules! define_request_response_pairs {
    ($({
        rust_name = $rust_name:ident,
        $(req_name = $req_name:literal,)?
        $(resp_name = $resp_name:literal,)?
        req = { $($req:tt)* },
        resp = $(( $resp_inner:ident ))? $({ $($resp_fields:tt)* })?,
    },)*) => {
        $(
            paste! {
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Request>] { $($req)* }

                impl Request for [<$rust_name Request>] {
                    type Response = [<$rust_name Response>];
                    const MESSAGE_TYPE: &'static str = first_expr![
                        $($resp_name,)?
                        concat!(stringify!($rust_name), "Request")
                    ];
                }

                impl_enum!(RequestData, [<$rust_name Request>]);

                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Response>] $(($resp_inner);)? $({ $($resp_fields)* })?

                impl Response for [<$rust_name Response>] {
                    const MESSAGE_TYPE: &'static str = first_expr![
                        $($req_name,)?
                        concat!(stringify!($rust_name), "Response")
                    ];
                }

                impl_enum!(ResponseData, [<$rust_name Response>]);
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
                #[serde(rename = "APIError")]
                ApiError(ApiError),
                #[serde(rename = "VTubeStudioAPIStateBroadcast")]
                VTubeStudioApiStateBroadcast(VTubeStudioApiStateBroadcast),
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
            pub plugin_icon: Option<String>,
        },
        resp = {
            pub authentication_token: String,
        },
    },

    {
        rust_name = Authentication,
        req = {
            pub plugin_name: String,
            pub plugin_developer: String,
            pub authentication_token: String,
        },
        resp = {
            pub authenticated: bool,
            pub reason: String,
        },
    },

    {
        rust_name = Statistics,
        req = {},
        resp = {
            pub uptime: i64,
            pub framerate: i32,
            #[serde(rename = "vTubeStudioVersion")]
            pub vtubestudio_version: String,
            pub allowed_plugins: i32,
            pub connected_plugins: i32,
            pub started_with_steam: bool,
            pub window_width: i32,
            pub window_height: i32,
            pub window_is_fullscreen: bool,
        },
    },

    {
        rust_name = VtsFolderInfo,
        req_name = "VTSFolderInfoRequest",
        resp_name = "VTSFolderInfoResponse",
        req = {},
        resp = {
            pub models: String,
            pub backgrounds: String,
            pub items: String,
            pub config: String,
            pub logs: String,
            pub backup: String,
        },
    },

    {
        rust_name = CurrentModel,
        req = {},
        resp = {
            pub model_loaded: bool,
            pub model_name: String,
            #[serde(rename = "modelID")]
            pub model_id: String,
            pub vts_model_name: String,
            pub vts_model_icon_name: String,
            #[serde(rename = "live2DModelName")]
            pub live2d_model_name: String,
            pub model_load_time: i64,
            pub time_since_model_loaded: i64,
            #[serde(rename = "numberOfLive2DParameters")]
            pub number_of_live2d_parameters: i32,
            #[serde(rename = "numberOfLive2DArtmeshes")]
            pub number_of_live2d_artmeshes: i32,
            pub has_physics_file: bool,
            pub number_of_textures: i32,
            pub texture_resolution: i32,
            pub model_position: ModelPosition,
        },
    },

    {
        rust_name = AvailableModels,
        req = {},
        resp = {
            pub number_of_models: i32,
            pub available_models: Vec<Model>,
        },
    },

    {
        rust_name = ModelLoad,
        req = {
            #[serde(rename = "modelID")]
            pub model_id: String,
        },
        resp = {
            #[serde(rename = "modelID")]
            pub model_id: String,
        },
    },


    {
        rust_name = MoveModel,
        req = {
            pub time_in_seconds: f64,
            pub values_are_relative_to_model: bool,
            pub position_x: f64,
            pub position_y: f64,
            pub rotation: f64,
            pub size: f64,
        },
        resp = {},
    },

    {
        rust_name = HotkeysInCurrentModel,
        req = {
            #[serde(rename = "modelID")]
            pub model_id: Option<String>,
        },
        resp = {
            pub model_loaded: bool,
            pub model_name: String,
            #[serde(rename = "modelID")]
            pub model_id: String,
            pub available_hotkeys: Vec<Hotkey>,
        },
    },

    {
        rust_name = HotkeyTrigger,
        req = {
            #[serde(rename = "hotkeyID")]
            pub hotkey_id: String,
        },
        resp = {
            #[serde(rename = "hotkeyID")]
            pub hotkey_id: String,
        },
    },

    {
        rust_name = ArtMeshList,
        req = {},
        resp = {
            pub model_loaded: bool,
            pub number_of_art_mesh_names: i32,
            pub number_of_art_mesh_tags: i32,
            pub art_mesh_names: Vec<String>,
            pub art_mesh_tags: Vec<String>,
        },
    },

    {
        rust_name = ColorTint,
        req = {
            pub color_tint: ColorTint,
            pub art_mesh_matcher: ArtMeshMatcher,
        },
        resp = {
            pub matched_art_meshes: i32,
        },
    },

    {
        rust_name = SceneColorOverlayInfo,
        req = {},
        resp = {
            pub active: bool,
            pub items_included: bool,
            pub is_window_capture: bool,
            pub base_brightness: i32,
            pub color_boost: i32,
            pub smoothing: i32,
            pub color_overlay_r: u8,
            pub color_overlay_g: u8,
            pub color_overlay_b: u8,
            pub color_avg_r: u8,
            pub color_avg_g: u8,
            pub color_avg_b: u8,
            pub left_capture_part: CapturePart,
            pub middle_capture_part: CapturePart,
            pub right_capture_part: CapturePart,
        },
    },

    {
        rust_name = FaceFound,
        req = {},
        resp = {
            pub found: bool,
        },
    },

    {
        rust_name = InputParameterList,
        req = {},
        resp = {
            pub model_loaded: bool,
            pub model_name: String,
            #[serde(rename = "modelID")]
            pub model_id: String,
            pub custom_parameters: Vec<Parameter>,
            pub default_parameters: Vec<Parameter>,
        },
    },

    {
        rust_name = ParameterValue,
        req = {
            pub name: String,
        },
        resp = (Parameter),
    },

    {
        rust_name = Live2DParameterList,
        req = {},
        resp = {
            pub model_loaded: bool,
            pub model_name: String,
            #[serde(rename = "modelID")]
            pub model_id: String,
            pub parameters: Vec<Parameter>,
        },
    },

    {
        rust_name = ParameterCreation,
        req = {
            pub parameter_name: String,
            pub explanation: Option<String>,
            pub min: f64,
            pub max: f64,
            pub default_value: f64,
        },
        resp = {
            pub parameter_name: String,
        },
    },

    {
        rust_name = ParameterDeletion,
        req = {
            pub parameter_name: String,
        },
        resp = {
            pub parameter_name: String,
        },
    },

    {
        rust_name = InjectParameterData,
        req = {
            pub parameter_values: Vec<ParameterValue>,
        },
        resp = {},
    },

);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    // TODO: ErrorId enum
    // https://github.com/DenchiSoft/VTubeStudio/blob/master/Files/ErrorID.cs
    #[serde(rename = "errorID")]
    pub error_id: i32,
    pub message: String,
}

impl Response for ApiError {
    const MESSAGE_TYPE: &'static str = "ApiError";
}

impl ApiError {
    pub fn is_auth_error(&self) -> bool {
        self.error_id == 8
    }
}

impl_enum!(ResponseData, ApiError);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VTubeStudioApiStateBroadcast {
    pub active: bool,
    pub port: i32,
    #[serde(rename = "instanceID")]
    pub instance_id: String,
    pub window_title: String,
}

impl Response for VTubeStudioApiStateBroadcast {
    const MESSAGE_TYPE: &'static str = "VTubeStudioAPIStateBroadcast";
}

impl_enum!(ResponseData, VTubeStudioApiStateBroadcast);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPosition {
    pub position_x: f64,
    pub position_y: f64,
    pub rotation: f64,
    pub size: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub model_loaded: bool,
    pub model_name: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
    pub vts_model_name: String,
    pub vts_model_icon_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    pub name: String,
    // TODO: HotkeyType enum
    // https://github.com/DenchiSoft/VTubeStudio/blob/master/Files/HotkeyAction.cs
    #[serde(rename = "type")]
    pub type_: String,
    pub file: String,
    #[serde(rename = "hotkeyID")]
    pub hotkey_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorTint {
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    pub mix_with_scene_lighting_color: Option<f64>,
    #[serde(rename = "jeb_")]
    pub jeb: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtMeshMatcher {
    pub tint_all: bool,
    pub art_mesh_number: Vec<i32>,
    pub name_exact: Vec<String>,
    pub name_contains: Vec<String>,
    pub tag_exact: Vec<String>,
    pub tag_contains: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePart {
    pub active: bool,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    pub added_by: String,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub default_value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    pub id: String,
    pub value: f64,
    pub weight: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn request() -> Result {
        assert_eq!(
            serde_json::to_value(&RequestEnvelope {
                api_name: "VTubeStudioPublicAPI".into(),
                api_version: "1.0".into(),
                request_id: Some("MyIDWithLessThan64Characters".into()),
                data: ApiStateRequest {}.into(),
            })?,
            json!({
                "apiName": "VTubeStudioPublicAPI",
                "apiVersion": "1.0",
                "requestID": "MyIDWithLessThan64Characters",
                "messageType": "APIStateRequest",
                "data": {}
            })
        );

        Ok(())
    }

    #[test]
    fn response() -> Result {
        assert_eq!(
            serde_json::from_value::<ResponseEnvelope>(json!({
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
            }))?,
            ResponseEnvelope {
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
        );

        Ok(())
    }

    #[test]
    fn parameter_value_response() -> Result {
        assert_eq!(
            serde_json::from_value::<ResponseEnvelope>(json!({
                "apiName": "VTubeStudioPublicAPI",
                "apiVersion": "1.0",
                "timestamp": 1625405710728i64,
                "requestID": "SomeID",
                "messageType": "ParameterValueResponse",
                "data": {
                    "name": "MyCustomParamName1",
                    "addedBy": "My Plugin Name",
                    "value": 12.4,
                    "min": -30,
                    "max": 30,
                    "defaultValue": 0
                }
            }))?,
            ResponseEnvelope {
                api_name: "VTubeStudioPublicAPI".into(),
                api_version: "1.0".into(),
                request_id: "SomeID".into(),
                timestamp: 1625405710728,
                data: ParameterValueResponse(Parameter {
                    name: "MyCustomParamName1".into(),
                    added_by: "My Plugin Name".into(),
                    value: 12.4,
                    min: -30.0,
                    max: 30.0,
                    default_value: 0.0
                })
                .into(),
            }
        );

        Ok(())
    }

    #[test]
    fn request_response_pairs() -> Result {
        use std::convert::TryFrom;

        let resp = ApiStateResponse {
            active: true,
            vtubestudio_version: "1.9.0".into(),
            current_session_authenticated: false,
        };

        let resp_enum = ResponseData::from(resp.clone());

        assert_eq!(
            <ApiStateRequest as Request>::Response::try_from(resp_enum).unwrap(),
            resp
        );

        Ok(())
    }
}
