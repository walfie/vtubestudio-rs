use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub api_name: String,
    pub api_version: String,
    #[serde(rename = "requestID")]
    pub request_id: String,
    #[serde(flatten)]
    pub data: RequestData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "messageType", content = "data")]
pub enum RequestData {
    #[serde(rename = "APIStateRequest")]
    ApiState,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "messageType", content = "data")]
pub enum ResponseData {
    #[serde(rename = "APIStateResponse")]
    ApiState(ApiStateResponse),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStateResponse {
    pub active: bool,
    pub v_tube_studio_version: String,
    pub current_session_authenticated: bool,
}

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
                request_id: "MyIDWithLessThan64Characters".into(),
                data: RequestData::ApiState,
            })
            .unwrap(),
            json!({
                "apiName": "VTubeStudioPublicAPI",
                "apiVersion": "1.0",
                "requestID": "MyIDWithLessThan64Characters",
                "messageType": "APIStateRequest",
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
                data: ResponseData::ApiState(ApiStateResponse {
                    active: true,
                    v_tube_studio_version: "1.9.0".into(),
                    current_session_authenticated: false
                }),
            }
        )
    }
}
