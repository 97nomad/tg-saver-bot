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
    let target_path = build_final_path(&target_dir, tokens, &default_filename);

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

fn build_final_path(target_dir: &Path, tokens: &Vec<MessageTokens>, file_path: &Path) -> PathBuf {
    let mut result_path = target_dir.to_path_buf();

    // Add hashtag tokens to path
    for token in tokens {
        match token {
            MessageTokens::Hashtag(text) => result_path.push(text),
            _ => (),
        }
    }

    // Generate filename from first text token and extract extension from file_path
    // TODO: generate filename from timestamp if we don't have text token
    let filename = tokens
        .iter()
        .find_map(|token| match token {
            MessageTokens::Text(text) => Some(text),
            _ => None,
        })
        .map(|text| {
            file_path
                .extension()
                .map(|ext| format!("{}.{}", text, ext.to_str().unwrap()))
                .unwrap_or(text.to_string())
        })
        .unwrap_or(file_path.file_name().unwrap().to_str().unwrap().to_string());
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

    fn test_files() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/")
            .to_path_buf()
    }

    #[test]
    fn non_existing_file() {
        let expected = test_files().join("non_existing_file");
        let result = build_final_path(&test_files(), &vec![], Path::new("non_existing_file"));
        assert_eq!(result, expected);
    }

    #[test]
    fn file_without_extension() {
        let expected = test_files().join("file_without_extension_1");
        let result = build_final_path(&test_files(), &vec![], Path::new("file_without_extension"));
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_extension() {
        let expected = test_files().join("file_with_extension_2.txt");
        let result = build_final_path(&test_files(), &vec![], Path::new("file_with_extension.txt"));
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
            ],
            Path::new("file"),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_hashtag_and_filename() {
        let expected = test_files().join("folder").join("parsed_file.txt");
        let result = build_final_path(
            &test_files(),
            &vec![
                MessageTokens::Hashtag("folder".to_owned()),
                MessageTokens::Text("parsed_file".to_owned()),
            ],
            Path::new("file.txt"),
        );
        assert_eq!(result, expected)
    }

    #[test]
    fn file_with_hashtag_and_filename_without_extension() {
        let expected = test_files().join("folder").join("parsed_file");
        let result = build_final_path(
            &test_files(),
            &vec![
                MessageTokens::Hashtag("folder".to_owned()),
                MessageTokens::Text("parsed_file".to_owned()),
            ],
            Path::new("file"),
        );
        assert_eq!(result, expected)
    }
}
