pub mod ithome;
pub mod pengpai_news;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::director::source::SourceName;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewsTitle {
    pub source: SourceName,
    pub title: String,
    pub url: String,
    pub pics: Vec<String>,
    pub videos: Vec<String>,
}

impl NewsTitle {
    /// extracts the material from this news
    pub async fn get_news_material<T>(&self, material_extractor: &T) -> NewsMaterialResult
    where
        T: MaterialExtractor,
    {
        material_extractor.get_material(self).await
    }
}

pub type NewsMaterialResult = Result<NewsMaterial, NewsMaterialError>;

#[derive(Error, Debug)]
pub enum NewsMaterialError {
    #[error("网络错误: {0}")]
    Net(String),
    #[error("AI 错误: {0}")]
    AI(String),
}

pub struct NewsMaterial {
    pub title: String,
    pub summary: Vec<String>,
    pub pics: Vec<String>,
    pub videos: Vec<String>,
}

/// crawler, indicats how to get the news title list
#[async_trait::async_trait]
pub trait NewsCrawler {
    async fn get_hot_news_list(&self) -> Vec<NewsTitle>;
}

/// Extractor, responsible for extractor news material: news summary, pictures, videos.
#[async_trait::async_trait]
pub trait MaterialExtractor {
    async fn get_material(&self, hot_news: &NewsTitle) -> NewsMaterialResult;
}

fn default_prompt() -> &'static str {
    r#"
你是一个爆款短视频的作者，我会给你一个 HTML 格式的新闻稿，你要根据要求总结里面的新闻，并提取正文的图片，具体要求为：
1. 将新闻内容浓缩为200字内的短视频风格摘要，在200字以内，使用吸引眼球的短视频的风格夸张语气和俏皮。
风格夸张俏皮，喜欢使用网络热词和热梗，保持事实准确，突出核心事件、关键人物和戏剧性细节，纯文字输出，禁止使用表情符号。
时间地点人物等关键信息必须准确，注意中文标点符号使用规范。正文要根据逗号、句号分割，放在数组内，每一项不超过30字，尽量接近 30 字，如：

```json
["句子1", "句子2"]
```

2. 从新闻稿HTML中提取仅正文部分的图片链接（排除封面、视频缩略图、图标等非正文内容），并去除URL中的querystring参数。严格限定在正文内容区域，排除所有非正文图片（封面/视频缩略图/广告等），清除URL中?及后面的参数，结果以JSON数组格式返回，若无符合条件图片则返回空数组[]。如：

```json
[
        "https://imgpai.cn/newpai/image/175423202.jpg",
        "https://imgpai.cn/newpai/image/17540.jpg"
      ]
```
3. 新闻中如果有出现主要人物，人物需要出现在摘要中，但不宜过多
4. 摘要内容要流畅，不宜断句太频繁影响阅读

上面的要求按照 JSON 格式输出，如：

{
  "summary": ["句子1", "句子2"],
  "images": ["图片1", "图片2"]
}
        "#
}
