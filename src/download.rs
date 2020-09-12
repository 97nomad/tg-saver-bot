use ::chrono::NaiveDateTime;
use ::telegram_bot::prelude::CanGetFile;
use ::telegram_bot::types::ToFileRef;
use ::telegram_bot::Api;

use crate::parser::MessageTokens;
use crate::settings::Settings;

use std::path::{Path, PathBuf};

pub async fn download_file<F>(
    file_ref: F,
    api: &Api,
    settings: &Settings,
    tokens: &Vec<MessageTokens>,
    timestamp: NaiveDateTime,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>>
where
    F: ToFileRef,
{
    let token = &settings.telegram.token;
    let target_dir = Path::new(&settings.download.target_dir);

    let file_info = api.send(file_ref.get_file()).await?;
    let file_path = &file_info.file_path.clone().unwrap();
    let size = &file_info.file_size.unwrap();

    let default_filename = Path::new(&file_path);
    let target_path = build_final_path(&target_dir, tokens, &default_filename, timestamp);

    println!(
        "Downloading file {} with size {}",
        target_path.display(),
        size
    );

    let bytes = reqwest::get(&file_info.get_url(token).unwrap())
        .await?
        .bytes()
        .await?;

    // Create directories
    tokio::fs::create_dir_all(target_path.parent().unwrap()).await?;
    // Download file
    tokio::fs::write(&target_path, bytes).await?;

    Ok(target_path)
}

fn build_final_path(
    target_dir: &Path,
    tokens: &Vec<MessageTokens>,
    file_path: &Path,
    timestamp: NaiveDateTime,
) -> PathBuf {
    let mut result_path = target_dir.to_path_buf();

    // Add hashtag tokens to path
    for token in tokens {
        match token {
            MessageTokens::Hashtag(text) => result_path.push(text),
            _ => (),
        }
    }

    // Generate filename from first text token and extract extension from file_path
    let extension_str = file_path.extension().and_then(|ext| ext.to_str());
    let filename = tokens
        .iter()
        .find_map(|token| match token {
            MessageTokens::Text(text) => Some(text),
            _ => None,
        })
        .map(|text| {
            extension_str
                .map(|ext| format!("{}.{}", text, ext))
                .unwrap_or(text.to_string())
        })
        .unwrap_or({
            let name = timestamp.format("file_%Y-%m-%d_%H-%M-%S").to_string();
            extension_str
                .map(|ext| format!("{}.{}", name, ext))
                .unwrap_or(name.to_string())
        });
    result_path.push(filename);

    // Check if file exists and add _x after filename
    if result_path.exists() {
        let mut tries: usize = 0;
        let mut not_existent_path = result_path.clone();

        while not_existent_path.exists() {
            tries += 1;

            let file_name = match result_path.extension() {
                Some(ext) => format!(
                    "{}_{}.{}",
                    result_path.file_stem().unwrap().to_str().unwrap(),
                    tries,
                    ext.to_str().unwrap()
                ),
                None => format!(
                    "{}_{}",
                    result_path.file_stem().unwrap().to_str().unwrap(),
                    tries,
                ),
            };

            not_existent_path = result_path.to_path_buf().with_file_name(file_name)
        }

        not_existent_path
    } else {
        result_path.to_path_buf()
    }
}

#[cfg(test)]
mod file_path_test {
    use super::*;
    use ::chrono::prelude::Utc;

    fn test_files() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/")
            .to_path_buf()
    }

    #[test]
    fn non_existing_file() {
        let expected = test_files().join("non_existing_file");
        let result = build_final_path(
            &test_files(),
            &vec![MessageTokens::Text("non_existing_file".to_owned())],
            Path::new("non_existing_file"),
            Utc::now().naive_utc(),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn file_without_extension() {
        let expected = test_files().join("file_without_extension_1");
        let result = build_final_path(
            &test_files(),
            &vec![MessageTokens::Text("file_without_extension".to_owned())],
            Path::new("file_without_extension"),
            Utc::now().naive_utc(),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_extension() {
        let expected = test_files().join("file_with_extension_2.txt");
        let result = build_final_path(
            &test_files(),
            &vec![MessageTokens::Text("file_with_extension".to_owned())],
            Path::new("file_with_extension.txt"),
            Utc::now().naive_utc(),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_hashtags() {
        let expected = test_files().join("one").join("two").join("file");
        let result = build_final_path(
            &test_files(),
            &vec![
                MessageTokens::Hashtag("one".to_owned()),
                MessageTokens::Hashtag("two".to_owned()),
                MessageTokens::Text("file".to_owned()),
            ],
            Path::new("file"),
            Utc::now().naive_utc(),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_hashtag_and_extension() {
        let expected = test_files().join("folder").join("parsed_file.txt");
        let result = build_final_path(
            &test_files(),
            &vec![
                MessageTokens::Hashtag("folder".to_owned()),
                MessageTokens::Text("parsed_file".to_owned()),
            ],
            Path::new("file.txt"),
            Utc::now().naive_utc(),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_withput_text_token() {
        let time = Utc::now().naive_utc();
        let expected = test_files().join("folder").join(format!(
            "file_{}",
            time.format("%Y-%m-%d_%H-%M-%S").to_string()
        ));
        let result = build_final_path(
            &test_files(),
            &vec![MessageTokens::Hashtag("folder".to_owned())],
            Path::new("file"),
            time,
        );
        assert_eq!(result, expected)
    }
}
