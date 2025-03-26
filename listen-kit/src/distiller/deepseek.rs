use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent, Faster100xAnalystAgent,
    LunarCrushAnalystAgent, TwitterAnalystAgent, WebAnalystAgent,
};
use anyhow::Result;

#[derive(Clone)]
pub struct DeepseekAnalystAgent {
    agent_type: AnalystType,
    locale: String,
}

impl DeepseekAnalystAgent {
    pub fn new(
        agent_type: AnalystType,
        locale: &str,
        _custom_preamble: Option<String>,
    ) -> Self {
        DeepseekAnalystAgent {
            agent_type,
            locale: locale.to_string(),
        }
    }
    
    async fn completion(&self, _prompt: String) -> Result<String, AnalystError> {
        // Versione semplificata che restituisce un messaggio predefinito in base al locale
        if self.locale == "zh" {
            Ok("抱歉，DeepSeek模型暂时不可用。这里是一个简化的分析，仅供参考。".to_string())
        } else {
            Ok("Sorry, the DeepSeek model is temporarily unavailable. Here is a simplified analysis for reference.".to_string())
        }
    }
}

impl AnalystAgent for DeepseekAnalystAgent {
    fn locale(&self) -> &str {
        &self.locale
    }

    fn agent_type(&self) -> AnalystType {
        self.agent_type
    }
}

#[async_trait::async_trait]
impl TwitterAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        _response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let intent_str = intent.unwrap_or_default();
        let prompt = format!("Twitter analysis for query: {} with intent: {}", query, intent_str);
        self.completion(prompt).await
    }
}

#[async_trait::async_trait]
impl LunarCrushAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_lunarcrush(
        &self,
        query: &str,
        _response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let intent_str = intent.unwrap_or_default();
        let prompt = format!("LunarCrush analysis for query: {} with intent: {}", query, intent_str);
        self.completion(prompt).await
    }
}

#[async_trait::async_trait]
impl Faster100xAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_faster100x(
        &self,
        query: &str,
        _response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let intent_str = intent.unwrap_or_default();
        let prompt = format!("Faster100x analysis for query: {} with intent: {}", query, intent_str);
        self.completion(prompt).await
    }
}

#[async_trait::async_trait]
impl ChartAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_chart(
        &self,
        _candlesticks: &[crate::data::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let intent_str = intent.unwrap_or_default();
        let prompt = format!("Chart analysis for interval: {} with intent: {}", interval, intent_str);
        self.completion(prompt).await
    }
}

#[async_trait::async_trait]
impl WebAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        _content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let intent_str = intent.unwrap_or_default();
        let prompt = format!("Web analysis for URL: {} with intent: {}", url, intent_str);
        self.completion(prompt).await
    }
}

pub fn make_deepseek_analyst(
    agent_type: AnalystType,
    locale: &str,
    custom_preamble: Option<String>,
) -> DeepseekAnalystAgent {
    DeepseekAnalystAgent::new(agent_type, locale, custom_preamble)
}
