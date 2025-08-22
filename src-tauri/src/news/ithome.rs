use openai::{
    Credentials,
    chat::{ChatCompletion, ChatCompletionMessage, ChatCompletionMessageRole},
};
use reqwest::Client;
use scraper::Selector;

use crate::news::{
    MaterialExtractor, NewsCrawler, NewsMaterial, NewsMaterialError, NewsMaterialResult, NewsTitle,
};

pub const SOURCE_NAME: &str = "ithome";

pub struct ITHomeNews {
    client: Client,
}

impl ITHomeNews {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl NewsCrawler for ITHomeNews {
    async fn get_hot_news_list(&self) -> Vec<super::NewsTitle> {
        let url = "https://www.ithome.com/block/rank.html";

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "accept-language",
            "en,zh-CN;q=0.9,zh;q=0.8".parse().unwrap(),
        );
        headers.insert("cache-control", "no-cache".parse().unwrap());
        headers.insert("pragma", "no-cache".parse().unwrap());
        headers.insert("priority", "u=0, i".parse().unwrap());
        headers.insert(
            "sec-ch-ua",
            "\"Google Chrome\";v=\"131\", \"Chromium\";v=\"131\", \"Not_A Brand\";v=\"24\""
                .parse()
                .unwrap(),
        );
        headers.insert("sec-ch-ua-mobile", ".unwrap()0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", "\"macOS\"".parse().unwrap());
        headers.insert("sec-fetch-dest", "document".parse().unwrap());
        headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
        headers.insert("sec-fetch-site", "none".parse().unwrap());
        headers.insert("sec-fetch-user", ".unwrap()1".parse().unwrap());
        headers.insert("upgrade-insecure-requests", "1".parse().unwrap());
        headers.insert("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".parse().unwrap());

        let response = self.client.get(url).headers(headers).send().await.unwrap();
        let raw_html = response.text().await.unwrap();

        let html = scraper::Html::parse_fragment(&raw_html);

        let id_1 = if let Some(id_1) = html.select(&Selector::parse("#d-1").unwrap()).nth(0) {
            id_1
        } else {
            return vec![];
        };

        let mut list = vec![];
        for a_tag in id_1.select(&Selector::parse("a").unwrap()) {
            let title = a_tag.attr("title").unwrap_or("");
            let url = a_tag.attr("href").unwrap_or("");
            if title.is_empty() || url.is_empty() {
                continue;
            }

            list.push(NewsTitle {
                source: SOURCE_NAME.to_owned(),
                title: title.to_owned(),
                url: url.to_owned(),
                pics: vec![],
                videos: vec![],
            });
        }

        list
    }
}

pub struct ITHomeNewsMaterialExtractor {
    credentials: Credentials,
}

impl ITHomeNewsMaterialExtractor {
    pub fn from_deepseek(api_key: impl Into<String>) -> Self {
        Self {
            credentials: Credentials::new(api_key, "https://api.deepseek.com"),
        }
    }
}

#[async_trait::async_trait]
impl MaterialExtractor for ITHomeNewsMaterialExtractor {
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

        Ok(NewsMaterial {
            title: hot_news.title.clone(),
            summary: deepseek_result.summary,
            videos: hot_news.videos.clone(),
            pics: deepseek_result.images,
        })
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
struct DeepSeekResult {
    pub summary: Vec<String>,
    pub images: Vec<String>,
}

impl ITHomeNewsMaterialExtractor {
    fn get_body_inner_text(raw_content: String) -> String {
        use scraper::Html;
        let document = Html::parse_document(&raw_content);
        let main = Selector::parse("#paragraph").unwrap();

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
