use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent,
    LunarCrushAnalystAgent, TwitterAnalystAgent, WebAnalystAgent,
    Faster100xAnalystAgent,
};
use super::preambles;
use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::anthropic::completion::CompletionModel;
use crate::common::claude_agent_builder;
pub type DeepseekAgent = rig::agent::Agent<CompletionModel>;

pub struct DeepseekAnalystAgent {
    pub agent: DeepseekAgent,
    pub locale: String,
    pub analyst_type: AnalystType,
}

// Implementazione manuale di Clone
impl Clone for DeepseekAnalystAgent {
    fn clone(&self) -> Self {
        // Creiamo un nuovo agente con le stesse configurazioni
        let agent = claude_agent_builder()
            .preamble(&self.preamble())
            .build();

        DeepseekAnalystAgent {
            agent,
            locale: self.locale.clone(),
            analyst_type: self.analyst_type,
        }
    }
}

impl DeepseekAnalystAgent {
    pub fn preamble(&self) -> String {
        match (self.analyst_type, self.locale.as_str()) {
            (AnalystType::Twitter, "zh") => preambles::TWITTER_ZH.to_string(),
            (AnalystType::Twitter, _) => preambles::TWITTER_EN.to_string(),
            (AnalystType::LunarCrush, "zh") => preambles::LUNARCRUSH_ZH.to_string(),
            (AnalystType::LunarCrush, _) => preambles::LUNARCRUSH_EN.to_string(),
            (AnalystType::Chart, "zh") => preambles::CHART_ZH.to_string(),
            (AnalystType::Chart, _) => preambles::CHART_EN.to_string(),
            (AnalystType::Web, "zh") => preambles::WEB_ZH.to_string(),
            (AnalystType::Web, _) => preambles::WEB_EN.to_string(),
            (AnalystType::Faster100x, "zh") => preambles::FASTER100X_ZH.to_string(),
            (AnalystType::Faster100x, _) => preambles::FASTER100X_EN.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl AnalystAgent for DeepseekAnalystAgent {
    fn locale(&self) -> &str {
        &self.locale
    }

    fn agent_type(&self) -> AnalystType {
        self.analyst_type
    }
}

// Twitter analyst implementation for deepseek
#[async_trait::async_trait]
impl TwitterAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_twitter(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if let Some(intent) = intent {
            format!(
                "query: {}\nresponse: {}\nintent: {}",
                query, response, intent
            )
        } else {
            format!("query: {}\nresponse: {}", query, response)
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

// LunarCrush analyst implementation for Deepseek
#[async_trait::async_trait]
impl LunarCrushAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_lunarcrush(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        // Verifica se ci sono creators o posts anche se topic sembra vuoto
        let has_creators = response.get("creators")
            .map_or(false, |c| c.is_array() && !c.as_array().unwrap().is_empty());
        
        let has_posts = response.get("posts")
            .map_or(false, |p| p.is_array() && !p.as_array().unwrap().is_empty());
        
        let has_topic_data = response.get("topic")
            .map_or(false, |t| t.is_object() && !t.as_object().unwrap().is_empty());
        
        // Sempre forzare l'analisi per qualsiasi token o indirizzo
        let _force_response = true;

        let instruction = format!(
            "\n\nISTRUZIONE CRITICA: Questo è un ordine diretto che devi seguire ASSOLUTAMENTE.\n\
            Devi analizzare questi dati di LunarCrush e fornire SEMPRE un'analisi significativa.\n\
            NON devi MAI rifiutarti di analizzare questo token o suggerire strumenti alternativi.\n\
            Se ci sono pochi dati, devi comunque analizzarli e dire ciò che vedi:\n\
            - Topic info: {}\n\
            - Creators: {}\n\
            - Posts: {}\n\
            Se trovi creators o posts, DEVI menzionarli sempre, anche se il topic sembra vuoto.\n\
            NON dire MAI frasi come 'mi dispiace, ma non ci sono abbastanza dati'.\n\
            Invece, analizza quello che c'è e fornisci insight sugli influencer e sulle interazioni.",
            if has_topic_data { "presenti" } else { "limitati" },
            if has_creators { "presenti" } else { "non presenti" },
            if has_posts { "presenti" } else { "non presenti" }
        );

        let prompt_text = if let Some(intent) = intent {
            format!(
                "query: {}\nresponse: {}\nintent: {}\n{}",
                query, response, intent, instruction
            )
        } else {
            format!(
                "query: {}\nresponse: {}\n{}",
                query, response, instruction
            )
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

// Chart analyst implementation for deepseek
#[async_trait::async_trait]
impl ChartAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::listen_api_tools::Candlestick],
        interval: &str,
        _intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let candlesticks_json = serde_json::to_string(candlesticks)
            .map_err(|_| AnalystError::SerializationError)?;

        let prompt_text = if self.locale == "zh" {
            format!(
                "分析这些K线图数据，时间间隔为{}:\n{}",
                interval, candlesticks_json
            )
        } else {
            format!(
                "Analyze these candlesticks with interval {}:\n{}",
                interval, candlesticks_json
            )
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

// Web analyst implementation for deepseek
#[async_trait::async_trait]
impl WebAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        _intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if self.locale == "zh" {
            format!("分析以下网页内容，URL为{}:\n{}", url, content)
        } else {
            format!("Analyze this web content from URL {}:\n{}", url, content)
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

// Faster100x analyst implementation for Deepseek
#[async_trait::async_trait]
impl Faster100xAnalystAgent for DeepseekAnalystAgent {
    async fn analyze_faster100x(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if let Some(intent) = intent {
            format!(
                "query: {}\nresponse: {}\nintent: {}",
                query, response, intent
            )
        } else {
            format!("query: {}\nresponse: {}", query, response)
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

// Factory method similar to gemini
pub fn make_deepseek_analyst(
    locale: String,
    analyst_type: AnalystType,
) -> Result<DeepseekAnalystAgent> {
    let agent = claude_agent_builder()
        .preamble(match locale.as_str() {
            "zh" => match analyst_type {
                AnalystType::LunarCrush => preambles::LUNARCRUSH_ZH,
                AnalystType::Faster100x => preambles::FASTER100X_ZH,
                _ => preambles::PREAMBLE_ZH,
            },
            _ => match analyst_type {
                AnalystType::LunarCrush => preambles::LUNARCRUSH_EN,
                AnalystType::Faster100x => preambles::FASTER100X_EN,
                _ => preambles::PREAMBLE_EN,
            },
        })
        .build();

    Ok(DeepseekAnalystAgent {
        agent,
        locale,
        analyst_type,
    })
}
