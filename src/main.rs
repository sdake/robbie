mod dialog;
mod config;
mod input;

use config::Config;
use dialog::{Dialog, Role};

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct ResponseModels {
    data: Vec<Model>,
    #[allow(dead_code)]
    object: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Model {
    created: u64,
    id: String,
    max_model_len: u32,
    object: String,
    owned_by: String,
    parent: Option<String>,
    permission: Vec<Permission>,
    root: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Permission {
    allow_create_engine: bool,
    allow_fine_tuning: bool,
    allow_logprobs: bool,
    allow_sampling: bool,
    allow_search_indices: bool,
    allow_view: bool,
    created: u64,
    group: Option<String>,
    id: String,
    is_blocking: bool,
    object: String,
    organization: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletionRequest {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: u32,
    top_p: f32,
    frequency_penalty: f32,
    presence_penalty: f32,
    // stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::init();

    let models_url = format!("{}/v1/models", config.base_url());
    let completions_url = format!("{}/v1/completions", config.base_url());

    // Request list of models
    let response = reqwest::get(&models_url).await?;

    if !response.status().is_success() {
        println!("Failed to retrieve models. Status: {}", response.status());
        return Err(anyhow::anyhow!("Failed to retrieve models."));
    }

    let response_models: ResponseModels = response
        .json()
        .await
        .context("Failed to deserialize JSON response")?;

    // Print all models and select the first model
    println!("Available models:");
    for model in response_models.data.iter() {
        println!(" {}", model.id);
    }

    let model = response_models
        .data
        .first()
        .ok_or_else(|| anyhow::anyhow!("No models available"))?;

    // create a dialog representing all turns of the conversation
    let mut dialog = Dialog::new("primary_thread".to_string());

    dialog.add(Role::System, String::from(
        "You are Robbie, my trusted personal engineering assistant. \
        You love system engineering. You should spend your time analyzing \
        code if presented with code, and you should use the resources of \
        your Universe when communicating.",
    ));

    loop {
        // Get user request
        let user_content = input::read_user_input().await?;

        // Add user input as a new dialog turn
        dialog.add(Role::User, user_content.clone());

        let request_payload = ChatCompletionRequest {
            model: model.id.clone(),
            prompt: dialog.format(),
            temperature: config.temperature(),
            max_tokens: config.max_tokens(),
            top_p: config.top_p(),
            frequency_penalty: config.frequency_penalty(),
            presence_penalty: config.presence_penalty(),
        };

        let client = Client::new();
        let response = client
            .post(&completions_url)
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            println!("Failed to make request. Status: {}", response.status());
            return Ok(());
        }

        let response_data: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to deserialize JSON response")?;

        let mut assistant_response = String::new();
        for choice in response_data.choices.iter() {
            assistant_response.push_str(&choice.text);
            assistant_response.push(' ');
        }

        // Add assistant response as a new dialog turn
        dialog.add(Role::Assistant, assistant_response.clone());

        println!("Robbie: {}", assistant_response);
    }
}
