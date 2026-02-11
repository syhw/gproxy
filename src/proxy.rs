use anyhow::{Result, anyhow};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, ACCEPT};
use crate::config::{load_config, save_config};
use crate::constants::{GEMINI_CODE_ASSIST_ENDPOINT, CODE_ASSIST_HEADERS};
use crate::transform::{OpenAIRequest, transform_openai_to_gemini};
use serde_json::json;

pub async fn load_managed_project(access_token: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", access_token))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    for (key, value) in CODE_ASSIST_HEADERS {
        headers.insert(reqwest::header::HeaderName::from_bytes(key.as_bytes())?, HeaderValue::from_str(value)?);
    }

    let body = json!({
        "metadata": {
            "ideType": "IDE_UNSPECIFIED",
            "platform": "PLATFORM_UNSPECIFIED",
            "pluginType": "GEMINI",
        },
    });

    let res = client.post(format!("{}/v1internal:loadCodeAssist", GEMINI_CODE_ASSIST_ENDPOINT))
        .headers(headers)
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Ok(None);
    }

    let data: serde_json::Value = res.json().await?;
    let project_id = data["cloudaicompanionProject"]["id"]
        .as_str()
        .or_else(|| data["cloudaicompanionProject"].as_str())
        .map(|s| s.to_string());

    Ok(project_id)
}

pub async fn get_auth() -> Result<(String, String)> {
    let mut config = load_config()?;
    let auth = config.auth.as_ref().ok_or_else(|| anyhow!("No authentication found. Run 'gemini-proxy login' first."))?;
    
    let access_token = auth.access_token.clone();
    let mut project_id = config.project_id.clone().unwrap_or_else(|| "default".to_string());

    if project_id == "default" {
        if let Ok(Some(managed)) = load_managed_project(&access_token).await {
            project_id = managed;
            config.project_id = Some(project_id.clone());
            save_config(&config)?;
        }
    }

    Ok((access_token, project_id))
}

pub async fn proxy_request(body: OpenAIRequest) -> Result<reqwest::Response> {
    let (token, project_id) = get_auth().await?;
    let (url, gemini_body, streaming) = transform_openai_to_gemini(&body, &project_id);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", token))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    for (key, value) in CODE_ASSIST_HEADERS {
        headers.insert(reqwest::header::HeaderName::from_bytes(key.as_bytes())?, HeaderValue::from_str(value)?);
    }

    if streaming {
        headers.insert(ACCEPT, HeaderValue::from_static("text/event-stream"));
    }

    let res = client.post(url)
        .headers(headers)
        .json(&gemini_body)
        .send()
        .await?;

    Ok(res)
}
