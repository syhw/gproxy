use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::constants::GEMINI_CODE_ASSIST_ENDPOINT;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIToolCall {
    pub id: String,
    pub r#type: String,
    pub function: OpenAIFunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    pub stream: Option<bool>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<OpenAITool>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAITool {
    pub r#type: String,
    pub function: OpenAIFunctionDefinition,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIFunctionDefinition {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiContentPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<GeminiFunctionResponse>,
    #[serde(rename = "thoughtSignature", skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiFunctionResponse {
    pub name: String,
    pub response: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiContentPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(rename = "maxOutputTokens", skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiWrappedRequest {
    pub project: String,
    pub model: String,
    pub request: GeminiRequest,
}

pub fn transform_openai_to_gemini(request: &OpenAIRequest, project_id: &str) -> (String, GeminiWrappedRequest, bool) {
    let streaming = request.stream.unwrap_or(false);
    let action = if streaming { "streamGenerateContent" } else { "generateContent" };
    let url = format!("{}/v1internal:{}{}", GEMINI_CODE_ASSIST_ENDPOINT, action, if streaming { "?alt=sse" } else { "" });

    let mut contents = Vec::new();
    for msg in &request.messages {
        let mut parts = Vec::new();
        
        if let Some(content) = &msg.content {
            parts.push(GeminiContentPart {
                text: Some(content.clone()),
                function_call: None,
                function_response: None,
                thought_signature: None,
            });
        }
        
        if let Some(tool_calls) = &msg.tool_calls {
            for tc in tool_calls {
                let args: Value = serde_json::from_str(&tc.function.arguments).unwrap_or(Value::Null);
                parts.push(GeminiContentPart {
                    text: None,
                    function_call: Some(GeminiFunctionCall {
                        name: tc.function.name.clone(),
                        args,
                    }),
                    function_response: None,
                    thought_signature: Some("skip_thought_signature_validator".to_string()),
                });
            }
        }
        
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "model",
            "tool" => "function",
            "system" => continue, // Skip system for now or handle separately
            _ => "user",
        };
        
        contents.push(GeminiContent {
            role: role.to_string(),
            parts,
        });
    }

    let generation_config = Some(GeminiGenerationConfig {
        temperature: request.temperature,
        max_output_tokens: request.max_tokens,
    });

    let tools = request.tools.as_ref().map(|t| vec![GeminiTool {
        function_declarations: t.iter().map(|ot| GeminiFunctionDeclaration {
            name: ot.function.name.clone(),
            description: ot.function.description.clone(),
            parameters: ot.function.parameters.clone(),
        }).collect(),
    }]);

    let gemini_request = GeminiRequest {
        contents,
        generation_config,
        tools,
    };

    let wrapped = GeminiWrappedRequest {
        project: project_id.to_string(),
        model: request.model.clone(),
        request: gemini_request,
    };

    (url, wrapped, streaming)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIChoice {
    pub index: u32,
    pub message: OpenAIMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiCandidate {
    pub content: Option<GeminiContent>,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiResponse {
    pub candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<GeminiUsageMetadata>,
}

pub fn transform_gemini_to_openai(gemini_res: &GeminiResponse, model: &str) -> OpenAIResponse {
    let mut choices = Vec::new();
    
    if let Some(candidates) = &gemini_res.candidates {
        for (i, candidate) in candidates.iter().enumerate() {
            let mut text = String::new();
            let mut tool_calls = Vec::new();
            
            if let Some(content) = &candidate.content {
                for part in &content.parts {
                    if let Some(t) = &part.text {
                        text.push_str(t);
                    }
                    if let Some(fc) = &part.function_call {
                        tool_calls.push(OpenAIToolCall {
                            id: format!("call_{}", rand::random::<u32>()),
                            r#type: "function".to_string(),
                            function: OpenAIFunctionCall {
                                name: fc.name.clone(),
                                arguments: fc.args.to_string(),
                            },
                        });
                    }
                }
            }
            
            choices.push(OpenAIChoice {
                index: candidate.index.unwrap_or(i as u32),
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: if text.is_empty() { None } else { Some(text) },
                    tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                    tool_call_id: None,
                },
                finish_reason: candidate.finish_reason.clone().or(Some("stop".to_string())),
            });
        }
    }

    let usage = gemini_res.usage_metadata.as_ref().map(|u| OpenAIUsage {
        prompt_tokens: u.prompt_token_count.unwrap_or(0),
        completion_tokens: u.candidates_token_count.unwrap_or(0),
        total_tokens: u.total_token_count.unwrap_or(0),
    });

    OpenAIResponse {
        id: format!("chatcmpl-{}", rand::random::<u32>()),
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        model: model.to_string(),
        choices,
        usage,
    }
}
