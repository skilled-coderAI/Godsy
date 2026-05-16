use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponseStatus {
    pub code: u16,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub id: String,
    pub method: HttpMethod,
    pub path: String,
    pub summary: String,
    pub auth_required: bool,
    #[serde(default)]
    pub query_params: Vec<ApiField>,
    #[serde(default)]
    pub path_params: Vec<ApiField>,
    pub request_body: Option<ApiBody>,
    pub response_body: Option<ApiBody>,
    pub statuses: Vec<ApiResponseStatus>,
    #[serde(default)]
    pub entity_refs: Vec<String>,
    #[serde(default)]
    pub component_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiField {
    pub name: String,
    pub ty: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBody {
    pub content_type: String,
    pub fields: Vec<ApiField>,
    #[serde(default)]
    pub example: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    None,
    ApiKey,
    Bearer,
    Session,
    OAuth2,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiSpec {
    pub base_url: String,
    pub auth: Option<ApiAuth>,
    pub endpoints: Vec<ApiEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAuth {
    pub scheme: AuthScheme,
    pub description: String,
}
