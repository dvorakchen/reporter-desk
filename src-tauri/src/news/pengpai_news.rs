use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
};
use reqwest::Client;
use scraper::Selector;
use std::{cell::RefCell, sync::Mutex};

use crate::news::{
    MaterialExtractor, NewsCrawler, NewsMaterial, NewsMaterialError, NewsMaterialResult, NewsTitle,
};

pub const SOURCE_NAME: &str = "pengpai";

pub struct PengPaiNews {
    client: Client,
    hot_news_resp: Mutex<RefCell<Vec<PengPaiHotNews>>>,
}

impl PengPaiNews {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            hot_news_resp: Mutex::new(RefCell::new(vec![])),
        }
    }
}

#[async_trait::async_trait]
impl NewsCrawler for PengPaiNews {
    async fn get_hot_news_list(&self) -> Vec<super::NewsTitle> {
        let url = "https://cache.thepaper.cn/contentapi/wwwIndex/rightSidebar";

        let response = self.client.get(url).send().await.unwrap();
        let json: ResponseContent = response.json().await.unwrap();

        let resp_data = json.data.hot_news;

        {
            let mut lock = self.hot_news_resp.lock().unwrap();
            let list = lock.get_mut();
            *list = resp_data.clone();
        }

        resp_data
            .into_iter()
            .map(PengPaiHotNews::to_hot_news)
            .collect()
    }
}

#[derive(Debug, serde::Deserialize)]
struct ResponseContent {
    pub data: Data,
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    #[serde(rename = "hotNews")]
    pub hot_news: Vec<PengPaiHotNews>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct PengPaiHotNews {
    #[serde(rename = "contId")]
    pub cont_id: String,
    pub name: String,
    #[serde(rename = "pic")]
    pub cover: String,
    pub videos: Option<Video>,
}

impl PengPaiHotNews {
    pub fn to_hot_news(self) -> NewsTitle {
        let mut pics = vec![self.cover];
        let mut videos = vec![];

        if let Some(video) = self.videos {
            videos.push(video.url);
            pics.push(video.cover);
        }

        NewsTitle {
            source: SOURCE_NAME.to_owned(),
            title: self.name,
            url: format!(
                "https://www.thepaper.cn/newsDetail_forward_{}",
                self.cont_id
            ),
            pics,
            videos,
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
struct Video {
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "coverUrlFirstFrame")]
    pub cover: String,
}

pub struct PengPaiNewsMaterialExtractor {
    credentials: Credentials,
}

impl PengPaiNewsMaterialExtractor {
    pub fn from_deepseek(api_key: impl Into<String>) -> Self {
        Self {
            credentials: Credentials::new(api_key, "https://api.deepseek.com"),
        }
    }
}

#[async_trait::async_trait]
impl MaterialExtractor for PengPaiNewsMaterialExtractor {
    async fn get_material(&self, hot_news: &NewsTitle) -> NewsMaterialResult {
        let http = Client::new();
        let response = http
            .get(hot_news.url.clone())
            .send()
            .await
            .map_err(|e| NewsMaterialError::Net(e.to_string()))?;

        let raw_content = response
            .text()
            .await
            .map_err(|e| NewsMaterialError::Net(e.to_string()))?;

        let body = Self::get_body_inner_text(raw_content);

        let deepseek_result = self
            .ask_deepseek(&body)
            .await
            .map_err(|e| NewsMaterialError::AI(e.to_string()))?;

        let mut pics = hot_news.pics.clone();
        pics.extend(deepseek_result.images);

        Ok(NewsMaterial {
            title: hot_news.title.clone(),
            summary: deepseek_result.summary.clone(),
            videos: hot_news.videos.clone(),
            pics,
        })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
struct DeepSeekResult {
    pub summary: Vec<String>,
    pub images: Vec<String>,
}

impl PengPaiNewsMaterialExtractor {
    fn get_body_inner_text(raw_content: String) -> String {
        use scraper::Html;
        let document = Html::parse_document(&raw_content);
        let main = Selector::parse("main").unwrap();

        let mut mains = document.select(&main);
        let main = mains.next().unwrap();
        main.inner_html()
    }

    async fn ask_deepseek(&self, news_content: &str) -> Result<DeepSeekResult, String> {
        let messages = vec![
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some(Self::get_prompt().to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
            ChatCompletionMessage {
                role: ChatCompletionMessageRole::User,
                content: Some(news_content.to_string()),
                name: None,
                function_call: None,
                tool_call_id: None,
                tool_calls: None,
            },
        ];

        let chat_completion = ChatCompletion::builder("deepseek-chat", messages.clone())
            .credentials(self.credentials.clone())
            .create()
            .await
            .map_err(|e| {
                if &e.error_type == "authentication_error" {
                    "DeepSeek API Key 无效，请检查".to_owned()
                } else {
                    e.to_string()
                }
            })?;

        let returned_message = chat_completion.choices.first().unwrap().message.clone();

        let raw_json = returned_message
            .content
            .unwrap()
            .trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_end_matches("```")
            .trim()
            .to_string();

        log::debug!("raw_json: {}", raw_json);
        let res: DeepSeekResult = serde_json::from_str(&raw_json).unwrap();

        Ok(res)
    }

    fn get_prompt() -> &'static str {
        super::default_prompt()
    }
}
