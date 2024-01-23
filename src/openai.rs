use async_openai::{types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs}, Client};
use once_cell::sync::Lazy;
use regex::Regex;

pub async fn autofill_test(text: &String) -> Option<String>{
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"<body.*?>").unwrap());

    let Some(body) = RE.split(text).nth(1) else {
        return None;
    };

    let mut body = body.to_string();
    body.truncate(10000);

    let client = Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u16)
        .model("gpt-4")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful program that simply parses html source code \
                          and returns a single regex template that can be repeatedly used to pull out the title, a possible link, and possible \
                          content from the repetitive site content.")
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestSystemMessageArgs::default()
                .content("However, use {*} as the wildcard instead of .*? and {%} as capture groups instead of (.*?). \
                          Use at most only 3 {%} capture groups and simply discard repeated information with {*}. \
                          Simplicity is key here, use as little hard coded html information (class, id, etc...) and
                          explicit content as possible.")
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(
r##"
<div class="tl-page">
<header class="tl-main-header">
<div class="tl-title">
<h1>Plurrrr</h1>
<span>home</span>
</div>
<nav>
<ul class="tl-navigation">
<li>
<a href="/">home</a>
</li>
<li>
<a href="/subscribe.html">subscribe</a>
</li>
<li>
<a href="/tags/">tags</a>
</li>
<li>
<a href="/about.html">about</a>
</li>
</ul>
</nav>
</header>
<main>
<time class="tl-date" datetime="2023-07-07">
<a href="archive/2023/07/07.html" title="Nix shell template, IN vs. ANY, and Corinna in the Perl Core">Fri 07 Jul 2023</a>
</time>
<article>
<h2 id="nix-shell-template">
<a href="https://plurrrr.com/archive/2023/07/07.html#nix-shell-template">Nix shell template</a>
</h2>
<blockquote>
<p>Nix shells are the best tool for creating software developmentenvironments right now. This article provides a template to get youstarted with Nix shells from scratch, and explains how to add commonfeatures.</p>
</blockquote>
<p>Source: <a href="https://paperless.blog/nix-shell-template">Nix shelltemplate</a>, an article byVictor Engmark.</p>
<ul class="tl-tags">
<li>
<a href="https://plurrrr.com/tags/2023/nix.html">nix</a>
</li>
</ul>
</article>
<article>
<h2 id="til---in-is-not-the-same-as-any">
"##
                )
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(r##"
<article>
<h2 id="{*}">
<a href="{%}">{%}</a>
</h2>
<blockquote>
<p>{%}</p>
</blockquote>
{*}
</article>
"##)
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(body)
                .build()
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap();

    let response = client.chat().create(request).await.unwrap();
    eprintln!("{:?}", response);

    response.choices[0].message.content.clone()
}
