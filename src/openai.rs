use std::{fs, io::Read};
use async_openai::{types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs}, Client};
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::Error;

// There may be a simpler way of creating these lazy strings (maybe caching?)
fn get_system_prompt() -> Lazy<String> {
    Lazy::new(|| {
        let mut prompt = String::new();
        let mut f = fs::File::open("prompts/system-prompt-1.txt")
            .expect("i mean if this file is missing it's a big problem");
        f.read_to_string(&mut prompt)
            .expect("failing to read file is catastrophic");

        prompt
    })
}

fn get_user_prompt() -> Lazy<String> {
    Lazy::new(|| {
        let mut prompt = String::new();
        let mut f = fs::File::open("prompts/user-prompt-1.txt")
            .expect("i mean if this file is missing it's a big problem");
        f.read_to_string(&mut prompt)
            .expect("failing to read file is catastrophic");

        prompt
    })
}

fn get_assistant_prompt() -> Lazy<String> {
    Lazy::new(|| {
        let mut prompt = String::new();
        let mut f = fs::File::open("prompts/assistant-prompt-1.txt")
            .expect("i mean if this file is missing it's a big problem");
        f.read_to_string(&mut prompt)
            .expect("failing to read file is catastrophic");

        prompt
    })
}


pub async fn autofill_test(text: &String) -> Result<String, Error> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<body.*?>")
        .expect("this is hard-coded, if it breaks i am silly"));

    let Some(body) = RE.split(text).nth(1) else {
        return Err(Error::BadRequest("no body in html"));
    };

    let mut body = body.to_string();
    body.truncate(10000);

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u16)
        .model("gpt-4")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(get_system_prompt().to_string())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(get_user_prompt().to_string())
                .build()?
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(get_assistant_prompt().to_string())
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(body)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    Ok(response.choices[0].message.content.clone().unwrap_or_default())
}
