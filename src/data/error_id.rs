use serde::{Deserialize, Serialize};
use std::fmt;

/// Error ID returned in [`ApiError`](crate::data::ApiError) responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ErrorId(i32);

impl ErrorId {
    /// Creates a new error ID.
    pub fn new(value: i32) -> Self {
        Self(value)
    }

    /// Returns the numeric ID of the error.
    pub fn as_i32(&self) -> i32 {
        self.0
    }

    /// Returns true if this is an [`ErrorId::REQUEST_REQUIRES_AUTHENTICATION`] error.
    pub fn is_unauthenticated(&self) -> bool {
        self == Self::REQUEST_REQUIRES_AUTHENTICATION
    }
}

impl From<i32> for ErrorId {
    fn from(id: i32) -> Self {
        Self(id)
    }
}

impl From<ErrorId> for i32 {
    fn from(id: ErrorId) -> Self {
        id.0
    }
}

/// Formats the error ID, including its name.
///
/// # Example
///
/// ```
/// # use vtubestudio::error::ErrorId;
/// assert_eq!(
///     format!("{}", ErrorId::REQUEST_REQUIRES_AUTHENTICATION),
///     "8 (RequestRequiresAuthentication)"
/// )
/// ```
impl fmt::Display for ErrorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{} ({})", self.as_i32(), name)
        } else {
            write!(f, "{}", self.as_i32())
        }
    }
}

impl PartialEq<&ErrorId> for ErrorId {
    fn eq(&self, rhs: &&ErrorId) -> bool {
        self == *rhs
    }
}

impl PartialEq<ErrorId> for &ErrorId {
    fn eq(&self, rhs: &ErrorId) -> bool {
        *self == rhs
    }
}

macro_rules! define_error_ids {
    (
        $(
            $(#[$docs:meta])*
            ($id:expr, $rust_name:ident, $cs_name:ident),
        )+
    ) => {
        impl ErrorId {
            /// Returns a descriptive name for the error.
            pub fn name(&self) -> Option<&'static str> {
                match self.0 {
                    $(
                        $id => Some(stringify!($cs_name)),
                    )+
                    _ => None,
                }
            }

            $(
                $(#[$docs])*
                #[doc = concat!("`", stringify!($id), " ", stringify!($cs_name), "`")]
                pub const $rust_name: ErrorId = ErrorId($id);
            )+

        }
    }
}

// https://github.com/DenchiSoft/VTubeStudio/blob/cbf875f9d16f868d57e9921235a129cf2f756a4d/Files/ErrorID.cs
define_error_ids! {
    // General errors
    (0, INTERNAL_SERVER_ERROR, InternalServerError),
    (1, API_ACCESS_DEACTIVATED, APIAccessDeactivated),
    (2, JSON_INVALID, JSONInvalid),
    (3, API_NAME_INVALID, APINameInvalid),
    (4, API_VERSION_INVALID, APIVersionInvalid),
    (5, REQUEST_ID_INVALID, RequestIDInvalid),
    (6, REQUEST_TYPE_MISSING_OR_EMPTY, RequestTypeMissingOrEmpty),
    (7, REQUEST_TYPE_UNKNOWN, RequestTypeUnknown),
    (8, REQUEST_REQUIRES_AUTHENTICATION, RequestRequiresAuthentication),
    (9, REQUEST_REQUIRES_PERMISSION, RequestRequiresPermission),

    // Errors related to AuthenticationTokenRequest
    (50, TOKEN_REQUEST_DENIED, TokenRequestDenied),
    (51, TOKEN_REQUEST_CURRENTLY_ONGOING, TokenRequestCurrentlyOngoing),
    (52, TOKEN_REQUEST_PLUGIN_NAME_INVALID, TokenRequestPluginNameInvalid),
    (53, TOKEN_REQUEST_DEVELOPER_NAME_INVALID, TokenRequestDeveloperNameInvalid),
    (54, TOKEN_REQUEST_PLUGIN_ICON_INVALID, TokenRequestPluginIconInvalid),

    // Errors related to AuthenticationRequest
    (100, AUTHENTICATION_TOKEN_MISSING, AuthenticationTokenMissing),
    (101, AUTHENTICATION_PLUGIN_NAME_MISSING, AuthenticationPluginNameMissing),
    (102, AUTHENTICATION_PLUGIN_DEVELOPER_MISSING, AuthenticationPluginDeveloperMissing),

    // Errors related to ModelLoadRequest
    (150, MODEL_ID_MISSING, ModelIDMissing),
    (151, MODEL_ID_INVALID, ModelIDInvalid),
    (152, MODEL_ID_NOT_FOUND, ModelIDNotFound),
    (153, MODEL_LOAD_COOLDOWN_NOT_OVER, ModelLoadCooldownNotOver),
    (154, CANNOT_CURRENTLY_CHANGE_MODEL, CannotCurrentlyChangeModel),

    // Errors related to HotkeyTriggerRequest
    (200, HOTKEY_QUEUE_FULL, HotkeyQueueFull),
    (201, HOTKEY_EXECUTION_FAILED_BECAUSE_NO_MODEL_LOADED, HotkeyExecutionFailedBecauseNoModelLoaded),
    (202, HOTKEY_ID_NOT_FOUND_IN_MODEL, HotkeyIDNotFoundInModel),
    (203, HOTKEY_COOLDOWN_NOT_OVER, HotkeyCooldownNotOver),
    (204, HOTKEY_ID_FOUND_BUT_HOTKEY_DATA_INVALID, HotkeyIDFoundButHotkeyDataInvalid),
    (205, HOTKEY_EXECUTION_FAILED_BECAUSE_BAD_STATE, HotkeyExecutionFailedBecauseBadState),
    (206, HOTKEY_UNKNOWN_EXECUTION_FAILURE, HotkeyUnknownExecutionFailure),
    (207, HOTKEY_EXECUTION_FAILED_BECAUSE_LIVE2D_ITEM_NOT_FOUND, HotkeyExecutionFailedBecauseLive2DItemNotFound),
    (208, HOTKEY_EXECUTION_FAILED_BECAUSE_LIVE2D_ITEMS_DO_NOT_SUPPORT_THIS_HOTKEY_TYPE, HotkeyExecutionFailedBecauseLive2DItemsDoNotSupportThisHotkeyType),

    // Errors related to ColorTintRequest
    (250, COLOR_TINT_REQUEST_NO_MODEL_LOADED, ColorTintRequestNoModelLoaded),
    (251, COLOR_TINT_REQUEST_MATCH_OR_COLOR_MISSING, ColorTintRequestMatchOrColorMissing),
    (252, COLOR_TINT_REQUEST_INVALID_COLOR_VALUE, ColorTintRequestInvalidColorValue),

    // Errors related to MoveModelRequest
    (300, MOVE_MODEL_REQUEST_NO_MODEL_LOADED, MoveModelRequestNoModelLoaded),
    (301, MOVE_MODEL_REQUEST_MISSING_FIELDS, MoveModelRequestMissingFields),
    (302, MOVE_MODEL_REQUEST_VALUES_OUT_OF_RANGE, MoveModelRequestValuesOutOfRange),

    // Errors related to ParameterCreationRequest
    (350, CUSTOM_PARAM_NAME_INVALID, CustomParamNameInvalid),
    (351, CUSTOM_PARAM_VALUES_INVALID, CustomParamValuesInvalid),
    (352, CUSTOM_PARAM_ALREADY_CREATED_BY_OTHER_PLUGIN, CustomParamAlreadyCreatedByOtherPlugin),
    (353, CUSTOM_PARAM_EXPLANATION_TOO_LONG, CustomParamExplanationTooLong),
    (354, CUSTOM_PARAM_DEFAULT_PARAM_NAME_NOT_ALLOWED, CustomParamDefaultParamNameNotAllowed),
    (355, CUSTOM_PARAM_LIMIT_PER_PLUGIN_EXCEEDED, CustomParamLimitPerPluginExceeded),
    (356, CUSTOM_PARAM_LIMIT_TOTAL_EXCEEDED, CustomParamLimitTotalExceeded),

    // Errors related to ParameterDeletionRequest
    (400, CUSTOM_PARAM_DELETION_NAME_INVALID, CustomParamDeletionNameInvalid),
    (401, CUSTOM_PARAM_DELETION_NOT_FOUND, CustomParamDeletionNotFound),
    (402, CUSTOM_PARAM_DELETION_CREATED_BY_OTHER_PLUGIN, CustomParamDeletionCreatedByOtherPlugin),
    (403, CUSTOM_PARAM_DELETION_CANNOT_DELETE_DEFAULT_PARAM, CustomParamDeletionCannotDeleteDefaultParam),

    // Errors related to InjectParameterDataRequest
    (450, INJECT_DATA_NO_DATA_PROVIDED, InjectDataNoDataProvided),
    (451, INJECT_DATA_VALUE_INVALID, InjectDataValueInvalid),
    (452, INJECT_DATA_WEIGHT_INVALID, InjectDataWeightInvalid),
    (453, INJECT_DATA_PARAM_NAME_NOT_FOUND, InjectDataParamNameNotFound),
    (454, INJECT_DATA_PARAM_CONTROLLED_BY_OTHER_PLUGIN, InjectDataParamControlledByOtherPlugin),
    (455, INJECT_DATA_MODE_UNKNOWN, InjectDataModeUnknown),

    // Errors related to ParameterValueRequest
    (500, PARAMETER_VALUE_REQUEST_PARAMETER_NOT_FOUND, ParameterValueRequestParameterNotFound),

    // Errors related to NDIConfigRequest
    (550, NDI_CONFIG_COOLDOWN_NOT_OVER, NDIConfigCooldownNotOver),
    (551, NDI_CONFIG_RESOLUTION_INVALID, NDIConfigResolutionInvalid),

    // Errors related to ExpressionStateRequest
    (600, EXPRESSION_STATE_REQUEST_INVALID_FILENAME, ExpressionStateRequestInvalidFilename),
    (601, EXPRESSION_STATE_REQUEST_FILE_NOT_FOUND, ExpressionStateRequestFileNotFound),

    // Errors related to ExpressionActivationRequest
    (650, EXPRESSION_ACTIVATION_REQUEST_INVALID_FILENAME, ExpressionActivationRequestInvalidFilename),
    (651, EXPRESSION_ACTIVATION_REQUEST_FILE_NOT_FOUND, ExpressionActivationRequestFileNotFound),
    (652, EXPRESSION_ACTIVATION_REQUEST_NO_MODEL_LOADED, ExpressionActivationRequestNoModelLoaded),

    // Errors related to SetCurrentModelPhysicsRequest
    (700, SET_CURRENT_MODEL_PHYSICS_REQUEST_NO_MODEL_LOADED, SetCurrentModelPhysicsRequestNoModelLoaded),
    (701, SET_CURRENT_MODEL_PHYSICS_REQUEST_MODEL_HAS_NO_PHYSICS, SetCurrentModelPhysicsRequestModelHasNoPhysics),
    (702, SET_CURRENT_MODEL_PHYSICS_REQUEST_PHYSICS_CONTROLLED_BY_OTHER_PLUGIN, SetCurrentModelPhysicsRequestPhysicsControlledByOtherPlugin),
    (703, SET_CURRENT_MODEL_PHYSICS_REQUEST_NO_OVERRIDES_PROVIDED, SetCurrentModelPhysicsRequestNoOverridesProvided),
    (704, SET_CURRENT_MODEL_PHYSICS_REQUEST_PHYSICS_GROUP_ID_NOT_FOUND, SetCurrentModelPhysicsRequestPhysicsGroupIDNotFound),
    (705, SET_CURRENT_MODEL_PHYSICS_REQUEST_NO_OVERRIDE_VALUE_PROVIDED, SetCurrentModelPhysicsRequestNoOverrideValueProvided),
    (706, SET_CURRENT_MODEL_PHYSICS_REQUEST_DUPLICATE_PHYSICS_GROUP_ID, SetCurrentModelPhysicsRequestDuplicatePhysicsGroupID),

    // Errors related to ItemLoadRequest
    (750, ITEM_FILE_NAME_MISSING, ItemFileNameMissing),
    (751, ITEM_FILE_NAME_NOT_FOUND, ItemFileNameNotFound),
    (752, ITEM_LOAD_LOAD_COOLDOWN_NOT_OVER, ItemLoadLoadCooldownNotOver),
    (753, CANNOT_CURRENTLY_LOAD_ITEM, CannotCurrentlyLoadItem),
    (754, CANNOT_LOAD_ITEM_SCENE_FULL, CannotLoadItemSceneFull),
    (755, ITEM_ORDER_INVALID, ItemOrderInvalid),
    (756, ITEM_ORDER_ALREADY_TAKEN, ItemOrderAlreadyTaken),
    (757, ITEM_LOAD_VALUES_INVALID, ItemLoadValuesInvalid),
    (758, ITEM_CUSTOM_DATA_INVALID, ItemCustomDataInvalid),
    (759, ITEM_CUSTOM_DATA_CANNOT_ASK_RIGHT_NOW, ItemCustomDataCannotAskRightNow),
    (760, ITEM_CUSTOM_DATA_LOAD_REQUEST_REJECTED_BY_USER, ItemCustomDataLoadRequestRejectedByUser),

    // Errors related to ItemUnloadRequest
    (800, CANNOT_CURRENTLY_UNLOAD_ITEM, CannotCurrentlyUnloadItem),

    // Errors related to ItemAnimationControlRequest
    (850, ITEM_ANIMATION_CONTROL_INSTANCE_ID_NOT_FOUND, ItemAnimationControlInstanceIDNotFound),
    (851, ITEM_ANIMATION_CONTROL_UNSUPPORTED_ITEM_TYPE, ItemAnimationControlUnsupportedItemType),
    (852, ITEM_ANIMATION_CONTROL_AUTO_STOP_FRAMES_INVALID, ItemAnimationControlAutoStopFramesInvalid),
    (853, ITEM_ANIMATION_CONTROL_TOO_MANY_AUTO_STOP_FRAMES, ItemAnimationControlTooManyAutoStopFrames),
    (854, ITEM_ANIMATION_CONTROL_SIMPLE_IMAGE_DOES_NOT_SUPPORT_ANIM, ItemAnimationControlSimpleImageDoesNotSupportAnim),

    // Errors related to ItemMoveRequest
    (900, ITEM_MOVE_REQUEST_INSTANCE_ID_NOT_FOUND, ItemMoveRequestInstanceIDNotFound),
    (901, ITEM_MOVE_REQUEST_INVALID_FADE_MODE, ItemMoveRequestInvalidFadeMode),
    (902, ITEM_MOVE_REQUEST_ITEM_ORDER_TAKEN_OR_INVALID, ItemMoveRequestItemOrderTakenOrInvalid),
    (903, ITEM_MOVE_REQUEST_CANNOT_CURRENTLY_CHANGE_ORDER, ItemMoveRequestCannotCurrentlyChangeOrder),

    // Errors related to EventSubscriptionRequest
    (950, EVENT_SUBSCRIPTION_REQUEST_EVENT_TYPE_UNKNOWN, EventSubscriptionRequestEventTypeUnknown),

    // Errors related to ArtMeshSelectionRequest
    (1000, ART_MESH_SELECTION_REQUEST_NO_MODEL_LOADED, ArtMeshSelectionRequestNoModelLoaded),
    (1001, ART_MESH_SELECTION_REQUEST_OTHER_WINDOWS_OPEN, ArtMeshSelectionRequestOtherWindowsOpen),
    (1002, ART_MESH_SELECTION_REQUEST_MODEL_DOES_NOT_HAVE_ART_MESH, ArtMeshSelectionRequestModelDoesNotHaveArtMesh),
    (1003, ART_MESH_SELECTION_REQUEST_ART_MESH_ID_LIST_ERROR, ArtMeshSelectionRequestArtMeshIDListError),

    // Errors related to ItemPinRequest
    (1050, ITEM_PIN_REQUEST_GIVEN_ITEM_NOT_LOADED, ItemPinRequestGivenItemNotLoaded),
    (1051, ITEM_PIN_REQUEST_INVALID_ANGLE_OR_SIZE_TYPE, ItemPinRequestInvalidAngleOrSizeType),
    (1052, ITEM_PIN_REQUEST_MODEL_NOT_FOUND, ItemPinRequestModelNotFound),
    (1053, ITEM_PIN_REQUEST_ART_MESH_NOT_FOUND, ItemPinRequestArtMeshNotFound),
    (1054, ITEM_PIN_REQUEST_PIN_POSITION_INVALID, ItemPinRequestPinPositionInvalid),

    // Errors related to PermissionRequest
    (1100, PERMISSION_REQUEST_UNKNOWN_PERMISSION, PermissionRequestUnknownPermission),
    (1101, PERMISSION_REQUEST_CANNOT_REQUEST_RIGHT_NOW, PermissionRequestCannotRequestRightNow),
    (1102, PERMISSION_REQUEST_FILE_PROBLEM, PermissionRequestFileProblem),

    // Errors related to PostProcessingListReqest
    (1150, POST_PROCESSING_LIST_REQEST_INVALID_FILTER, PostProcessingListReqestInvalidFilter),

    // Errors related to PostProcessingUpdateReqest
    (1200, POST_PROCESSING_UPDATE_REQEST_CANNOT_UPDATE_RIGHT_NOW, PostProcessingUpdateReqestCannotUpdateRightNow),
    (1201, POST_PROCESSING_UPDATE_REQUEST_FADE_TIME_INVALID, PostProcessingUpdateRequestFadeTimeInvalid),
    (1202, POST_PROCESSING_UPDATE_REQUEST_LOADING_PRESET_AND_VALUES, PostProcessingUpdateRequestLoadingPresetAndValues),
    (1203, POST_PROCESSING_UPDATE_REQUEST_PRESET_FILE_LOAD_FAILED, PostProcessingUpdateRequestPresetFileLoadFailed),
    (1204, POST_PROCESSING_UPDATE_REQUEST_VALUE_LIST_INVALID, PostProcessingUpdateRequestValueListInvalid),
    (1205, POST_PROCESSING_UPDATE_REQUEST_VALUE_LIST_CONTAINS_DUPLICATES, PostProcessingUpdateRequestValueListContainsDuplicates),
    (1206, POST_PROCESSING_UPDATE_REQUEST_TRIED_TO_LOAD_RESTRICTED_EFFECT, PostProcessingUpdateRequestTriedToLoadRestrictedEffect),

    // Event config errors
    (100_000, EVENT_TEST_EVENT_TEST_MESSAGE_TOO_LONG, Event_TestEvent_TestMessageTooLong),
    (100_050, EVENT_MODEL_LOADED_EVENT_MODEL_ID_INVALID, Event_ModelLoadedEvent_ModelIDInvalid),
}
