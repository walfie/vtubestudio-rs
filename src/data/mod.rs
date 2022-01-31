//! Request/response types for the VTube Studio API.
//!
//! For a list of all request types, see the implementors for [`Request`].
//! For the corresponding response types, see [`Response`].

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
use serde::{Deserialize, Serialize, Serializer};
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
    /// Removes all items from the scene.
    RemoveAllItems,
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
        $(#[doc = $req_doc:expr])+
        req = { $($req:tt)* },
        $(#[doc = $resp_doc:expr])+
        resp = $(( $($resp_inner:tt)+ ))? $({ $($resp_fields:tt)* })?,
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
                $(#[doc = $req_doc])+
                ///
                #[doc = concat!("This request returns [`", stringify!($rust_name), "Response`].")]
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Request>] { $($req)* }

                impl Request for [<$rust_name Request>] {
                    type Response = [<$rust_name Response>];

                    #[doc = concat!("[`RequestType::", stringify!($rust_name), "Request`]")]
                    const MESSAGE_TYPE: EnumString<RequestType> = EnumString::new(RequestType::[<$rust_name Request>]);
                }

                $(#[doc = $resp_doc])+
                ///
                #[doc = concat!("This is the return value of [`", stringify!($rust_name), "Request`].")]
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_name Response>] $(( $($resp_inner)+ );)? $({ $($resp_fields)* })?

                impl Response for [<$rust_name Response>] {
                    #[doc = concat!("[`ResponseType::", stringify!($rust_name), "Response`]")]
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
        /// Get the current state of the API.
        req = {},
        /// The API state.
        resp = {
            /// Whether the API is active.
            pub active: bool,
            /// The VTube Studio version.
            #[serde(rename = "vTubeStudioVersion")]
            pub vtubestudio_version: String,
            /// Whether the current session is authenticated.
            pub current_session_authenticated: bool,
        },
    },

    {
        rust_name = AuthenticationToken,
        /// Request an authentication token.
        req = {
            /// The name of the plugin.
            pub plugin_name: Cow<'static, str>,
            /// The developer of the plugin.
            pub plugin_developer: Cow<'static, str>,
            /// A Base64 encoded image representing the plugin icon.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub plugin_icon: Option<Cow<'static, str>>,
        },
        /// Authentication token response.
        resp = {
            /// The authentication token.
            pub authentication_token: String,
        },
    },

    {
        rust_name = Authentication,
        /// Authenticate with the API using a token.
        req = {
            /// The name of the plugin.
            pub plugin_name: Cow<'static, str>,
            /// The developer of the plugin.
            pub plugin_developer: Cow<'static, str>,
            /// The authentication token.
            pub authentication_token: String,
        },
        /// Whether the authentication request was successful.
        resp = {
            /// Whether the session is authenticated.
            pub authenticated: bool,
            /// A human-readable explanation of the authentication status.
            pub reason: String,
        },
    },

    {
        rust_name = Statistics,
        /// Getting current VTS statistics.
        req = {},
        /// Statistics about the VTube Studio session.
        resp = {
            /// Uptime in milliseconds.
            pub uptime: i64,
            /// The frame rate.
            pub framerate: i32,
            /// The VTube Studio version.
            #[serde(rename = "vTubeStudioVersion")]
            pub vtubestudio_version: String,
            /// Number of plugins registered.
            pub allowed_plugins: i32,
            /// Number of plugins currently connected.
            pub connected_plugins: i32,
            /// Whether VTube Studio was started with Steam.
            pub started_with_steam: bool,
            /// Width of the window.
            pub window_width: i32,
            /// Height of the window.
            pub window_height: i32,
            /// Whether the window is in fullscreen mode.
            pub window_is_fullscreen: bool,
        },
    },

    {
        rust_name = VtsFolderInfo,
        req_name = "VTSFolderInfoRequest",
        resp_name = "VTSFolderInfoResponse",
        /// Getting list of VTS folders.
        req = {},
        /// Names of various folders in the `StreamingAssets` directory.
        resp = {
            /// The models folder.
            pub models: String,
            /// The backgrounds folder.
            pub backgrounds: String,
            /// The items folder.
            pub items: String,
            /// The config folder.
            pub config: String,
            /// The logs folder.
            pub logs: String,
            /// The backup folder.
            pub backup: String,
        },
    },

    {
        rust_name = CurrentModel,
        /// Getting the currently loaded model.
        req = {},
        /// Information about the current model.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// The VTube Studio JSON file for this model.
            ///
            /// E.g., `"Model.vtube.json"`
            pub vts_model_name: String,
            /// The image name of this model's VTube Studio icon.
            pub vts_model_icon_name: String,
            /// The Live2D model JSON file.
            ///
            /// E.g., `"Model.model3.json"`
            #[serde(rename = "live2DModelName")]
            pub live2d_model_name: String,
            /// How many milliseconds it took to load the model.
            pub model_load_time: i64,
            /// Milliseconds elapsed since the model was loaded.
            pub time_since_model_loaded: i64,
            /// Number of Live2D parameters.
            #[serde(rename = "numberOfLive2DParameters")]
            pub number_of_live2d_parameters: i32,
            /// Number of Live2D art meshes.
            #[serde(rename = "numberOfLive2DArtmeshes")]
            pub number_of_live2d_artmeshes: i32,
            /// Whether the model has a physics file.
            pub has_physics_file: bool,
            /// Number of textures.
            pub number_of_textures: i32,
            /// The resolution of the texture. E.g., `4096`
            pub texture_resolution: i32,
            /// The position of the model.
            pub model_position: ModelPosition,
        },
    },

    {
        rust_name = AvailableModels,
        /// Getting a list of available VTS models
        req = {},
        /// List of available models.
        resp = {
            /// Number of models.
            pub number_of_models: i32,
            /// List of models.
            pub available_models: Vec<Model>,
        },
    },

    {
        rust_name = ModelLoad,
        /// Loading a VTS model by its ID.
        req = {
            /// The ID of the model to load.
            #[serde(rename = "modelID")]
            pub model_id: String,
        },
        /// Information about the loaded model ID.
        resp = {
            /// The ID of the model loaded.
            #[serde(rename = "modelID")]
            pub model_id: String,
        },
    },

    {
        rust_name = MoveModel,
        /// Moving the currently loaded VTS model.
        req = {
            /// How many seconds the animation should take. Maximum `2`.
            pub time_in_seconds: f64,
            /// If `true`, apply movements relative to the model's current state.
            pub values_are_relative_to_model: bool,
            /// Horizontal position. `-1` for left edge, `1` for right edge.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub position_x: Option<f64>,
            /// Vertical position. `-1` for bottom edge, `1` for top edge.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub position_y: Option<f64>,
            /// Rotation in degrees. Must be between `-360` and `360`.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub rotation: Option<f64>,
            /// Size, between `-100` and `100`.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub size: Option<f64>,
        },
        /// Empty response on model move success.
        resp = {},
    },

    {
        rust_name = HotkeysInCurrentModel,
        /// Requesting list of hotkeys available in current or other VTS model.
        req = {
            /// The ID of the model.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "modelID")]
            pub model_id: Option<String>,
        },
        /// Model info and list of hotkeys.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// List of available hotkeys.
            pub available_hotkeys: Vec<Hotkey>,
        },
    },

    {
        rust_name = HotkeyTrigger,
        /// Requesting execution of hotkeys.
        req = {
            /// The ID of the hotkey.
            #[serde(rename = "hotkeyID")]
            pub hotkey_id: String,
        },
        /// The hotkey that was triggered.
        resp = {
            /// The ID of the hotkey.
            #[serde(rename = "hotkeyID")]
            pub hotkey_id: String,
        },
    },

    {
        rust_name = ArtMeshList,
        /// Requesting list of art meshes in current model.
        req = {},
        /// List of art meshes.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// Number of art mesh names.
            pub number_of_art_mesh_names: i32,
            /// Number of art mesh tags.
            pub number_of_art_mesh_tags: i32,
            /// List of art mesh names.
            pub art_mesh_names: Vec<String>,
            /// List of art mesh tags.
            pub art_mesh_tags: Vec<String>,
        },
    },

    {
        rust_name = ColorTint,
        /// Tint art meshes with color
        req = {
            /// The color tint information.
            pub color_tint: ColorTint,
            /// Which art meshes should be tinted.
            pub art_mesh_matcher: ArtMeshMatcher,
        },
        /// Number of matched art meshes.
        resp = {
            /// Number of matched art meshes.
            pub matched_art_meshes: i32,
        },
    },

    {
        rust_name = SceneColorOverlayInfo,
        /// Getting scene lighting overlay color.
        req = {},
        /// Info about the color overlay.
        resp = {
            /// Whether the overlay is active.
            pub active: bool,
            /// Whether items are included in the overlay.
            pub items_included: bool,
            /// Whether the overlay is a window capture.
            ///
            /// If `false`, it means the entire screen is being captured.
            pub is_window_capture: bool,
            /// Base brightness (between 0 and 100).
            pub base_brightness: i32,
            /// Color boost (between 0 and 100).
            pub color_boost: i32,
            /// Smoothing.(between 0 and 60).
            pub smoothing: i32,
            /// The red component of the overlay (between 0 and 459).
            pub color_overlay_r: i32,
            /// The green component of the overlay (between 0 and 459).
            pub color_overlay_g: i32,
            /// The blue component of the overlay (between 0 and 459).
            pub color_overlay_b: i32,
            /// The average red component of the overlay.
            pub color_avg_r: u8,
            /// The average green component of the overlay.
            pub color_avg_g: u8,
            /// The average blue component of the overlay.
            pub color_avg_b: u8,
            /// The left capture part.
            pub left_capture_part: CapturePart,
            /// The middle capture part.
            pub middle_capture_part: CapturePart,
            /// The right capture part.
            pub right_capture_part: CapturePart,
        },
    },

    {
        rust_name = FaceFound,
        /// Checking if face is currently found by tracker.
        req = {},
        /// Whether the face was found.
        resp = {
            /// Whether the face was found.
            pub found: bool,
        },
    },

    {
        rust_name = InputParameterList,
        /// Requesting list of available tracking parameters.
        req = {},
        /// List of available parameters.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// List of custom parameters.
            pub custom_parameters: Vec<Parameter>,
            /// List of default parameters.
            pub default_parameters: Vec<Parameter>,
        },
    },

    {
        rust_name = ParameterValue,
        /// Get the value for one specific parameter, default or custom.
        req = {
            /// The name of the parameter.
            pub name: String,
        },
        /// The requested parameter.
        resp = (
            /// The requested parameter.
            pub Parameter
        ),
    },

    {
        rust_name = Live2DParameterList,
        /// Get the value for all Live2D parameters in the current model.
        req = {},
        /// Info about the current model and list of parameters.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// List of parameters.
            pub parameters: Vec<Parameter>,
        },
    },

    {
        rust_name = ParameterCreation,
        /// Adding new tracking parameters ("custom parameters").
        req = {
            /// Name of the parameter.
            pub parameter_name: String,
            /// A description of the parameter.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub explanation: Option<String>,
            /// The minimum value.
            pub min: f64,
            /// The maximum value.
            pub max: f64,
            /// The default value.
            pub default_value: f64,
        },
        /// Name of the created parameter.
        resp = {
            /// Name of the created parameter.
            pub parameter_name: String,
        },
    },

    {
        rust_name = ParameterDeletion,
        /// Delete custom parameters.
        req = {
            /// The name of the parameter to delete.
            pub parameter_name: String,
        },
        /// Name of the deleted parameter.
        resp = {
            /// Name of the deleted parameter.
            pub parameter_name: String,
        },
    },

    {
        rust_name = InjectParameterData,
        /// Feeding in data for default or custom parameters.
        req = {
            /// The parameter values to inject.
            pub parameter_values: Vec<ParameterValue>,
        },
        /// Empty response on parameter injection success.
        resp = {},
    },

    {
        rust_name = ExpressionState,
        /// Requesting current expression state list.
        req = {
            /// Whether to return more details in the response.
            ///
            /// This affects whether items are returned in the `used_in_hotkeys` and `parameters`
            /// fields.
            pub details: bool,
            /// If specified, return only the state of this expression.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub expression_file: Option<String>,
        },
        /// Data about the requested expressions.
        resp = {
            /// Whether the model is loaded.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// List of expressions.
            pub expressions: Vec<Expression>,
        },
    },

    {
        rust_name = ExpressionActivation,
        /// Requesting activation or deactivation of expressions.
        req = {
            /// File name of the expression file.
            ///
            /// E.g., `myExpression_1.exp3.json`.
            pub expression_file: String,
            /// Whether the expression should be active.
            pub active: bool,
        },
        /// Empty response on successful expression activation/deactivation.
        resp = {},
    },

    {
        rust_name = NdiConfigRequest,
        req_name = "NDIConfigRequest",
        resp_name = "NDIConfigResponse",
        /// Get and set NDI settings.
        req = {
            /// Set to `false` to only return existing config (other fields will be ignored).
            pub set_new_config: bool,
            /// Whether NDI should be active.
            pub ndi_active: bool,
            /// Whether NDI 5 should be used.
            #[serde(rename = "useNDI5")]
            pub use_ndi5: bool,
            /// Whether a custom resolution should be used.
            ///
            /// Setting this to `true` means the NDI stream will no longer have
            /// the same resolution as the VTube Studio window, but instead use
            /// the custom resolution set via the UI or the `custom_width`
            /// fields of this request.
            pub use_custom_resolution: bool,
            /// Custom NDI width if `use_custom_resolution` is specified.
            ///
            /// Must be a multiple of 16 and be between `256` and `8192`.
            #[serde(rename = "customWidthNDI", serialize_with = "ndi_default_size")]
            pub custom_width_ndi: Option<i32>,
            /// Custom NDI height if `use_custom_resolution` is specified.
            ///
            /// Must be a multiple of 8 and be between `256` and `8192`.
            #[serde(rename = "customHeightNDI", serialize_with = "ndi_default_size")]
            pub custom_height_ndi: Option<i32>,
        },
        /// Data about the requested expressions.
        resp = {
            /// This field has no significance in the response.
            pub set_new_config: bool,
            /// Whether NDI is active.
            pub ndi_active: bool,
            /// Whether NDI 5 is being used.
            #[serde(rename = "useNDI5")]
            pub use_ndi5: bool,
            /// Whether a custom resolution is being used.
            pub use_custom_resolution: bool,
            /// The NDI width.
            #[serde(rename = "customWidthNDI")]
            pub custom_width_ndi: i64,
            /// The NDI height.
            #[serde(rename = "customHeightNDI")]
            pub custom_height_ndi: i64,
        },
    },

);

// Per the docs, we should send `-1` if the user doesn't want to change the NDI width or height.
fn ndi_default_size<S>(value: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(value.unwrap_or(-1))
}

/// Error returned by the VTube Studio API.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[error("APIError {error_id}: {message}")]
pub struct ApiError {
    /// The error ID.
    #[serde(rename = "errorID")]
    pub error_id: ErrorId,
    /// A description of the error.
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

/// API server discovery message (sent over UDP).
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VTubeStudioApiStateBroadcast {
    /// Whether the API is active.
    pub active: bool,
    /// The websocket port.
    pub port: i32,
    /// The ID of the VTube Studio instance.
    #[serde(rename = "instanceID")]
    pub instance_id: String,
    /// The title of the VTube Studio window.
    pub window_title: String,
}

impl Response for VTubeStudioApiStateBroadcast {
    const MESSAGE_TYPE: EnumString<ResponseType> =
        EnumString::new(ResponseType::VTubeStudioApiStateBroadcast);
}

/// Used in [`CurrentModelResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPosition {
    /// The X position of the model.
    pub position_x: f64,
    /// The Y position of the model.
    pub position_y: f64,
    /// The rotation of the model in degrees.
    pub rotation: f64,
    /// The size of the model (between -100 and 100).
    pub size: f64,
}

/// Used in [`AvailableModelsResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Whether the model is loaded.
    pub model_loaded: bool,
    /// The name of the model.
    pub model_name: String,
    /// The ID of the model.
    #[serde(rename = "modelID")]
    pub model_id: String,
    /// The VTube Studio JSON file for this model.
    pub vts_model_name: String,
    /// The image name of this model's VTube Studio icon.
    pub vts_model_icon_name: String,
}

/// Used in [`HotkeysInCurrentModelResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    /// The name of the hotkey.
    pub name: String,
    /// The hotkey type.
    #[serde(rename = "type")]
    pub type_: EnumString<HotkeyAction>,
    /// The JSON file associated with this hotkey, if any (possibly an empty string).
    ///
    /// E.g., `"myExpression_1.exp3.json"`, `"myAnimation.motion3.json"`, `"someOtherModel.vtube.json"`.
    pub file: String,
    /// Unique ID of the hotkey.
    #[serde(rename = "hotkeyID")]
    pub hotkey_id: String,
    /// Human-readable description of the hotkey type.
    pub description: Option<String>,
}

/// Used in [`ColorTintRequest`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorTint {
    /// The red component of the color.
    pub color_r: u8,
    /// The green component of the color.
    pub color_g: u8,
    /// The blue component of the color.
    pub color_b: u8,
    /// The alpha component of the color.
    pub color_a: u8,
    /// The weight of this color tint relative to the scene lighting.
    ///
    /// This should be a value between 0 and 1 (where 0 means the scene lighting takes full
    /// priority, and 1 means this color tint takes full priority), with the default being 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mix_with_scene_lighting_color: Option<f64>,
    /// Enable rainbow mode.
    #[serde(rename = "jeb_")]
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
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtMeshMatcher {
    /// Whether to tint all art meshes.
    pub tint_all: bool,
    /// The number of this art mesh.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub art_mesh_number: Vec<i32>,
    /// Match art meshes with these exact names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub name_exact: Vec<String>,
    /// Match art meshes that contain these strings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub name_contains: Vec<String>,
    /// Match art meshes with these exact tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tag_exact: Vec<String>,
    /// Match art meshes that have tags that contain these strings.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tag_contains: Vec<String>,
}

/// Used in [`SceneColorOverlayInfoResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePart {
    /// Whether this capture part is active.
    pub active: bool,
    /// The red component of the color.
    pub color_r: u8,
    /// The green component of the color.
    pub color_g: u8,
    /// The blue component of the color.
    pub color_b: u8,
}

/// Used in [`InputParameterListResponse`], [`ParameterValueResponse`], [`Live2DParameterListResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    /// The name of the parameter.
    pub name: String,
    /// The plugin that created this parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub added_by: Option<String>,
    /// The current value.
    pub value: f64,
    /// The minimum value.
    pub min: f64,
    /// The maximum value.
    pub max: f64,
    /// The default value.
    pub default_value: f64,
}

/// Used in [`InjectParameterDataRequest`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterValue {
    /// The ID (name) of the parameter.
    pub id: String,
    /// The value of the parameter.
    pub value: f64,
    /// The weight of this parameter injection value compared to values provided by facial
    /// tracking.
    ///
    /// This value should be between 0 and 1 (with 1 being the default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

/// Used in [`ExpressionStateResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Expression {
    /// Name of the expression.
    ///
    /// E.g., `myExpression_optional_1`.
    pub name: String,
    /// File name of the expression.
    ///
    /// E.g., `myExpression_optional_1.exp3.json`.
    pub file: String,
    /// Whether the expression is active.
    pub active: bool,
    /// Whether the expression deactivates when let go.
    pub deactivate_when_key_is_let_go: bool,
    /// Whether the expression auto-deactivates after some time.
    pub auto_deactivate_after_seconds: bool,
    /// Seconds remaining until the expression deactivates.
    ///
    /// This will be `0` if `auto_deactivate_after_seconds` is `false`.
    pub seconds_remaining: f64,
    /// Which hotkeys this expression is used in.
    pub used_in_hotkeys: Vec<ExpressionUsedInHotkey>,
    /// The Live2D parameter IDs and target values of all parameters used in the expression.
    pub parameters: Vec<ExpressionParameter>,
}

/// Used in [`Expression`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpressionParameter {
    /// Live2D parameter name of the expression.
    pub name: String,
    /// Value of the expression.
    pub value: i32,
}

/// Used in [`Expression`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpressionUsedInHotkey {
    /// Name of the hotkey.
    pub name: String,
    /// ID of the hotkey.
    pub id: String,
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
                    added_by: Some("My Plugin Name".into()),
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
