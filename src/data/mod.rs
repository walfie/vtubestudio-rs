mod enumeration;
mod envelope;
mod error_id;

pub use crate::data::enumeration::EnumString;
pub use crate::data::envelope::{
    OpaqueValue, RequestEnvelope, RequestId, ResponseData, ResponseEnvelope, API_NAME, API_VERSION,
};
pub use crate::data::error_id::ErrorId;

use paste::paste;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Trait describing a VTube Studio request.
pub trait Request: Serialize {
    /// The message type of this request.
    const MESSAGE_TYPE: EnumString<RequestType>;

    /// The expected [`Response`] type for this request.
    type Response: Response;
}

/// Trait describing a VTube Studio response.
pub trait Response: DeserializeOwned + Send + 'static {
    /// The message type of this response.
    const MESSAGE_TYPE: EnumString<ResponseType>;
}

// https://github.com/DenchiSoft/VTubeStudio/blob/08681904e285d37b8c22d17d7d3a36c8c6834425/Files/HotkeyAction.cs
/// Known hotkey types for [`EnumString<HotkeyAction>`] (used in [`Hotkey`]).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HotkeyAction {
    /// Unset.
    Unset,
    /// Play an animation.
    TriggerAnimation,
    /// Change the idle animation.
    ChangeIdleAnimation,
    /// Toggle an expression.
    ToggleExpression,
    /// Remove all expressions.
    RemoveAllExpressions,
    /// Moves the model to the target position.
    MoveModel,
    /// Change the current background.
    ChangeBackground,
    /// Reload the current microphone.
    ReloadMicrophone,
    /// Reload the model texture.
    ReloadTextures,
    /// Calibrate Camera.
    CalibrateCam,
    /// Change VTS model.
    #[serde(rename = "ChangeVTSModel")]
    ChangeVtsModel,
    /// Takes a screenshot with the screenshot settings previously set in the UI.
    TakeScreenshot,
    /// Activates/Deactivates model screen color overlay.
    ScreenColorOverlay,
}

impl Default for HotkeyAction {
    fn default() -> Self {
        Self::Unset
    }
}

macro_rules! define_request_response_pairs {
    ($({
        rust_name = $rust_name:ident,
        $(req_name = $req_name:literal,)?
        $(resp_name = $resp_name:literal,)?
        req = { $($req:tt)* },
        resp = $(( $resp_inner:ident ))? $({ $($resp_fields:tt)* })?,
    },)*) => {
        paste! {
            /// Known message types for [`EnumString<RequestType>`].
            #[allow(missing_docs)]
            #[non_exhaustive]
            #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
            pub enum RequestType {
                $(
                    $(#[serde(rename = $req_name)])?
                    [<$rust_name Request>],
                )*
            }

            /// Known message types for [`EnumString<ResponseType>`].
            #[allow(missing_docs)]
            #[non_exhaustive]
            #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
            pub enum ResponseType {
                #[serde(rename = "APIError")]
                ApiError,
                $(
                    $(#[serde(rename = $resp_name)])?
                    [<$rust_name Response>],
                )*
                #[serde(rename = "VTubeStudioAPIStateBroadcast")]
                VTubeStudioApiStateBroadcast,
            }
        }

        $(
            paste! {
                #[allow(missing_docs)]
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Request>] { $($req)* }

                impl Request for [<$rust_name Request>] {
                    type Response = [<$rust_name Response>];
                    const MESSAGE_TYPE: EnumString<RequestType> = EnumString::new(RequestType::[<$rust_name Request>]);
                }

                #[allow(missing_docs)]
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Response>] $(($resp_inner);)? $({ $($resp_fields)* })?

                impl Response for [<$rust_name Response>] {
                    const MESSAGE_TYPE: EnumString<ResponseType> = EnumString::new(ResponseType::[<$rust_name Response>]);
                }
            }
        )*

    };
}

impl Default for RequestType {
    fn default() -> Self {
        Self::ApiStateRequest
    }
}

impl Default for ResponseType {
    fn default() -> Self {
        Self::ApiStateResponse
    }
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
            pub plugin_name: Cow<'static, str>,
            pub plugin_developer: Cow<'static, str>,
            pub plugin_icon: Option<Cow<'static, str>>,
        },
        resp = {
            pub authentication_token: String,
        },
    },

    {
        rust_name = Authentication,
        req = {
            pub plugin_name: Cow<'static, str>,
            pub plugin_developer: Cow<'static, str>,
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
            pub position_x: Option<f64>,
            pub position_y: Option<f64>,
            pub rotation: Option<f64>,
            pub size: Option<f64>,
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

#[allow(missing_docs)]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[error("APIError {error_id}: {message}")]
pub struct ApiError {
    #[serde(rename = "errorID")]
    pub error_id: ErrorId,
    pub message: String,
}

impl Response for ApiError {
    const MESSAGE_TYPE: EnumString<ResponseType> = EnumString::new(ResponseType::ApiError);
}

impl ApiError {
    /// Returns `true` if this error is an authentication error.
    pub fn is_unauthenticated(&self) -> bool {
        self.error_id.is_unauthenticated()
    }
}

#[allow(missing_docs)]
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
    const MESSAGE_TYPE: EnumString<ResponseType> =
        EnumString::new(ResponseType::VTubeStudioApiStateBroadcast);
}

/// Used in [`CurrentModelResponse`].
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPosition {
    pub position_x: f64,
    pub position_y: f64,
    pub rotation: f64,
    pub size: f64,
}

/// Used in [`AvailableModelsResponse`].
#[allow(missing_docs)]
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

/// Used in [`HotkeysInCurrentModelResponse`].
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: EnumString<HotkeyAction>,
    pub file: String,
    #[serde(rename = "hotkeyID")]
    pub hotkey_id: String,
}

/// Used in [`ColorTintRequest`].
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorTint {
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    pub mix_with_scene_lighting_color: Option<f64>,
    pub jeb_: bool,
}

impl Default for ColorTint {
    fn default() -> Self {
        Self {
            color_r: 0,
            color_g: 0,
            color_b: 0,
            color_a: 255,
            mix_with_scene_lighting_color: None,
            jeb_: false,
        }
    }
}

/// Used in [`ColorTintRequest`].
#[allow(missing_docs)]
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

/// Used in [`SceneColorOverlayInfoResponse`].
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePart {
    pub active: bool,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

/// Used in [`InputParameterListResponse`], [`ParameterValueResponse`], [`Live2DParameterListResponse`].
#[allow(missing_docs)]
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

/// Used in [`InjectParameterDataRequest`].
#[allow(missing_docs)]
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
    fn response_type_json() -> Result {
        assert_eq!(
            serde_json::from_value::<EnumString<ResponseType>>(json!("APIError"))?,
            EnumString::new(ResponseType::ApiError),
        );

        assert_eq!(
            serde_json::to_value::<EnumString<ResponseType>>(EnumString::new(
                ResponseType::ApiError
            ))?,
            json!("APIError"),
        );

        assert_eq!(
            serde_json::from_value::<EnumString<ResponseType>>(json!("ColorTintResponse"))?,
            ResponseType::ColorTintResponse,
        );

        assert_eq!(
            serde_json::to_value::<EnumString<ResponseType>>(
                ResponseType::ColorTintResponse.into()
            )?,
            json!("ColorTintResponse"),
        );

        assert_eq!(
            serde_json::from_value::<EnumString<ResponseType>>(json!("WalfieResponse"))?,
            EnumString::new_from_str("WalfieResponse"),
        );

        assert_eq!(
            serde_json::to_value(EnumString::<ResponseType>::new_from_str("WalfieResponse"))?,
            json!("WalfieResponse"),
        );

        Ok(())
    }

    #[test]
    fn request() -> Result {
        let mut req = RequestEnvelope::new(&ApiStateRequest {})?;
        req.request_id = Some("MyIDWithLessThan64Characters".into());

        let json = json!({
            "apiName": "VTubeStudioPublicAPI",
            "apiVersion": "1.0",
            "requestID": "MyIDWithLessThan64Characters",
            "messageType": "APIStateRequest",
            "data": {}
        });

        assert_eq!(serde_json::to_value(&req)?, json);
        assert_eq!(serde_json::from_value::<RequestEnvelope>(json)?, req);

        Ok(())
    }

    #[test]
    fn response() -> Result {
        let json = json!({
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
        });

        let resp = ResponseEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: "MyIDWithLessThan64Characters".into(),
            timestamp: 1625405710728,
            data: Ok(ResponseData {
                message_type: ApiStateResponse::MESSAGE_TYPE.into(),
                data: OpaqueValue::new(&ApiStateResponse {
                    active: true,
                    vtubestudio_version: "1.9.0".into(),
                    current_session_authenticated: false,
                })?,
            }),
        };

        assert_eq!(serde_json::to_value(&resp)?, json);
        assert_eq!(serde_json::from_value::<ResponseEnvelope>(json)?, resp);

        Ok(())
    }

    #[test]
    fn api_error() -> Result {
        let json = json!({
            "apiName": "VTubeStudioPublicAPI",
            "apiVersion": "1.0",
            "timestamp": 1625405710728i64,
            "requestID": "SomeID",
            "messageType": "APIError",
            "data": {
                "errorID": 1,
                "message": "Error message"
            }
        });

        let resp = ResponseEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: "SomeID".into(),
            timestamp: 1625405710728,
            data: Err(ApiError {
                error_id: ErrorId::API_ACCESS_DEACTIVATED,
                message: "Error message".into(),
            }),
        };

        assert_eq!(serde_json::to_value(&resp)?, json);
        assert_eq!(serde_json::from_value::<ResponseEnvelope>(json)?, resp);

        Ok(())
    }

    #[test]
    fn parameter_value_response() -> Result {
        let json = json!({
            "apiName": "VTubeStudioPublicAPI",
            "apiVersion": "1.0",
            "timestamp": 1625405710728i64,
            "requestID": "SomeID",
            "messageType": "ParameterValueResponse",
            "data": {
                "name": "MyCustomParamName1",
                "addedBy": "My Plugin Name",
                "value": 12.4,
                "min": -30.0,
                "max": 30.0,
                "defaultValue": 0.0
            }
        });

        let resp = ResponseEnvelope {
            api_name: "VTubeStudioPublicAPI".into(),
            api_version: "1.0".into(),
            request_id: "SomeID".into(),
            timestamp: 1625405710728,
            data: Ok(ResponseData {
                message_type: ParameterValueResponse::MESSAGE_TYPE.into(),
                data: OpaqueValue::new(&ParameterValueResponse(Parameter {
                    name: "MyCustomParamName1".into(),
                    added_by: "My Plugin Name".into(),
                    value: 12.4,
                    min: -30.0,
                    max: 30.0,
                    default_value: 0.0,
                }))?,
            }),
        };

        assert_eq!(serde_json::to_value(&resp)?, json);
        assert_eq!(serde_json::from_value::<ResponseEnvelope>(json)?, resp);

        Ok(())
    }

    #[test]
    fn parse_response() -> Result {
        let data = ApiStateResponse {
            active: true,
            vtubestudio_version: "1.9.0".into(),
            current_session_authenticated: false,
        };

        let resp = ResponseEnvelope::new(&data)?;

        let parsed = resp.parse::<ApiStateResponse>()?;

        assert_eq!(parsed, data);

        Ok(())
    }
}