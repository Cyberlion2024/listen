use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::gemini::completion::CompletionModel as GeminiCompletionModel;
pub type GeminiAgent = rig::agent::Agent<GeminiCompletionModel>;

use super::analyst::{
    AnalystAgent, AnalystError, AnalystType, ChartAnalystAgent,
    LunarCrushAnalystAgent, TwitterAnalystAgent, WebAnalystAgent, Faster100xAnalystAgent,
};
use super::preambles;

pub struct GeminiAnalystAgent {
    pub agent: GeminiAgent,
    pub locale: String,
    pub analyst_type: AnalystType,
}

#[async_trait::async_trait]
impl AnalystAgent for GeminiAnalystAgent {
    fn locale(&self) -> &str {
        &self.locale
    }

    fn agent_type(&self) -> AnalystType {
        self.analyst_type
    }
}

// Twitter analyst implementation for Gemini
#[async_trait::async_trait]
impl TwitterAnalystAgent for GeminiAnalystAgent {
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

// LunarCrush analyst implementation for Gemini
#[async_trait::async_trait]
impl LunarCrushAnalystAgent for GeminiAnalystAgent {
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

// Faster100x analyst implementation for Gemini
#[async_trait::async_trait]
impl Faster100xAnalystAgent for GeminiAnalystAgent {
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

// Chart analyst implementation for Gemini
#[async_trait::async_trait]
impl ChartAnalystAgent for GeminiAnalystAgent {
    async fn analyze_chart(
        &self,
        candlesticks: &[crate::data::listen_api_tools::Candlestick],
        interval: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let candlesticks_json = serde_json::to_string(candlesticks)
            .map_err(|_| AnalystError::SerializationError)?;

        let prompt_text = if self.locale == "zh" {
            let base = format!(
                "分析这些K线图数据，时间间隔为{}:\n{}",
                interval, candlesticks_json
            );
            if let Some(intent) = intent {
                format!("{}意图是{}", base, intent)
            } else {
                base
            }
        } else {
            let base = format!(
                "Analyze these candlesticks with interval {}:\n{}",
                interval, candlesticks_json
            );
            if let Some(intent) = intent {
                format!("{}Intent is: {}", base, intent)
            } else {
                base
            }
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}
// Web analyst implementation for Gemini
#[async_trait::async_trait]
impl WebAnalystAgent for GeminiAnalystAgent {
    async fn analyze_web(
        &self,
        url: &str,
        content: &str,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        let prompt_text = if self.locale == "zh" {
            let base =
                format!("分析以下网页内容，URL为{}:\n{}", url, content);
            if let Some(intent) = intent {
                format!("{}意图是{}", base, intent)
            } else {
                base
            }
        } else {
            let base = format!(
                "Analyze this web content from URL {}:\n{}",
                url, content
            );
            if let Some(intent) = intent {
                format!("{}Intent is: {}", base, intent)
            } else {
                base
            }
        };

        self.agent
            .prompt(prompt_text)
            .await
            .map_err(AnalystError::PromptError)
    }
}

pub fn make_gemini_analyst(
    analyst_type: AnalystType,
    locale: &str,
    preamble: Option<String>,
) -> GeminiAnalystAgent {
    let default_preamble = match (analyst_type, locale) {
        (AnalystType::Twitter, "zh") => preambles::TWITTER_ZH,
        (AnalystType::Twitter, _) => preambles::TWITTER_EN,
        (AnalystType::LunarCrush, "zh") => preambles::LUNARCRUSH_ZH,
        (AnalystType::LunarCrush, _) => preambles::LUNARCRUSH_EN,
        (AnalystType::Faster100x, "zh") => preambles::FASTER100X_ZH,
        (AnalystType::Faster100x, _) => preambles::FASTER100X_EN,
        (AnalystType::Chart, "zh") => preambles::CHART_ZH,
        (AnalystType::Chart, _) => preambles::CHART_EN,
        (AnalystType::Web, "zh") => preambles::WEB_ZH,
        (AnalystType::Web, _) => preambles::WEB_EN,
    };

    let agent = rig::providers::gemini::Client::from_env()
        .agent(rig::providers::gemini::completion::GEMINI_2_0_FLASH)
        .preamble(&preamble.unwrap_or(default_preamble.to_string()))
        .build();

    GeminiAnalystAgent {
        agent,
        locale: locale.to_string(),
        analyst_type,
    }
}
