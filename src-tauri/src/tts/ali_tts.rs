use log::{debug, error, warn};
use nanoid::nanoid;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_retry::{RetryIf, strategy::FibonacciBackoff};

use crate::tts::{TTSError, TTSFile, TTSService, get_wav_len};

const DEFAULT_TEMP_DIR: &str = "temp";
const DEFAULT_ALI_TTS_URL: &str =
    "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation";

pub struct AliTTS {
    url: String,
    key: String,
    temp_dir: String,
}

impl AliTTS {
    /// Create a new instance of `AliTTS` with the given URL and key.
    /// url: The API endpoint for the Ali TTS service.
    /// key: The ALI API key for authentication.
    pub fn new(key: String) -> Self {
        Self {
            url: DEFAULT_ALI_TTS_URL.to_owned(),
            key,
            temp_dir: DEFAULT_TEMP_DIR.to_owned(),
        }
    }

    pub fn with_temp_dir<'a>(self, temp_dir: impl AsRef<&'a str>) -> Self {
        Self {
            temp_dir: temp_dir.as_ref().to_string(),
            ..self
        }
    }

    pub fn with_url(self, url: String) -> Self {
        Self { url, ..self }
    }
}

const ALI_CODE_THROTTLING_RATE_QUOTE: &str = "Throttling.RateQuota";
const ALI_CODE_INVALID_API_KEY: &str = "InvalidApiKey";

impl AliTTS {
    async fn req_tts_retry(
        index: usize,
        text: &String,
        temp_dir: &PathBuf,
        key: &String,
        url: &String,
    ) -> Result<TTSFile, TTSError> {
        let client = Client::new();

        let body = Self::build_body(text);
        let response = client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", key))
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(Self::to_network_err)?;

        if response.status() != 200 {
            let data: ApiError = response.json().await.map_err(Self::to_network_err)?;
            debug!("ali tts response data: {:?}", data);
            match data.code.as_str() {
                ALI_CODE_THROTTLING_RATE_QUOTE => {
                    return Err(TTSError::Network(ALI_CODE_THROTTLING_RATE_QUOTE.to_owned()));
                }
                ALI_CODE_INVALID_API_KEY => {
                    return Err(TTSError::Network("无效的阿里 API Key".to_owned()));
                }
                _ => {}
            }

            return Err(TTSError::Network(format!(
                "request ali tts failed: {}",
                data.message
            )));
        }

        let data: ApiResponse = response.json().await.map_err(Self::to_network_err)?;

        let audio_url = data.output.audio.url;

        let response = client
            .get(audio_url)
            .send()
            .await
            .map_err(Self::to_network_err)?;
        let bytes = response.bytes().await.map_err(Self::to_network_err)?;

        let file = temp_dir.join(format!("voice_{}_{:03}.wav", nanoid!(10), index));

        tokio::fs::write(&file, bytes).await?;

        let duration = get_wav_len(&file).await?;

        Ok(TTSFile {
            path: file,
            text: text.clone(),
            duration,
        })
    }

    async fn req_tts<'a>(
        index: usize,
        text: String,
        temp_dir: PathBuf,
        key: String,
        url: String,
    ) -> Result<TTSFile, TTSError> {
        let strategy = FibonacciBackoff::from_millis(1500).take(5);

        fn retry_if(error: &TTSError) -> bool {
            if matches!(error, TTSError::Network(v) if v.as_str() == ALI_CODE_THROTTLING_RATE_QUOTE) {
                warn!("Fucking Ali TTS Requests rate limit exceeded, Retrying...");
                true
            } else {
                false
            }
        }

        RetryIf::spawn(
            strategy,
            || async { Self::req_tts_retry(index, &text, &temp_dir, &key, &url).await },
            retry_if,
        )
        .await
    }

    fn build_body(text: &String) -> String {
        let len = text.chars().count();
        let mut body = String::with_capacity(100 + len);

        body.push_str(
            r#"{
    "model": "qwen-tts",
    "input": {
        "text": ""#,
        );
        body.push_str(&text);
        body.push_str(
            r#"",
        "voice": "Serena"
    }"#,
        );

        body
    }

    fn to_network_err(e: reqwest::Error) -> TTSError {
        TTSError::Network(e.to_string())
    }
}

#[async_trait::async_trait]
impl TTSService for AliTTS {
    async fn tts(&self, text_list: &Vec<String>) -> Result<Vec<TTSFile>, TTSError> {
        if self.key.is_empty() {
            return Err(TTSError::Network("请设置阿里 API Key".to_owned()));
        }

        let mut list = vec![];

        let tmp_path = PathBuf::from(&self.temp_dir);
        tokio::fs::create_dir_all(&tmp_path).await?;

        let mut handles = vec![];

        for (index, text) in text_list.iter().enumerate() {
            let key = self.key.clone();
            let text = text.clone();
            let temp_dir = tmp_path.clone();
            let url = self.url.clone();

            handles.push(tokio::spawn(async move {
                Self::req_tts(index, text, temp_dir, key, url).await
            }));
            // Fuck Ali Requests rate limit exceeded
            // tokio::time::sleep(Duration::from_millis(500)).await;
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(value)) => {
                    list.push(value);
                }
                Ok(Err(e)) => {
                    error!("Ali TTS request failed: {}", e.to_string());
                    return Err(TTSError::Network(e.to_string()));
                }
                Err(e) => {
                    error!("Ali TTS handle failed");
                    return Err(TTSError::HandleFailed(e.to_string()));
                }
            }
        }

        Ok(list)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Audio {
    pub expires_at: i64,
    pub data: String,
    pub id: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Output {
    pub finish_reason: String,
    pub audio: Audio,
}

#[derive(Debug, Serialize, Deserialize)]
struct InputTokensDetails {
    pub text_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputTokensDetails {
    pub audio_tokens: i32,
    pub text_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Usage {
    pub input_tokens_details: InputTokensDetails,
    pub total_tokens: i32,
    pub output_tokens: i32,
    pub input_tokens: i32,
    pub output_tokens_details: OutputTokensDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    pub output: Output,
    pub usage: Usage,
    pub request_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn test_wav_result_one() {
        let wav_data = vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6D,
            0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x40, 0x1F, 0x00, 0x00,
            0x40, 0x1F, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00,
            0x00, 0x00,
        ];

        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mut first_body = String::from(
            r#"{"output":{"finish_reason":"stop","audio":{"expires_at":1751529865,"data":"","id":"audio_f51d7b99-deab-9785-8599-73705c774fa3",
        "url":""#,
        );
        first_body.push_str(&format!("{}/second", url));
        first_body.push_str(
        r#""
        }},"usage":{"input_tokens_details":{"text_tokens":19},"total_tokens":224,"output_tokens":205,"input_tokens":19,"output_tokens_details":{"audio_tokens":205,"text_tokens":0}},"request_id":"f51d7b99-deab-9785-8599-73705c774fa3"}"#);

        // Create a mock
        let _mock = server
            .mock("POST", "/first")
            .with_status(200)
            .with_header(header::CONTENT_TYPE, "application/json")
            .with_body(first_body)
            .create_async()
            .await;

        let _mock = server
            .mock("GET", "/second")
            .with_status(200)
            .with_header(header::CONTENT_TYPE, "application/json")
            .with_body(wav_data)
            .create_async()
            .await;

        let ali_tts = AliTTS::new("test_key".to_string()).with_url(format!("{}/first", url));
        let tts_files = ali_tts.tts(&vec!["测试".to_owned()]).await;

        assert!(tts_files.is_ok());
        let tts_files = tts_files.unwrap();

        assert_eq!(tts_files.len(), 1);
        assert_eq!(tts_files[0].text, "测试");
        assert!(tts_files[0].path.exists());

        _ = fs::remove_file(&tts_files[0].path);
    }

    #[tokio::test]
    async fn test_wav_result_two() {
        let wav_data = vec![
            0x52, 0x49, 0x46, 0x46, 0x24, 0x00, 0x00, 0x00, 0x57, 0x41, 0x56, 0x45, 0x66, 0x6D,
            0x74, 0x20, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x40, 0x1F, 0x00, 0x00,
            0x40, 0x1F, 0x00, 0x00, 0x01, 0x00, 0x08, 0x00, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00,
            0x00, 0x00,
        ];

        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mut first_body = String::from(
            r#"{"output":{"finish_reason":"stop","audio":{"expires_at":1751529865,"data":"","id":"audio_f51d7b99-deab-9785-8599-73705c774fa3",
        "url":""#,
        );
        first_body.push_str(&format!("{}/second", url));
        first_body.push_str(
        r#""
        }},"usage":{"input_tokens_details":{"text_tokens":19},"total_tokens":224,"output_tokens":205,"input_tokens":19,"output_tokens_details":{"audio_tokens":205,"text_tokens":0}},"request_id":"f51d7b99-deab-9785-8599-73705c774fa3"}"#);

        // Create a mock
        let _mock = server
            .mock("POST", "/first")
            .with_status(200)
            .with_header(header::CONTENT_TYPE, "application/json")
            .with_body(first_body)
            .create();

        let _mock = server
            .mock("GET", "/second")
            .with_status(200)
            .with_header(header::CONTENT_TYPE, "application/json")
            .with_body(wav_data)
            .create();

        let ali_tts = AliTTS::new("test_key".to_string()).with_url(format!("{}/first", url));
        let tts_files = ali_tts
            .tts(&vec!["测试".to_owned(), "测试".to_owned()])
            .await;

        assert!(tts_files.is_ok());
        let tts_files = tts_files.unwrap();

        assert_eq!(tts_files.len(), 2);
        assert_eq!(tts_files[0].text, "测试");
        assert!(tts_files[0].path.exists());
        _ = fs::remove_file(&tts_files[0].path);

        assert_eq!(tts_files[1].text, "测试");
        assert!(tts_files[1].path.exists());
        _ = fs::remove_file(&tts_files[1].path);
    }

    #[tokio::test]
    async fn test_invalid_key() {
        let ali_tts = AliTTS::new("INVALID ALI KEY".to_string());
        let tts_files = ali_tts.tts(&vec!["INVALID ALI KEY".to_owned()]).await;

        assert!(tts_files.is_err());
        let tts_files = tts_files.unwrap_err();
        match tts_files {
            TTSError::Network(v) => {
                assert!(v.contains("无效的阿里 API Key"));
            }
            _ => {
                assert!(false)
            }
        }
    }
}
