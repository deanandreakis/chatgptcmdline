use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use reqwest_eventsource::{Event, RequestBuilderExt};
use reqwest_eventsource::EventSource as ReqEventSource;
//pub use futures_core::stream::Stream;

//use futures::executor; //standard executors to provide a context for futures and streams
//use futures::executor::ThreadPool;
use futures::StreamExt;

#[derive(Debug, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    model: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct Response {
    choice: Choice,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Delta,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Delta {
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

    let mut input = String::new();

    loop {
        print!("Ask ChatGPT: (Ctrl-C to exit) ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut input)?;
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: input.trim().to_string(),
        });

        let chat_request = ChatRequest {
            messages: messages.clone(),
            model: "gpt-3.5-turbo".to_string(),
            stream: true,
        };

        let req_builder = client.post("https://api.openai.com/v1/chat/completions").json(&chat_request);
        let mut _event_source:ReqEventSource = RequestBuilderExt::eventsource(req_builder)?;

        while let Some(event) = _event_source.next().await {
            match event {
                Ok(Event::Open) => println!("Connection Open!"),
                Ok(Event::Message(message)) => println!("Message: {:#?}", message),
                Err(err) => {
                    println!("Error: {}", err);
                    _event_source.close();
                }
            }
        }
       input.clear();
    }
}
