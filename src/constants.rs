pub const GEMINI_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];
pub const GEMINI_REDIRECT_URI: &str = "http://localhost:8085/oauth2callback";
pub const GEMINI_CODE_ASSIST_ENDPOINT: &str = "https://cloudcode-pa.googleapis.com";

pub const CODE_ASSIST_HEADERS: &[(&str, &str)] = &[
    ("User-Agent", "google-api-nodejs-client/9.15.1"),
    ("X-Goog-Api-Client", "gl-node/22.17.0"),
    ("Client-Metadata", "ideType=IDE_UNSPECIFIED,platform=PLATFORM_UNSPECIFIED,pluginType=GEMINI"),
];
