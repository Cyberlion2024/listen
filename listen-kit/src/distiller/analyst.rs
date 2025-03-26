use crate::distiller::{
    deepseek::make_deepseek_analyst, gemini::make_gemini_analyst,
};
use anyhow::{anyhow, Result};

pub struct Analyst {
    pub twitter_agent: Option<Box<dyn TwitterAnalystAgent>>,
    pub lunarcrush_agent: Option<Box<dyn LunarCrushAnalystAgent>>,
    pub chart_agent: Option<Box<dyn ChartAnalystAgent>>,
    pub web_agent: Option<Box<dyn WebAnalystAgent>>,
    pub faster100x_agent: Option<Box<dyn Faster100xAnalystAgent>>,
    pub locale: String,
}

// Create a general error type for analysts
#[derive(Debug, thiserror::Error)]
pub enum AnalystError {
    #[error("API key is not set")]
    ApiKeyNotSet,

    #[error("Model error")]
    PromptError(rig::completion::PromptError),

    #[error("Serialization error")]
    SerializationError,

    #[error("Unsupported operation for this analyst type")]
    UnsupportedOperation,
}

// Common trait for all analyst types
#[async_trait::async_trait]
pub trait AnalystAgent: Send + Sync {
    fn locale(&self) -> &str;
    fn agent_type(&self) -> AnalystType;
}

// Enum to identify different analyst types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalystType {
    Twitter,
    LunarCrush,
    Chart,
    Web,
    Faster100x,
}

// Twitter analyst trait
#[async_trait::async_trait]
pub trait TwitterAnalystAgent: AnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

// LunarCrush analyst trait
#[async_trait::async_trait]
pub trait LunarCrushAnalystAgent: AnalystAgent {
    async fn analyze_lunarcrush(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

// Chart analyst trait
#[async_trait::async_trait]
pub trait ChartAnalystAgent: AnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

// Web analyst trait
#[async_trait::async_trait]
pub trait WebAnalystAgent: AnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

#[async_trait::async_trait]
pub trait Faster100xAnalystAgent: AnalystAgent {
    async fn analyze_faster100x(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError>;
}

impl Analyst {
    pub fn new(locale: String) -> Self {
        Self {
            twitter_agent: None,
            lunarcrush_agent: None,
            chart_agent: None,
            web_agent: None,
            faster100x_agent: None,
            locale,
        }
    }

    pub fn with_twitter_analyst(
        mut self,
        agent: Box<dyn TwitterAnalystAgent>,
    ) -> Self {
        self.twitter_agent = Some(agent);
        self
    }

    pub fn with_lunarcrush_analyst(
        mut self,
        agent: Box<dyn LunarCrushAnalystAgent>,
    ) -> Self {
        self.lunarcrush_agent = Some(agent);
        self
    }

    pub fn with_chart_analyst(
        mut self,
        agent: Box<dyn ChartAnalystAgent>,
    ) -> Self {
        self.chart_agent = Some(agent);
        self
    }

    pub fn with_web_analyst(
        mut self,
        agent: Box<dyn WebAnalystAgent>,
    ) -> Self {
        self.web_agent = Some(agent);
        self
    }

    pub fn with_faster100x_analyst(
        mut self,
        agent: Box<dyn Faster100xAnalystAgent>,
    ) -> Self {
        self.faster100x_agent = Some(agent);
        self
    }

    pub fn from_env_with_locale(locale: String) -> Result<Self> {
        let twitter_agent = make_deepseek_analyst(locale.clone(), AnalystType::Twitter)?;
        let lunarcrush_agent = make_deepseek_analyst(locale.clone(), AnalystType::LunarCrush)?;
        let chart_agent = make_deepseek_analyst(locale.clone(), AnalystType::Chart)?;
        let web_agent = make_deepseek_analyst(locale.clone(), AnalystType::Web)?;
        let faster100x_agent = make_deepseek_analyst(locale.clone(), AnalystType::Faster100x)?;

        Ok(Self::new(locale)
            .with_twitter_analyst(Box::new(twitter_agent))
            .with_lunarcrush_analyst(Box::new(lunarcrush_agent))
            .with_chart_analyst(Box::new(chart_agent))
            .with_web_analyst(Box::new(web_agent))
            .with_faster100x_analyst(Box::new(faster100x_agent)))
    }

    pub fn from_env_with_locale_and_provider(
        locale: String,
        provider: &str,
    ) -> Result<Self> {
        match provider {
            "deepseek" => {
                let twitter_agent = make_deepseek_analyst(locale.clone(), AnalystType::Twitter)?;
                let lunarcrush_agent = make_deepseek_analyst(locale.clone(), AnalystType::LunarCrush)?;
                let chart_agent = make_deepseek_analyst(locale.clone(), AnalystType::Chart)?;
                let web_agent = make_deepseek_analyst(locale.clone(), AnalystType::Web)?;
                let faster100x_agent = make_deepseek_analyst(locale.clone(), AnalystType::Faster100x)?;

                Ok(Self::new(locale)
                    .with_twitter_analyst(Box::new(twitter_agent))
                    .with_lunarcrush_analyst(Box::new(lunarcrush_agent))
                    .with_chart_analyst(Box::new(chart_agent))
                    .with_web_analyst(Box::new(web_agent))
                    .with_faster100x_analyst(Box::new(faster100x_agent)))
            }
            "gemini" => {
                let twitter_agent = make_gemini_analyst(locale.clone(), AnalystType::Twitter)?;
                let lunarcrush_agent = make_gemini_analyst(locale.clone(), AnalystType::LunarCrush)?;
                let chart_agent = make_gemini_analyst(locale.clone(), AnalystType::Chart)?;
                let web_agent = make_gemini_analyst(locale.clone(), AnalystType::Web)?;
                let faster100x_agent = make_gemini_analyst(locale.clone(), AnalystType::Faster100x)?;

                Ok(Self::new(locale)
                    .with_twitter_analyst(Box::new(twitter_agent))
                    .with_lunarcrush_analyst(Box::new(lunarcrush_agent))
                    .with_chart_analyst(Box::new(chart_agent))
                    .with_web_analyst(Box::new(web_agent))
                    .with_faster100x_analyst(Box::new(faster100x_agent)))
            }
            _ => Err(anyhow!("Unsupported provider: {}", provider)),
        }
    }
}
