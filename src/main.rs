extern crate config;
extern crate serde;

#[macro_use]
extern crate serde_derive;

pub mod settings;
use settings::Settings;

mod download;
pub mod parser;

use parser::MessageTokens;

use futures::StreamExt;
use telegram_bot::*;

use std::cmp::Ordering;
use std::iter::Iterator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let settings = Settings::new()?;
    println!("{:?}", settings);

    let token = &settings.telegram.token;
    let api = Api::new(&token);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;

        if let UpdateKind::Message(message) = update.kind {
            if is_user_allowed(&message, &settings) {
                process_message(message, &api, &settings).await?;
            } else {
                let name = message.from.username.unwrap_or(message.from.first_name);
                println!("<{}>: unallowed user trying to send something", name);
            }
        }
    }

    Ok(())
}

async fn process_message(
    message: Message,
    api: &Api,
    settings: &Settings,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match message.kind {
        MessageKind::Text { ref data, .. } => process_text_message(&message, data, &api).await,
        MessageKind::Photo {
            ref data,
            ref caption,
            ref media_group_id,
        } => process_photo_message(&message, &data, caption, media_group_id, &settings, &api).await,
        MessageKind::Sticker { ref data, .. } => {
            process_sticker_message(&message, data, &settings, &api).await
        }

        _ => Ok(()),
    }
}

fn is_user_allowed(message: &Message, settings: &Settings) -> bool {
    match &message.from.username {
        None => false,
        Some(username) => settings.telegram.allowed_usernames.contains(username),
    }
}

async fn process_text_message(
    message: &Message,
    text: &str,
    api: &Api,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("<{}>: {}", &message.from.first_name, text);

    api.send(message.text_reply(format!(
        "Hi, {}! You wrote '{}'",
        &message.from.first_name, text
    )))
    .await?;

    Ok(())
}

// TODO: Add LRU cache to save media_group_ids
async fn process_photo_message(
    message: &Message,
    photos: &Vec<PhotoSize>,
    caption: &Option<String>,
    _media_group_id: &Option<String>,
    settings: &Settings,
    api: &Api,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(biggest_image) = find_biggest_image(photos) {
        let mut tokens: Vec<MessageTokens> = settings
            .download
            .image_tags
            .iter()
            .map(|tag| MessageTokens::Hashtag(tag.to_owned()))
            .collect();
        let parsed_tokens = caption
            .as_ref()
            .map(|text| parser::parse_message(&text.clone()))
            .unwrap_or(vec![]);
	tokens.extend(parsed_tokens);
	
        let path = download::download_file(biggest_image, api, settings, &tokens).await?;

        api.send(message.text_reply(format!("Файл сохранён в {}", path.display())))
            .await?;
    } else {
        println!("Strange photo without file");
    }

    Ok(())
}

async fn process_sticker_message(
    message: &Message,
    sticker: &Sticker,
    settings: &Settings,
    api: &Api,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(image) = &sticker.thumb {
        let tokens = settings
            .download
            .sticker_tags
            .iter()
            .map(|tag| MessageTokens::Hashtag(tag.to_owned()))
            .collect();
        let path = download::download_file(image, api, settings, &tokens).await?;

        api.send(message.text_reply(format!("Файл сохранён в {}", path.display())))
            .await?;
    } else {
        println!("Strange sticker without thumb {:?}", sticker);
    }

    Ok(())
}

fn find_biggest_image(images: &Vec<PhotoSize>) -> Option<&PhotoSize> {
    images
        .iter()
        .max_by(|&x, &y| match (x.file_size, y.file_size) {
            (Some(x), Some(y)) => x.cmp(&y),
            (None, Some(_y)) => Ordering::Less,
            (Some(_x), None) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        })
}
