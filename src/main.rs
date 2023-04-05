use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};

#[derive(Debug, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("OPENAI_API_KEY").expect("Missing OPENAI_API_KEY environment variable");
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION,
        format!("Bearer {}", api_key).parse().unwrap());

    headers.insert(reqwest::header::CONTENT_TYPE,
        format!("application/json").parse().unwrap());

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let mut messages = Vec::new();

    messages.push(ChatMessage {
            role: "system".to_string(),
            content: "I am a ChatGPT model. Ask me anything!".to_string(),
        });

    loop {
        print!("Ask ChatGPT: (Ctrl-C to exit) ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.trim().to_string(),
        });

        let chat_request = ChatRequest {
            messages: messages.clone(),
            model: "gpt-3.5-turbo".to_string(),
        };

        let response: Response = client
            .post("https://api.openai.com/v1/chat/completions")
            .json(&chat_request)
            .send()
            .await?
            .json()
            .await?;

        if let Some(choice) = response.choices.first() {
            println!("ChatGPT: {}", choice.message.content.trim());
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: choice.message.content.trim().to_string(),
        });
        } else {
            println!("ChatGPT: Failed to generate a response");
        }
    }
}
