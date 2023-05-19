use reqwest::{Client, header::HeaderMap};
use serde::{Serialize, Deserialize};
use std::env;
use std::io::{self, Write, stdout};
use reqwest_eventsource::{Event, RequestBuilderExt};
use reqwest_eventsource::EventSource as ReqEventSource;
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

/// Represents the response from the chat API call.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

/// Represents a choice in the chat API response.
#[derive(Debug, Deserialize)]
pub struct Choice {
    #[serde(default)]
    pub delta: Delta,

    #[serde(default)]
    pub index: i64,

    pub finish_reason: Option<String>,
}

/// Represents a delta in the chat API call.
#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
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
                Ok(Event::Open) => {
                    println!("Connection Open!");
                },
                Ok(Event::Message(message)) => {
                    //at the end , message.data == [DONE]
                    if &message.data == "[DONE]" {
                        println!("{}","");
                        break;
                    } else {
                      let chat_response: ChatResponse = serde_json::from_str(&message.data)?;
                      let mut lock = stdout().lock();
                      write!(lock, "{}",
                               match chat_response.choices[0].delta.content{
                                   None => "",
                                   Some(ref x) => x,
                               }
                      ).unwrap();
                      io::stdout().flush().unwrap();
                    }
                },
                Err(err) => {
                    println!("There is an Error: {}", err);
                    _event_source.close();
                }
            }
        }
       input.clear();
    }
}
