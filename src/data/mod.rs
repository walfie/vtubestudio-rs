//! Request/response types for the VTube Studio API.
//!
//! * For a list of all request types, see the implementors for [`Request`].
//! * For the corresponding response types, see [`Response`].
//! * For event types specifically:
//!   * [`EventSubscriptionRequest`] is a [`Request`] used to subscribe to events.
//!   * [`EventData`] is a trait used to correlate [`Event`]s with their corresponding [`EventConfig`]s.
//!   * [`Event`] is an enum of known event types.

mod enumeration;
mod envelope;
mod error_id;

pub use crate::data::enumeration::EnumString;
pub use crate::data::envelope::{
    OpaqueValue, RequestEnvelope, RequestId, ResponseData, ResponseEnvelope, API_NAME, API_VERSION,
};
pub use crate::data::error_id::ErrorId;

use crate::data::enumeration::Enum;
use paste::paste;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::convert::TryFrom;

/// Trait describing a VTube Studio request. Used to set data in [`RequestEnvelope`].
pub trait Request: Serialize {
    /// The message type of this request.
    const MESSAGE_TYPE: EnumString<RequestType>;

    /// The expected [`Response`] type for this request.
    type Response: Response;
}

/// Trait describing a VTube Studio response. Typically parsed from [`ResponseEnvelope`].
pub trait Response: DeserializeOwned + Send + 'static {
    /// The message type of this response.
    const MESSAGE_TYPE: EnumString<ResponseType>;
}

/// Trait describing VTube Studio event data.
///
/// See [`Event`] for an enum of known event types.
pub trait EventData: Response {
    /// The config for this event.
    type Config: EventConfig;
}

/// Trait describing a VTube Studio event's config.
pub trait EventConfig: Serialize {
    /// The corresponding event for this config.
    type Event: EventData;
}

/// Used for serde default. E.g., `#[serde(default = "serde_true")]`
fn serde_true() -> bool {
    true
}

// https://github.com/DenchiSoft/VTubeStudio/blob/08681904e285d37b8c22d17d7d3a36c8c6834425/Files/HotkeyAction.cs
/// Known hotkey types for [`EnumString<HotkeyAction>`]. Used in [`Hotkey`].
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
    /// Toggles the tracking on/off. Can be webcam or USB/WiFi connected phone.
    ToggleTracker,
}

impl Default for HotkeyAction {
    fn default() -> Self {
        Self::Unset
    }
}

/// Known animation event types for [`EnumString<AnimationEventType>`]. Used in [`ModelAnimationEvent`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimationEventType {
    /// Triggered when an animation **starts** (idle-animations or normal one-time animations).
    Start,
    /// Triggered when an animation **ends** (idle-animations or normal one-time animations)
    End,
    /// Triggered when a custom event is encountered in the animation.
    ///
    /// These events can be added at any point in the animation when creating animations in the
    /// Live2D Cubism Animation Editor. You can find more details about how to create/use those
    /// events below.
    Custom,
}

impl Default for AnimationEventType {
    fn default() -> Self {
        Self::Custom
    }
}

/// Known event types for [`EnumString<ItemEventType>`]. Used in [`ItemEvent`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ItemEventType {
    /// Item was added to the scene.
    Added,
    /// Item was removed from the scene.
    Removed,
    /// Item was dropped on the model and is now pinned.
    DroppedPinned,
    /// Item was dropped but not on the model. It is now unpinned.
    DroppedUnpinned,
    /// Item was clicked.
    Clicked,
    /// Item was locked.
    Locked,
    /// Item was unlocked.
    Unlocked,
}

impl Default for ItemEventType {
    fn default() -> Self {
        Self::Clicked
    }
}

/// Known values for [`EnumString<AngleRelativeTo>`]. Used in [`ItemPinRequest`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AngleRelativeTo {
    /// Absolute angle.
    ///
    /// That means if you pass in 0 as angle, the item will be pinned upright at an angle of 0
    /// compared to the VTS window. You'd use this if you want the item to face a certain direction
    /// relative to the VTS window.
    RelativeToWorld,
    /// Relative to the angle the item currently is at.
    ///
    /// If you pass in 0 as angle, that means the item will be pinned at the angle it is already at
    /// meaning its current rotation will not be changed. You'd use this if you don't want to
    /// change the item rotation and just pin it as is.
    RelativeToCurrentItemRotation,
    /// Relative angle to model rotation.
    ///
    /// That means if you pass in 0 as angle and the user has rotated the model, the item will be
    /// pinned upright in relation to the model. This "model rotation" doesn't include rotation
    /// caused by Live2D ArtMesh deformation, only the actual rotation applied to the whole model
    /// by VTube Studio. You'd use this if you want the item to face a certain direction relative
    /// to the current rotation of the model.
    RelativeToModel,
    /// Relative angle to the pin position.
    ///
    /// This is what you should use if you want to pin an item at a certain position within a
    /// certain ArtMesh at a certain angle and you want that angle to be exactly the same no matter
    /// how the model is rotated right now or how the ArtMesh is deformed. However, what angle you
    /// have to pass in to get the desired effect will be completely different for each
    /// pin-position.
    RelativeToPinPosition,
}

impl Default for AngleRelativeTo {
    fn default() -> Self {
        Self::RelativeToWorld
    }
}

/// Known values for [`EnumString<SizeRelativeTo>`]. Used in [`ItemPinRequest`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SizeRelativeTo {
    /// Absolute size. Between 0 (smallest) and 1 (largest).
    ///
    /// See also [`ItemLoadRequest`].
    RelativeToWorld,
    /// Relative to current item size.
    ///
    /// You can pass in numbers between `-1` and `1`, which will be added to the current item size,
    /// meaning you can pass in 0 if you want to pin the item at its current size without changing
    /// it.
    RelativeToCurrentItemSize,
}

impl Default for SizeRelativeTo {
    fn default() -> Self {
        Self::RelativeToWorld
    }
}

/// Known animation event types for [`EnumString<VertexPinType>`]. Used in [`ItemPinRequest`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VertexPinType {
    /// The item will be pinned to the given ArtMesh using the pin position provided in the fields
    /// `vertexID1`, `vertexID2`, `vertexID3`, `vertexWeight1`, `vertexWeight2` and
    /// `vertexWeight3`.
    Provided,
    /// The item will be pinned to the "center" of the given ArtMesh. It's not really the center
    /// (spacially) but actually the triangle in the middle of the triangle list of the mesh. This
    /// will give you the same position every time for a given ArtMesh.
    Center,
    /// The item will be pinned to a random triangle within the given ArtMesh.
    Random,
}

impl Default for VertexPinType {
    fn default() -> Self {
        Self::Provided
    }
}


/// Known animation event types for [`EnumString<Permission>`]. Used in [`PermissionRequest`] and [`PermissionResponse`].
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// Load custom images as items.
    LoadCustomImagesAsItems,
}

impl Default for Permission {
    fn default() -> Self {
        Self::LoadCustomImagesAsItems
    }
}

/// Whether a permission was granted. Used in [`PermissionResponse`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionStatus {
    /// Permission name.
    pub name: EnumString<Permission>,
    /// Whether this permission was granted.
    pub granted: bool,
}

macro_rules! define_request_response {
    (
        req_resp = [
            $({
                rust_name = $rust_name:ident,
                $(req_name = $req_name:literal,)?
                $(resp_name = $resp_name:literal,)?
                $(#[doc = $req_doc:expr])+
                $(#[derive($extra_derives:tt)])?
                req = { $($req:tt)* },
                $(#[doc = $resp_doc:expr])+
                resp = $(( $($resp_inner:tt)+ ))? $({ $($resp_fields:tt)* })?,
            },)*
        ],
        events = [
            $({
                rust_name = $rust_event_name:ident,
                $(event_name = $event_name:literal,)?
                $(#[doc = $event_config_doc:expr])*
                config = { $($event_config_fields:tt)* },
                $(#[doc = $event_data_doc:expr])+
                data = { $($event_data_fields:tt)* },
            },)*
        ],
    ) => {
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

                $(
                    $(#[serde(rename = $event_name)])?
                    [<$rust_event_name Event>],
                )*
            }

            impl ResponseType {
                /// Whether this response type corresponds to an [`Event`].
                pub fn is_event(&self) -> bool {
                    match self {
                        $( Self::[<$rust_event_name Event>] => true, )*
                        _ => false
                    }
                }
            }
        }

        $(
            paste! {
                $(#[doc = $event_data_doc])+
                ///
                #[doc = concat!("This event can be configured using [`", stringify!($rust_event_name), "EventConfig`].")]
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_event_name Event>] { $($event_data_fields)* }

                impl Response for [<$rust_event_name Event>] {
                    #[doc = concat!("[`ResponseType::", stringify!($rust_event_name), "Event`]")]
                    const MESSAGE_TYPE: EnumString<ResponseType> =
                        EnumString::new(ResponseType::[<$rust_event_name Event>]);
                }

                impl EventData for [<$rust_event_name Event>] {
                    #[doc = concat!("[`", stringify!($rust_event_name), "EventConfig`]")]
                    type Config = [<$rust_event_name EventConfig>];
                }
            }
        )*

        paste! {
            #[derive(Debug, Clone, Serialize)]
            #[serde(untagged)]
            #[non_exhaustive]
            #[allow(missing_docs)]
            /// Event types. Events can be requested via [`EventSubscriptionRequest`].
            pub enum Event {
                $( $rust_event_name( [<$rust_event_name Event>] ), )*
                Unknown(ResponseData),
            }

            impl TryFrom<ResponseData> for Event {
                type Error = serde_json::Error;

                fn try_from(data: ResponseData) -> Result<Self, Self::Error> {
                    Ok(match data.message_type.0 {
                        $(
                            Enum::Known(ResponseType::[<$rust_event_name Event>]) =>
                                Event::$rust_event_name(
                                    data.data.deserialize::<[<$rust_event_name Event>]>()?
                                ),
                        )*
                        _ => Event::Unknown(data),
                    })
                }
            }

            $(
                #[doc = concat!("Config for [`", stringify!($rust_event_name), "Event`].")]
                /// Used in [`EventSubscriptionRequest`].
                ///
                $(#[doc = $event_config_doc])*
                #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
                #[serde(rename_all = "camelCase")]
                pub struct [<$rust_event_name EventConfig>] { $($event_config_fields)* }

                impl EventConfig for [<$rust_event_name EventConfig>] {
                    type Event = [<$rust_event_name Event>];
                }
            )*
        }

        $(
            paste! {
                $(#[doc = $req_doc])+
                ///
                #[doc = concat!("This request returns [`", stringify!($rust_name), "Response`].")]
                #[derive(Default, Debug, Clone, Serialize, Deserialize)]
                $(#[derive($extra_derives)])*
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

impl EventSubscriptionRequest {
    /// Subscribe to a specific event type.
    ///
    /// ```
    /// use vtubestudio::data::{EventSubscriptionRequest, TestEventConfig};
    /// let req = EventSubscriptionRequest::subscribe(&TestEventConfig {
    ///     test_message_for_event: "text the event will return".to_owned(),
    /// });
    /// ```
    pub fn subscribe<T>(config: &T) -> Result<Self, serde_json::Error>
    where
        T: EventConfig,
    {
        Ok(Self {
            subscribe: true,
            event_name: Some(T::Event::MESSAGE_TYPE),
            config: Some(OpaqueValue::new(config)?),
        })
    }

    /// Unsubscribe from a specific event type.
    ///
    /// ```
    /// use vtubestudio::data::{EventSubscriptionRequest, TestEvent};
    /// let req = EventSubscriptionRequest::unsubscribe::<TestEvent>();
    /// ```
    pub fn unsubscribe<T>() -> Self
    where
        T: EventData,
    {
        Self {
            subscribe: false,
            event_name: Some(T::MESSAGE_TYPE),
            config: None,
        }
    }

    /// Unsubscribe from all events.
    pub fn unsubscribe_all() -> Self {
        Self {
            subscribe: false,
            event_name: None,
            config: None,
        }
    }
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

define_request_response!(
    req_resp = [{
        rust_name = ApiState,
        req_name = "APIStateRequest",
        resp_name = "APIStateResponse",
        /// Get the current state of the API.
        #[derive(PartialEq)]
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
        rust_name = EventSubscription,
        /// Subscribe or unsubscribe from events.
        ///
        /// For a type-safe way to set the relevant fields, consider using the `subscribe` and
        /// `unsubscribe` constructors instead of setting them manually.
        req = {
            /// Set to `true` to subscribe, `false` to unsubscribe.
            pub subscribe: bool,
            /// The event type.
            pub event_name: Option<EnumString<ResponseType>>,
            /// Config for the event subscription.
            pub config: Option<OpaqueValue>,
        },
        /// Information about subscriptions.
        resp = {
            /// Number of event types that are subscribed to.
            pub subscribed_event_count: i32,
            /// Subscribed event types.
            pub subscribed_events: Vec<EnumString<ResponseType>>,
        },
    },

    {
        rust_name = AuthenticationToken,
        /// Request an authentication token.
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
            pub mode: Option<EnumString<InjectParameterDataMode>>,
        },
        /// Empty response on parameter injection success.
        resp = {},
    },

    {
        rust_name = ExpressionState,
        /// Requesting current expression state list.
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
            pub custom_width_ndi: i32,
            /// The NDI height.
            #[serde(rename = "customHeightNDI")]
            pub custom_height_ndi: i32,
        },
    },

    {
        rust_name = GetCurrentModelPhysics,
        /// Get the physics settings of the current model.
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
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
        #[derive(PartialEq)]
        req = {
            /// File name. E.g., `some_item_name.jpg`.
            pub file_name: String,
            /// X position.
            pub position_x: f64,
            /// Y position.
            pub position_y: f64,
            /// Item size. Should be between `0` and `1`.
            ///
            /// `0.32` is roughly the "default" size that items will have when the user loads them manually.
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
            /// Custom base 64 data to load.
            ///
            /// Must contain valid data of a PNG, JPG or GIF file. Custom data Live2D items are not
            /// currently supported. The dimensions must be between (inclusive) 64 and 2048 pixels
            /// and the data has to be smaller than 5 MB (the binary data before you encode it to
            /// base64). For GIF data, the GIF must contain 1024 frames or less but at least 1
            /// frame and all frames must have the same size.
            pub custom_data_base64: Option<String>,
            /// Set to `true` to ask the user for permission before loading custom data.
            pub custom_data_ask_user_first: bool,
            /// Set to `false` to always ask for confirmation to load custom data.
            #[serde(default = "serde_true")]
            pub custom_data_skip_asking_user_if_whitelisted: bool,
            /// Timeout for asking user permission to load custom data.
            ///
            /// If you set it to 0 (or smaller), the popup will be shown forever (until the user
            /// clicks OK or cancel). If you set it to any other positive number, the popup will be
            /// shown for that number in seconds, with a visible countdown shown to the user. If
            /// the user doesn't make a decision before the countdown is done, the item load
            /// request is considered rejected by the user.
            pub custom_data_ask_timer: f64,
        },
        /// Item loaded successfully.
        resp = {
            /// Instance ID of the loaded item.
            #[serde(rename = "instanceID")]
            pub instance_id: String,
            /// File name of the loaded item.
            ///
            /// If you loaded a custom data item, the filename will be generated by VTube Studio,
            /// so it will be different from the filename that you passed in. You can use that
            /// filename to request that item to be loaded again without passing in custom data.
            /// However, when VTube Studio is restarted, all temporary files are cleared, which
            /// includes custom data items.
            pub file_name: String,
        },
    },

    {
        rust_name = ItemUnload,
        /// Removing item from the scene.
        ///
        /// This may return an error of type `CannotCurrentlyUnloadItem` if the user currently has
        /// menus open that prevent VTS from loading/unloading items.
        #[derive(PartialEq)]
        req = {
            /// Whether to unload all items in the scene.
            pub unload_all_in_scene: bool,
            /// Whether to unload all items loaded by this plugin.
            pub unload_all_loaded_by_this_plugin: bool,
            /// Whether to allow unloading items that have been loaded by the user or other
            /// plugins.
            pub allow_unloading_items_loaded_by_user_or_other_plugins: bool,
            /// Request specific instance IDs to be unloaded.
            #[serde(rename = "instanceIDs")]
            pub instance_ids: Vec<String>,
            /// Request specific file names to be unloaded.
            pub file_names: Vec<String>,
        },
        /// Items unloaded successfully.
        resp = {
            /// List of unloaded items.
            pub unloaded_items: Vec<UnloadedItem>,
        },
    },

    {
        rust_name = ItemAnimationControl,
        /// Controling items and item animations.
        ///
        /// You can control certain aspects of items in the scene. This request allows you to make
        /// items darker (black overlay), change the opacity, and control the animation of animated
        /// items. This request does not work with Live2D items and will return an error of type
        /// `ItemAnimationControlUnsupportedItemType` if you try. This can be useful for "reactive
        /// PNG"-type plugins and more.
        #[derive(PartialEq)]
        req = {
            /// Item instance ID.
            #[serde(rename = "itemInstanceID")]
            pub item_instance_id: String,
            /// Frame rate for animated items, clamped between `0.1` and `120`.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub framerate: Option<f64>,
            /// Jump to a specific frame, zero-indexed.
            ///
            /// May return an error if the frame index is invalid, or if the item type does not
            /// support animation.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub frame: Option<i32>,
            /// Brightness.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub brightness: Option<f64>,
            /// Opacity.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub opacity: Option<f64>,
            /// Whether to set auto-stop frames.
            pub set_auto_stop_frames: bool,
            /// List of frame indices that the animation will automatically stop playing on.
            ///
            /// Once the animation reaches one of those frames, it will stop playing and can only
            /// be started again via the API using this request to set the animation play state to
            /// `true`.
            ///
            /// This only takes effect if `set_auto_stop_frames` is `true`. You can have a maximum
            /// of 1024 auto-stop frames.
            pub auto_stop_frames: Vec<i32>,
            /// Whether to set the animation play state.
            pub set_animation_play_state: bool,
            /// The animation play state (set to `false` to stop the animation).
            ///
            /// This only takes effect if `set_animation_play_state` is `true`.
            pub animation_play_state: bool,
        },
        /// Item animation updated successfully.
        resp = {
            /// Current frame index.
            pub frame: i32,
            /// Whether the animation is playing (only relevant for animated items).
            pub animation_playing: bool,
        },
    },

    {
        rust_name = ItemMove,
        /// Moving items in the scene.
        #[derive(PartialEq)]
        req = {
            /// Items to move. Entries beyond the 64th item will be ignored.
            pub items_to_move: Vec<ItemToMove>,
        },
        /// Item movement requested successfully.
        resp = {
            /// Moved items.
            pub moved_items: Vec<MovedItem>,
        },
    },

    {
        rust_name = ArtMeshSelection,
        /// Asking user to select ArtMeshes.
        ///
        /// You can use this request to show a list in VTube Studio containing all ArtMeshes of the
        /// currently loaded main Live2D model and have the user select one or more of them. Once
        /// the user is done selecting ArtMeshes, the ArtMesh IDs will be returned. You can use
        /// those ArtMesh IDs in various other API requests, for example to apply a color tint to
        /// them or make them invisible.
        ///
        /// If no model is currently loaded or there are currently other windows open, the request
        /// will return an error.
        ///
        /// The user can hover over ArtMeshes to show their ID and click them to filter the shown
        /// list for all ArtMeshes under on the click position.
        #[derive(PartialEq)]
        req = {
            /// This text is shown over the ArtMesh selection list.
            ///
            /// Must be between 4 and 1024 characters long, otherwise the default will be used.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub text_override: Option<String>,
            /// This text is shown when the user presses the `?` button.
            ///
            /// Must be between 4 and 1024 characters long, otherwise the default will be used.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub help_override: Option<String>,
            /// How many art meshes must be selected by the user.
            ///
            /// The "OK" button will be unavailable until exactly this many ArtMeshes are
            /// activated. If you set this to 0 or lower, the user will be asked to choose any
            /// arbitrary number of ArtMeshes (but at least one).
            pub requested_art_mesh_count: i32,
            /// List of ArtMeshes to be pre-selected.
            ///
            /// If any of these IDs are not contained in the current model, an error will be
            /// returned.
            #[serde(skip_serializing_if = "Vec::is_empty")]
            pub active_art_meshes: Vec<String>,
        },
        /// ArtMesh selection response.
        resp = {
            /// This will be `true` if the user clicked "OK", and `false` if the user clicked
            /// "Cancel".
            pub success: bool,
            /// ArtMeshes that were selected.
            pub active_art_meshes: Vec<String>,
            /// ArtMeshes that were not selected.
            pub inactive_art_meshes: Vec<String>,
        },
    },

    {
        rust_name = ItemPin,
        /// Pin items in the scene to the currently loaded model.
        #[derive(PartialEq)]
        req = {
            /// Set to `false` to unpin the item.
            pub pin: bool,
            /// Item instance ID.
            #[serde(rename = "itemInstanceID")]
            pub item_instance_id: String,
            /// How to interpret angles.
            pub angle_relative_to: EnumString<AngleRelativeTo>,
            /// How to interpret sizes.
            pub size_relative_to: EnumString<SizeRelativeTo>,
            /// Vertex pin type.
            pub vertex_pin_type: EnumString<VertexPinType>,
            /// Pin info.
            pub pin_info: ArtMeshHitInfo,
        },
        /// Item pinned successfully.
        resp = {
            /// Whether item is pinned.
            pub is_pinned: bool,
            /// Item instance ID. E.g., `"4a241269394f463ca16b8b21aa636568"`.
            #[serde(rename = "itemInstanceID")]
            pub item_instance_id: String,
            /// Item file name. E.g., `"my_test_item_2.png"`.
            pub item_file_name: String,
        },
    },

    {
        rust_name = Permission,
        /// Request permission or request a list of all already granted permissions.
        #[derive(PartialEq)]
        req = {
            /// Set to `None` to receive the list of currently granted/available permissions for
            /// this plugin without actually requesting a permission.
            pub requested_permission: Option<EnumString<Permission>>,
        },
        /// Permission response.
        resp = {
            /// Whether a permission was granted.
            pub grant_success: bool,
            /// Permission that was requested.
            pub requested_permission: Option<EnumString<Permission>>,
            /// List of permissions.
            pub permissions: Vec<PermissionStatus>,
        },
    },],

    events = [
        {
            rust_name = Test,
            config = {
                /// Message to be returned every second.
                pub test_message_for_event: String,
            },
            /// An event for testing the event API.
            data = {
                /// Test message specified in [`TestEventConfig`].
                pub your_test_message: String,
                /// Number of seconds since VTube Studio has been started.
                pub counter: i32,
            },
        },

        {
            rust_name = ModelLoaded,
            /// You can pass in one or more model IDs in the `modelID` array (optional). If you do,
            /// an event will only be sent every time the model with one of the specific IDs is
            /// loaded or unloaded. If you pass in model IDs, they all have to have the correct
            /// format (32 characters, only hex characters), otherwise an error is returned.
            config = {
                /// Optional model IDs to filter for.
                #[serde(rename = "modelID", skip_serializing_if = "Vec::is_empty")]
                pub model_id: Vec<String>
            },
            /// An event that is triggered every time a VTube Studio model is loaded or unloaded.
            data = {
                /// Whether the model is loaded.
                pub model_loaded: bool,
                /// Name of the model.
                pub model_name: String,
                /// Model ID.
                ///
                /// E.g., `165131471d8a4e42aae01a9738f255ef`.
                #[serde(rename = "modelID")]
                pub model_id: String,
            },
        },

        {
            rust_name = TrackingStatusChanged,
            config = {},
            /// An event that is triggered every time the face tracker finds/loses the face or
            /// hands.
            data = {
                /// Whether the face is found.
                pub face_found: bool,
                /// Whether the left hand is found.
                pub left_hand_found: bool,
                /// Whether the right hand is found.
                pub right_hand_found: bool,
            },
        },

        {
            rust_name = BackgroundChanged,
            config = {},
            /// An event that is triggered every time the background is changed by the user
            /// (manually or via hotkey).
            data = {
                /// Background name, as shown in the background selection list.
                ///
                /// This is typically the file name without the file extension.
                pub background_name: String,
            },
        },

        {
            rust_name = ModelConfigChanged,
            config = {},
            /// An event that is triggered every time the user manually changes the the
            /// settings/config of the currently loaded VTube Studio model.
            data = {
                /// Model ID.
                #[serde(rename = "modelID")]
                pub model_id: String,
                /// Model name.
                pub model_name: String,
                /// Whether the changed config is related to hotkeys.
                pub hotkey_config_changed: bool,
            },
        },

        {
            rust_name = ModelMoved,
            config = {},
            /// An event that is triggered every time a model is moved, resized or rotated.
            ///
            /// This will also be triggered right after subscribing, so it will send you the
            /// current position/scale/rotation of the currently loaded model the moment you
            /// subscribe (unless no model is loaded) and then send the current
            /// position/scale/rotation in every frame when there's a change.
            data = {
                /// Model ID.
                #[serde(rename = "modelID")]
                pub model_id: String,
                /// Model name.
                pub model_name: String,
                /// Model position.
                pub model_position: ModelPosition,
            },
        },

        {
            rust_name = ModelOutline,
            config = {
                /// Whether to draw the model outline.
                ///
                /// Drawing the outline in VTube Studio may not be performant so you should only do
                /// that for debugging purposes or to show the outline to the user for setup
                /// purposes. The outline will be drawn if at least one subscribed plugin has set
                /// "draw" to true.
                pub draw: bool,
            },
            /// An event that is triggered at a constant 15 FPS and sends subscribed plugins the
            /// model outline.
            ///
            /// Specifically, it sends an approximated convex polygon based on the bounding-box
            /// center points of all (visible) ArtMeshes in the model. If no model is loaded, the
            /// event isn't sent.
            data = {
                /// Model name. E.g., `"My VTS Model Name"`.
                pub model_name: String,
                /// Model ID. E.g., `"165131471d8a4e42aae01a9738f255ef"`.
                #[serde(rename = "modelID")]
                pub model_id: String,
                /// 2D points describing the rough outline of the model.
                ///
                /// This list is ordered. The x/y coordinate of each point is its position within
                /// the VTube Studio window. X or Y coordinates may be bigger than 1 or smaller
                /// than -1 if an outline point is outside of the window boundaries.
                ///
                /// The `convex_hull` list is guaranteed to have at least 3 entries. There is no
                /// limit to how many entries it can have, but for most normal Live2D models, it
                /// has between 5 and 25 entries. Please also keep in mind that the number of list
                /// entries can (and will most likely) change between events you receive. Do not
                /// implement your plugin expecting the number of outline points to remain
                /// constant.
                pub convex_hull: Vec<Vec2>,
                /// The center/average of all the `convex_hull` points.
                pub convex_hull_center: Vec2,
                /// The current VTube Studio window size.
                pub window_size: Vec2,
            },
        },

        {
            rust_name = HotkeyTriggered,
            config = {
                /// If provided, only hotkeys of the provided type (with the given action) will
                /// trigger this event. Otherwise, all hotkey activations will trigger the event.
                pub only_for_action: Option<EnumString<HotkeyAction>>,
                /// Set to `true` to avoid receiving events when a hotkey is triggered by a plugin
                /// via the trigger-hotkey-API
                #[serde(rename = "ignoreHotkeysTriggeredByAPI")]
                pub ignore_hotkeys_triggered_by_api: bool,
            },
            /// An event that is triggered every time a hotkey is triggered manually by the user
            /// (keyboard/hand-gesture) or via the hotkey-trigger-API.
            data = {
                /// Hotkey ID. E.g., `"21bf7ade9e664f3ec29d05156e4ce5c1"`.
                #[serde(rename = "hotkeyID")]
                pub hotkey_id: String,
                /// Hotkey name. E.g., `"Eyes Cry"`.
                pub hotkey_name: String,
                /// Hotkey action. E.g., `"ToggleExpression"`.
                pub hotkey_action: EnumString<HotkeyAction>,
                /// Hotkey file. E.g., `"EyesCry.exp3.json"`.
                pub hotkey_file: String,
                /// Whether the hotkey was triggered by the API.
                #[serde(rename = "hotkeyTriggeredByAPI")]
                pub hotkey_triggered_by_api: bool,
                /// Model ID. E.g., `"d8ee771d2909873b1aa0226d03ef4f51"`.
                #[serde(rename = "modelID")]
                pub model_id: String,
                /// Model name. E.g., `"Akari"`.
                pub model_name: String,
                /// Whether the hotkey is for a Live2D item.
                #[serde(rename = "isLive2DItem")]
                pub is_live2d_item: bool,
            },
        },

        {
            rust_name = ModelAnimation,
            config = {
                /// Ignore events triggered by Live2D items.
                #[serde(rename = "ignoreLive2DItems")]
                pub ignore_live2d_items: bool,
                /// Ignore events triggered by idle animations.
                pub ignore_idle_animations: bool,
            },
            /// An event that is triggered every time there's an `animation-event` encountered in
            /// an animation playing for any Live2D model in the scene (so for the main model and
            /// any Live2D items).
            data = {
                /// Animation event type.
                pub animation_event_type: EnumString<AnimationEventType>,
                /// Time in seconds within the animation when the animation-event was encountered.
                pub animation_event_time: f64,
                /// Animation event data. E.g., `"My Test Event 123456789"`.
                pub animation_event_data: String,
                /// Animation name. E.g., `"event_test_anim_4_final.motion3.json"`.
                pub animation_name: String,
                /// Length of animation in seconds.
                pub animation_length: f64,
                /// Whether the animation is an idle animation.
                pub is_idle_animation: bool,
                /// Model ID. E.g., `"d8ee771d2909873b1aa0226d03ef4f51"`.
                #[serde(rename = "modelID")]
                pub model_id: String,
                /// Model name. E.g., `"Akari"`.
                pub model_name: String,
                /// Whether the event is for a Live2D item.
                #[serde(rename = "isLive2DItem")]
                pub is_live2d_item: bool,
            },
        },

        {
            rust_name = Item,
            config = {
                /// Item instance IDs to match on. Set to empty to match all IDs.
                #[serde(rename = "itemInstanceIDs")]
                pub item_instance_ids: Vec<String>,
                /// Item file names to match on. Set to empty to match all file names.
                ///
                /// This does "contains-matching", so for example if you pass in `"my"`, it will match the item `"my_item.png"`.
                pub item_file_names: Vec<String>,
            },
            /// An event that is triggered every time certain actions are done with/by an item.
            data = {
                /// Item event type.
                pub item_event_type: EnumString<ItemEventType>,
                /// Item instance ID. E.g., `"3dcfc2456ac94a37bad369ec1875a15b"`.
                #[serde(rename = "itemInstanceID")]
                pub item_instance_id: String,
                /// Item file name. E.g., `"my_item.png"`
                pub item_file_name: String,
                /// Item position.
                pub item_position: Vec2,
            },
        },

        {
            rust_name = ModelClicked,
            config = {
                /// Set to `true` to only include clicks on model.
                pub only_clicks_on_model: bool,
            },
            /// An event that is triggered every time the model is clicked.
            ///
            /// Depending on the config, it is also triggered when you click anywhere in the VTS
            /// window even if the click wasn't on the model.
            data = {
                /// Whether the model is currently loaded.
                pub model_loaded: bool,
                /// Model ID. E.g., `"d8ee771d2909873b1aa0226d03ef4f51"`.
                #[serde(rename = "loadedModelID")]
                pub loaded_model_id: String,
                /// Model name. E.g., `"Akari"`.
                pub loaded_model_name: String,
                /// Whether model was clicked.
                pub model_was_clicked: bool,
                /// ID of the mouse button. 0 for left click, 1 for right click, 2 for middle click.
                // TODO: Turn this into an enum?
                #[serde(rename = "mouseButtonID")]
                pub mouse_button_id: i32,
                /// The position of the click in the usual coordinate system.
                ///
                /// If you need the exact pixel position of the click, you can use `windowSize`
                /// (current VTS window size in pixels) to calculate that.
                pub click_position: Vec2,
                /// Current VTS window size in pixels.
                pub window_size: Vec2,
                /// The number of ArtMeshes at the click position.
                pub clicked_art_mesh_count: i32,
                /// ArtMesh hits.
                pub art_mesh_hits: Vec<ArtMeshHit>,
            },
        },

        /*
        // Disabled for now since PostProcessingListRequest/etc are only available in beta.
        {
            rust_name = PostProcessing,
            config = {},
            /// An event that is triggered every time the post-processing system is turned on/off or a preset is loaded/unloaded.
            ///
            /// For more information about post-processing, check the [visual effects page](https://github.com/DenchiSoft/VTubeStudio/wiki/Visual-Effects).
            ///
            /// If after receiving this event you want to read the detailed post-processing state
            /// including all configs and their values use the [`PostProcessingListRequest`] and if
            /// you want to send post-processing values, use the [`PostProcessingUpdateRequest`].
            data = {
                /// Current on state.
                pub current_on_state: bool,
                /// Current preset. E.g., `"my_preset"`.
                pub current_preset: String,
            },
        },
        */

        {
            rust_name = Live2DCubismEditorConnected,
            config = {},
            /// This event is triggered every time the `Connect`/`Send parameters` toggles are
            /// toggled by the user and every time the connection state with the Live2D Cubism API
            /// changes.
            ///
            /// Additionally, the event is also sent exactly once when you first subscribe to the
            /// event. That way, you can easily get the initial state.
            data = {
                /// Is VTube Studio trying to connect to Live2D Cubism? `true` if the `Connect` toggle is turned on in VTS.
                pub trying_to_connect: bool,
                /// Is VTube Studio fully connected to and authenticated with Live2D Cubism?
                pub connected: bool,
                /// Has the user turned on the `Send parameters` toggle?
                ///
                /// If this is on and `connected` is true, VTube Studio is actively sending parameter data into Live2D Cubism.
                pub should_send_parameters: bool,
            },
        },
    ],
);

/// Art mesh hit. Used in [`ModelClickedEvent`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtMeshHit {
    /// The order in the ArtMesh stack at the click position.
    ///
    /// The topmost ArtMesh at the click position will have `"artMeshOrder": 0`. This would be the
    /// ArtMesh that items pin to when dropped by users at that exact position.
    pub art_mesh_order: i32,
    /// Whether the ArtMesh is masked.
    pub is_masked: bool,
    /// Info about the art mesh.
    pub hit_info: ArtMeshHitInfo,
}

/// ArtMesh hit info, used in [`ArtMeshHit`] and [`ItemPinRequest`].
///
/// The exact click position within the ArtMesh is given via the `vertexID1`, `vertexID2`,
/// `vertexID3`, `vertexWeight1`, `vertexWeight2`, `vertexWeight3` fields as [barycentric
/// coordinates]. In short, VTS checks which specific triangle was clicked in the ArtMesh and
/// returns the three vertex IDs that make up this triangle, including the weights that if
/// multiplied with the vertex positions will result in the exact click position within the
/// triangle.
///
/// The ArtMesh ID and barycentric coordinates can be used to identify an exact position on the
/// model and can be used to pin an item at that position using the `ItemPinRequest`.
///
/// [barycentric coordinates]: https://en.wikipedia.org/wiki/Barycentric_coordinate_system
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtMeshHitInfo {
    /// Model ID. E.g., `"d87b771d2902473bbaa0226d03ef4754"`.
    #[serde(rename = "modelID")]
    pub model_id: String,
    /// ArtMesh ID. E.g., `"hair_right6"`.
    #[serde(rename = "artMeshID")]
    pub art_mesh_id: String,
    /// Angle.
    pub angle: f64,
    /// Size.
    pub size: f64,
    /// Vertex ID 1.
    #[serde(rename = "vertexID1")]
    pub vertex_id1: i32,
    /// Vertex ID 2.
    #[serde(rename = "vertexID2")]
    pub vertex_id2: i32,
    /// Vertex ID 3.
    #[serde(rename = "vertexID3")]
    pub vertex_id3: i32,
    /// Vertex weight 1.
    pub vertex_weight1: f64,
    /// Vertex weight 2.
    pub vertex_weight2: f64,
    /// Vertex weight 3.
    pub vertex_weight3: f64,
}

/// Struct representing a coordinate or dimensions. Used in [`ModelOutlineEvent`].
#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Vec2 {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
}

#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
/// Known message types for [`EnumString<InjectParameterDataMode>`] (used in
/// [`InjectParameterDataRequest`]).
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
/// Known message types for [`EnumString<ItemType>`]. Used in [`ItemInstanceInScene`].
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

/// Used in [`ItemUnloadResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnloadedItem {
    /// Instance ID.
    #[serde(rename = "instanceID")]
    pub instance_id: String,
    /// File name.
    pub file_name: String,
}

/// Used in [`ItemListResponse`].
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

/// Used in [`ItemListResponse`].
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

/// Used in [`ItemMoveRequest`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemToMove {
    /// Item instance ID.
    #[serde(rename = "itemInstanceID")]
    pub item_instance_id: String,
    /// How long it takes to move the item, clamped between `0` and `30` seconds.
    pub time_in_seconds: f64,
    /// Fade mode, used if `time_in_seconds` is non-zero.
    pub fade_mode: EnumString<FadeMode>,
    /// X position.
    ///
    /// A value of `None` will be serialized as `-1000` as recommended by the documentation.
    #[serde(serialize_with = "item_move_default_i32")]
    pub position_x: Option<i32>,
    /// Y position.
    ///
    /// A value of `None` will be serialized as `-1000` as recommended by the documentation.
    #[serde(serialize_with = "item_move_default_i32")]
    pub position_y: Option<i32>,
    /// Size.
    ///
    /// A value of `None` will be serialized as `-1000` as recommended by the documentation.
    #[serde(serialize_with = "item_move_default_f64")]
    pub size: Option<f64>,
    /// Rotation, in degrees.
    ///
    /// A value of `None` will be serialized as `-1000` as recommended by the documentation.
    #[serde(serialize_with = "item_move_default_i32")]
    pub rotation: Option<i32>,
    /// Change the order of the item.
    ///
    /// A value of `None` will be serialized as `-1000` as recommended by the documentation.
    #[serde(serialize_with = "item_move_default_i32")]
    pub order: Option<i32>,
    /// Whether to set flip.
    pub set_flip: bool,
    /// Flip.
    pub flip: bool,
    /// Whether the user can stop the item movement by clicking/dragging it.
    pub user_can_stop: bool,
}

/// Used in [`ItemMoveResponse`].
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovedItem {
    /// Item instance ID.
    #[serde(rename = "itemInstanceID")]
    pub item_instance_id: String,
    /// Whether the item move was successful.
    pub success: bool,
    /// The error, if any. `None` means `-1` was returned from the API.
    #[serde(
        rename = "errorID",
        serialize_with = "moved_item_error_serialize",
        deserialize_with = "moved_item_error_deserialize"
    )]
    pub error_id: Option<ErrorId>,
}

fn moved_item_error_deserialize<'de, D>(deserializer: D) -> Result<Option<ErrorId>, D::Error>
where
    D: Deserializer<'de>,
{
    let id = i32::deserialize(deserializer)?;
    if id == -1 {
        Ok(None)
    } else {
        Ok(Some(ErrorId::new(id)))
    }
}

fn moved_item_error_serialize<S>(value: &Option<ErrorId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(match value {
        Some(v) => v.as_i32(),
        None => -1,
    })
}

// Per the docs, we should send `-1000` if the user doesn't want to change the item order.
fn item_move_default_i32<S>(value: &Option<i32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i32(value.unwrap_or(-1000))
}

// Per the docs, we should send `-1000` if the user doesn't want to change the item order.
fn item_move_default_f64<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(value.unwrap_or(-1000.0f64))
}

#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// Known message types for [`EnumString<FadeMode>`]. Used in [`ItemToMove`].
pub enum FadeMode {
    Linear,
    EaseIn,
    EaseOut,
    EaseBoth,
    Overshoot,
    Zip,
}

impl Default for FadeMode {
    fn default() -> Self {
        Self::Linear
    }
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

/// Used in [`CurrentModelResponse`] and [`ModelMovedEvent`].
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

    #[test]
    fn serialize_event_request() -> Result {
        let req = RequestEnvelope::new(&EventSubscriptionRequest::subscribe(&TestEventConfig {
            test_message_for_event: "text the event will return".to_owned(),
        })?)?
        .with_id(Some("SomeID".into()));

        // https://github.com/DenchiSoft/VTubeStudio/tree/5e45a961/Events#test-event
        let expected = json!({
            "apiName": "VTubeStudioPublicAPI",
            "apiVersion": "1.0",
            "requestID": "SomeID",
            "messageType": "EventSubscriptionRequest",
            "data": {
                "eventName": "TestEvent",
                "subscribe": true,
                "config": {
                    "testMessageForEvent": "text the event will return"
                }
            }
        });

        assert_eq!(
            serde_json::from_value::<RequestEnvelope>(expected.clone())?,
            req
        );
        assert_eq!(serde_json::to_value(&req)?, expected);

        Ok(())
    }

    #[test]
    fn parse_response_as_event() -> Result {
        // https://github.com/DenchiSoft/VTubeStudio/tree/5e45a961/Events#test-event
        let json = json!({
            "apiName": "VTubeStudioPublicAPI",
            "apiVersion": "1.0",
            "timestamp": 1625405710728i64,
            "requestID": "SomeID",
            "messageType": "TestEvent",
            "data": {
                "yourTestMessage": "text the event will return",
                "counter": 672
            }
        });

        let resp = serde_json::from_value::<ResponseEnvelope>(json)?;
        let parsed = resp.parse_event()?;

        let expected = TestEvent {
            your_test_message: "text the event will return".to_owned(),
            counter: 672,
        };

        assert!(matches!(parsed, Event::Test(event) if event == expected));

        Ok(())
    }
}
