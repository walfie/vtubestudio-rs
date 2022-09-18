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
    /// Loads an item scene.
    ToggleItemScene,
    /// Downloads a random item from the Steam Workshop and attempts to load it into the scene.
    DownloadRandomWorkshopItem,
    /// Executes a hotkey in the given Live2D item.
    ExecuteItemAction,
    /// Loads the recorded ArtMesh multiply/screen color preset.
    ArtMeshColorPreset,
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
        ///
        /// If `model_id` is absent, hotkeys for the current model are returned.
        ///
        /// If both `model_id` and `live2d_item_file_name` are provided, only `model_id` is used
        /// and the other field will be ignored.
        req = {
            /// The ID of the model.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "modelID")]
            pub model_id: Option<String>,
            /// Set this field to request hotkeys for a Live2D item.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "live2DItemFileName")]
            pub live2d_item_file_name: Option<String>,
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
            /// If present, trigger the hotkey for the given Live2D item. If absent, the hotkey
            /// will be triggered for the currently loaded model.
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "itemInstanceID")]
            pub item_instance_id: Option<String>,
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
            /// Whether to consider the user's face as found.
            ///
            /// Allows controlling the model when the "tracking lost" animation is played.
            pub face_found: bool,
            /// Whether to set or add the parameter data (default is `set`).
            ///
            /// Generally, if another plugin is already controlling one (default or custom)
            /// parameter, an error will be returned. This happens because only one plugin can
            /// `set` (override) a given parameter at a time, which is the default mode for this
            /// request. This is the mode that is used when you don't provide a value in the `mode`
            /// field or set the value to `set`.
            ///
            /// Alternatively, you can set the `"mode"` field to `"add"`. This will instead add the
            /// values you send to whatever the current parameter values are. The `weight` values
            /// aren't used in that case. Any number of plugins can use the `add` mode for a given
            /// parameter at the same time. This can be useful for bonk/throwing type plugins and
            /// other use-cases.
            pub mode: EnumString<InjectParameterDataMode>,
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
        rust_name = NdiConfig,
        req_name = "NDIConfigRequest",
        resp_name = "NDIConfigResponse",
        /// Get and set NDI settings.
        ///
        /// Note that the boolean fields (`ndi_optional`, `use_ndi5`, etc) are optional in this
        /// library since they're not strictly required by the API, but the API currently treats
        /// them the same as `false` if unspecified.
        req = {
            /// Set to `false` to only return existing config (other fields will be ignored).
            pub set_new_config: bool,
            /// Whether NDI should be active.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub ndi_active: Option<bool>,
            /// Whether NDI 5 should be used.
            #[serde(rename = "useNDI5", skip_serializing_if = "Option::is_none")]
            pub use_ndi5: Option<bool>,
            /// Whether a custom resolution should be used.
            ///
            /// Setting this to `true` means the NDI stream will no longer have
            /// the same resolution as the VTube Studio window, but instead use
            /// the custom resolution set via the UI or the `custom_width`
            /// fields of this request.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub use_custom_resolution: Option<bool>,
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

    {
        rust_name = GetCurrentModelPhysics,
        /// Get the physics settings of the current model.
        req = {},
        /// Data about the requested physics settings.
        resp = {
            /// Whether the model is loaded.
            ///
            /// If no model is loaded, this will be false. All other values do
            /// not have any significance in that case and the `physics_groups`
            /// array will be empty.
            pub model_loaded: bool,
            /// The name of the model.
            pub model_name: String,
            /// The ID of the model.
            #[serde(rename = "modelID")]
            pub model_id: String,
            /// Whether the model has physics.
            ///
            /// If a model is loaded, this field will tell you whether or not
            /// the model has a valid physics setup. Some models don't have
            /// physics set up or have a broken physics file which will cause the
            /// physics system to not start correctly.
            pub model_has_physics: bool,
            /// Whether physics is enabled.
            ///
            /// This will be `true` if the "Use Physics" toggle has been
            /// activated for this model by the user in VTube Studio.
            pub physics_switched_on: bool,
            /// Whether legacy physics is enabled.
            ///
            /// This corresponds to the "Legacy Physics" toggle in the VTube Studio UI.
            pub using_legacy_physics: bool,
            /// The physics FPS setting for this model.
            ///
            /// Can be 30, 60, 120, or -1, which indicates that the user has
            /// selected "Use same FPS as app" in the UI.
            #[serde(rename = "physicsFPSSetting")]
            pub physics_fps_setting: i32,
            /// Base physics strength for this model (between 0 and 100, default 50).
            pub base_strength: i32,
            /// Base wind strength for this model (between 0 and 100, default 0).
            pub base_wind: i32,
            /// Whether a plugin is currently overriding the physics settings of this model.
            pub api_physics_override_active: bool,
            /// The name of the plugin that is currently overriding physics settings, if any.
            pub api_physics_override_plugin_name: String,
            /// Physics groups for this model.
            pub physics_groups: Vec<PhysicsGroup>,
        },
    },

    {
        rust_name = SetCurrentModelPhysics,
        /// Overriding physics settings of currently loaded VTS model.
        ///
        /// If the user has turned off physics for the currently loaded model, you cannot turn
        /// physics on using this API. You can only use this API to override physics/wind base
        /// values and multipliers.
        ///
        /// Generally, the values set using this API are used to override the values set by the
        /// user in the app. They're not actually shown to the user on the UI and are not saved.
        /// Override values set using this API are automatically unset when their timer runs out
        /// (the value you set using `override_seconds`). If you want to keep overriding values,
        /// you have to repeatedly send this request.
        ///
        /// When all timers run out, the API will no longer consider your plugin as controlling the
        /// physics system so another plugin could start controlling the physics system.
        req = {
            /// Strength overrides.
            pub strength_overrides: Vec<PhysicsOverride>,
            /// Wind overrides.
            pub wind_overrides: Vec<PhysicsOverride>,
        },
        /// Empty response on successful physics override.
        resp = {},
    },

    {
        rust_name = ItemList,
        /// Requesting list of available items or items in scene.
        ///
        /// This request lets you request a list of items that are currently in the scene. It also
        /// lets you request a list of item files that are available to be loaded on the user's PC,
        /// including Live2D items, animation folders, etc.
        ///
        /// If you want to know which order-spots are available to load items into right now, set
        /// `"includeAvailableSpots"` to `true`. Otherwise, the `"availableSpots"` array in the
        /// response will be empty.
        ///
        /// If you want to know which items are loaded in the scene right now, set
        /// `"includeItemInstancesInScene"` to `true`. Otherwise, the `"itemInstancesInScene"`
        /// array in the response will be empty.
        ///
        /// If you want to know which item files are available to be loaded, set
        /// `"includeAvailableItemFiles"` to `true`. Otherwise, the `"availableItemFiles"` array in
        /// the response will be empty.
        ///
        /// **IMPORTANT:** This reads the full list of item files from the user's PC. This may lag
        /// the app for a small moment, so do not use this request with
        /// `"includeAvailableItemFiles"` set to `true` often. Only use it if you really need to
        /// refresh the list of available item files. Set it to `false` in any other case.
        ///
        /// ## Filtering for specific items
        ///
        /// If you only want the item lists to contain items with a certain item instance ID or a
        /// certain filename, you can provide them in the `"onlyItemsWithInstanceID"` and
        /// `"onlyItemsWithFileName"` fields respectively.
        ///
        /// There will only ever be at most one item with a certain instance ID in the scene, but
        /// there could be many items with the same filename because you can load many item
        /// instances based on the same item file.
        ///
        /// Please also note that item filenames are unique, meaning there cannot be two item files
        /// with the same filename. They are also case-sensitive, so if you want to refer to one
        /// specific file, make sure to not change the capitalization.
        req = {
            /// Include available spots.
            pub include_available_spots: bool,
            /// Include item instances in scene.
            pub include_item_instances_in_scene: bool,
            /// Include available item files.
            pub include_available_item_files: bool,
            /// Include only items with file name. E.g., `my_item_filename.png`.
            ///
            /// The filename is the name of the item file. This is the name you can use to load an
            /// instance of the item into the scene. For JPG/PNG/GIF items, this is the full
            /// filename (without path) including the file extension (for example "my_item.jpg").
            /// For animation folders, it's the folder name. For Live2D items, it is also the
            /// folder name.
            pub only_items_with_file_name: Option<String>,
            /// Include only items with instance ID. E.g., `IONAL_InstanceIdOfItemInScene`
            #[serde(rename = "onlyItemsWithInstanceID")]
            pub only_items_with_instance_id: Option<String>,
        },
        /// Item data.
        resp = {
            /// Number of items in scene.
            pub items_in_scene_count: i32,
            /// Total items allowed.
            pub total_items_allowed_count: i32,
            /// Whether item loading is allowed.
            ///
            /// May be `false` if the user has certain menus or dialogs open in VTube Studio. This
            /// generally prevents actions such as loading items, using hotkeys and more.
            pub can_load_items_right_now: bool,
            /// Available spots.
            pub available_spots: Vec<i32>,
            /// Item instances in scene.
            pub item_instances_in_scene: Vec<ItemInstanceInScene>,
            /// Available item files.
            pub available_item_files: Vec<AvailableItemFile>,
        },
    },

    {
        rust_name = ItemLoad,
        /// Loading item into the scene.
        req = {
            /// File name. E.g., `some_item_name.jpg`.
            pub file_name: String,
            /// X position.
            pub position_x: f64,
            /// Y position.
            pub position_y: f64,
            /// Item size. Should be between `0` and `1`.
            pub size: f64,
            /// Rotation, in degrees.
            pub rotation: i32,
            /// Fade time, in seconds. Should be between `0` and `2`.
            pub fade_time: f64,
            /// Item order. If the order is taken, VTube Studio will automatically try to find the
            /// next available order, unless `fail_if_order_taken` is `true`.
            pub order: Option<i32>,
            /// Set to `true` to fail with an `ItemOrderAlreadyTaken` error if the desired `order`
            /// is already taken.
            pub fail_if_order_taken: bool,
            /// Smoothing, between `0` and `1`.
            pub smoothing: f64,
            /// Whether the item is censored.
            pub censored: bool,
            /// Whether the item is flipped.
            pub flipped: bool,
            /// Whether the item is locked.
            pub locked: bool,
            /// Unload item when plugin disconnects.
            pub unload_when_plugin_disconnects: bool,
        },
        /// Item loaded successfully.
        resp = {
            /// Instance ID of the loaded item.
            #[serde(rename = "instanceID")]
            pub instance_id: String,
        },
    },
);

#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
/// Known message types for [`EnumString<InjectParameterDataMode>`].
pub enum InjectParameterDataMode {
    #[serde(rename = "set")]
    Set,
    #[serde(rename = "add")]
    Add,
}

impl Default for InjectParameterDataMode {
    fn default() -> Self {
        Self::Set
    }
}

#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
/// Known message types for [`EnumString<ItemType>`].
pub enum ItemType {
    #[serde(rename = "PNG")]
    Png,
    #[serde(rename = "JPG")]
    Jpg,
    #[serde(rename = "GIF")]
    Gif,
    AnimationFolder,
    #[serde(rename = "Live2D")]
    Live2D,
    Unknown,
}

impl Default for ItemType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Used in [`ItemInstancesInSceneResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInstanceInScene {
    /// File name
    pub file_name: String,
    /// Instance ID.
    #[serde(rename = "instanceID")]
    pub instance_id: String,
    /// Order.
    pub order: i32,
    /// Type of the item. E.g., `PNG`, `JPG`, `GIF`, `AnimationFolder` or `Live2D`.
    #[serde(rename = "type")]
    pub type_: EnumString<ItemType>,
    /// Censored.
    pub censored: bool,
    /// Flipped.
    pub flipped: bool,
    /// Locked.
    pub locked: bool,
    /// Smoothing.
    pub smoothing: f64,
    /// Animation frame rate.
    pub framerate: f64,
    /// Animation frame count.
    pub frame_count: i32,
    /// Current frame.
    pub current_frame: i32,
    /// Pinned to model.
    pub pinned_to_model: bool,
    /// Pinned model ID. May be empty if `pinned_to_model` is `false`.
    #[serde(rename = "pinnedModelID")]
    pub pinned_model_id: String,
    /// Pinned art mesh ID. May be empty if `pinned_to_model` is `false`.
    #[serde(rename = "pinnedArtMeshID")]
    pub pinned_art_mesh_id: String,
    /// Group name.
    pub group_name: String,
    /// Scene name.
    pub scene_name: String,
    /// Whether the item is from the Steam workshop.
    pub from_workshop: bool,
}

/// Used in [`ItemInstancesInSceneResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableItemFile {
    /// File name.
    pub file_name: String,
    /// Item type.
    #[serde(rename = "type")]
    pub type_: EnumString<ItemType>,
    /// How many of these items are loaded.
    pub loaded_count: i32,
}

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
    /// Keyboard/mouse key combination that will trigger this hotkey.
    ///
    /// According to the documentation, at the moment this array will always be empty, but may be
    /// reintroduced in a future update.
    pub key_combination: Vec<String>,
    /// On-screen button ID.
    ///
    /// `1` (top) to `8` (bottom), or `-1` if no on-screen button has been set for this hotkey.,
    #[serde(rename = "onScreenButtonID")]
    pub on_screen_button_id: i32,
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
    pub value: f64,
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

/// Used in [`GetCurrentModelPhysicsResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsGroup {
    /// The physics group ID.
    #[serde(rename = "groupID")]
    pub group_id: String,
    /// The physics group name.
    pub group_name: String,
    /// Strength multipler (between 0 and 2, default 1).
    pub strength_multiplier: f64,
    /// Wind multipler (between 0 and 2, default 1).
    pub wind_multiplier: f64,
}

/// Used in [`SetCurrentModelPhysicsRequest`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsOverride {
    /// Group ID of the physics override.
    ///
    /// This is only relevant if `set_base_value` is `false`.
    pub id: String,
    /// The physics override value.
    ///
    /// If `set_base_value` is `true`, this should be an integer between 0 and
    /// 100. If `set_base_value` is `false`, this should be a floating point
    /// value between 0 and 2.
    pub value: f64,
    /// Whether this override should set the base value for the entire model.
    ///
    /// If `true`, sets base value (`id` can be omitted). If `false`, sets
    /// multiplier value for the specific group ID.
    pub set_base_value: bool,
    /// How long the physics should be overridden for.
    ///
    /// Values outside the range of 0.5 and 5 will be clamped.
    pub override_seconds: f64,
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
