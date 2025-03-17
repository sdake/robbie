mod config;
mod dialog;
mod input;

use config::Config;
use dialog::{Dialog, Role};

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::signal;
use tokio::sync::{oneshot, Mutex};

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

    // Create a client with no TLS certificate verification
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Request list of models
    let response = client
        .get(&models_url)
        .header("Authorization", "Bearer robbie")
        .send()
        .await?;

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

    // Use system prompt from config if available, otherwise use default
    let system_prompt = config
        .system_prompt()
        .unwrap_or("You are Robbie, my trusted personal engineering assistant.");

    dialog.add(Role::User, String::from(system_prompt));

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

        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        println!("Generating response... (Press Ctrl+C to interrupt)");

        // Create a channel for cancellation
        let (cancel_tx, cancel_rx) = oneshot::channel();

        // Spawn a task to handle Ctrl+C
        let cancel_tx = Arc::new(Mutex::new(Some(cancel_tx)));
        let ctrl_c_tx = cancel_tx.clone();

        tokio::spawn(async move {
            if (signal::ctrl_c().await).is_ok() {
                println!("\nInference interrupted by user.");
                if let Some(tx) = ctrl_c_tx.lock().await.take() {
                    let _ = tx.send(());
                }
            }
        });

        // Prepare the request
        let request = client
            .post(&completions_url)
            .header("Authorization", "Bearer robbie")
            .json(&request_payload)
            .build()?;

        // Create a response future that can be cancelled
        let response_future = client.execute(request);

        // Wait for either response or cancellation
        let response = tokio::select! {
            response = response_future => {
                match response {
                    Ok(resp) => resp,
                    Err(e) => {
                        println!("Request error: {}", e);
                        return Ok(());
                    }
                }
            }
            _ = cancel_rx => {
                return Ok(());
            }
        };

        if !response.status().is_success() {
            println!("Failed to make request. Status: {}", response.status());
            return Ok(());
        }

        // Parse the JSON response with cancellation support
        let json_future = response.json::<ChatCompletionResponse>();

        let response_data = tokio::select! {
            json_result = json_future => {
                match json_result {
                    Ok(data) => data,
                    Err(e) => {
                        println!("Failed to deserialize JSON response: {}", e);
                        return Ok(());
                    }
                }
            }
            _ = signal::ctrl_c() => {
                println!("\nJSON parsing interrupted by user.");
                return Ok(());
            }
        };

        let mut assistant_response = String::new();
        for choice in response_data.choices.iter() {
            assistant_response.push_str(&choice.text);
            assistant_response.push(' ');
        }

        // Add model response as a new dialog turn
        dialog.add(Role::Model, assistant_response.clone());

        println!("Robbie: {}", assistant_response);
    }
}
