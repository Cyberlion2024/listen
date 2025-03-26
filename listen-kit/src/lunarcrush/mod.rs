use anyhow::Result;
use rig_tool_macro::tool;

pub mod client;
pub mod search;
pub mod topic;
pub mod posts;
pub mod creators;

// Re-export common types
pub use client::{LunarCrushApiClient, LunarCrushApiResponseError};
pub use topic::Topic;
pub use posts::Post;
pub use creators::Creator;

// Common types shared across modules
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub config: Option<ApiConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiConfig {
    pub topic: String,
    pub type_: Option<String>,
    pub id: String,
    pub generated: u64,
}

// LunarCrush API Implementation
pub struct LunarCrushApi {
    pub client: LunarCrushApiClient,
}

#[derive(Debug, thiserror::Error)]
pub enum LunarCrushApiError {
    #[error("[LunarCrushAPI] LunarCrush API Error: {0}")]
    ApiError(LunarCrushApiResponseError),

    #[error("[LunarCrushAPI] Failed to parse response: {0}")]
    ParseError(reqwest::Error),

    #[error("[LunarCrushAPI] Failed to deserialize response: {0}")]
    RequestError(reqwest::Error),

    #[error("[LunarCrushAPI] Deserialize error: {0} body: {1}")]
    DeserializeError(serde_json::Error, String),

    #[error("[LunarCrushAPI] Invalid input: {0}")]
    InvalidInput(anyhow::Error),
}

// Implementazione del tool utilizzando la macro #[tool]
#[tool(description = "
Research a cryptocurrency or blockchain topic using LunarCrush, which analyzes social media sentiment and activity.

Parameters:
- topic (string): The cryptocurrency/blockchain topic to research. For Solana tokens, provide the mint address directly for best results.

Returns information about:
- Topic overview and metrics
- Recent posts about the topic
- Influential creators discussing the topic
- Social engagement and sentiment analysis
")]
pub async fn analyze_topic(topic: String) -> Result<serde_json::Value> {
    let lunarcrush = LunarCrushApi::from_env()?;
    
    // Identifica se il topic sembra essere un indirizzo Solana (mint address)
    let is_mint_address = topic.len() > 30;
    
    // Se sembra un mint address, lo usiamo direttamente
    // Altrimenti, usiamo il topic originale
    let search_topic = if is_mint_address {
        tracing::info!("Usando mint address come parametro di ricerca LunarCrush: {}", topic);
        topic.clone()
    } else {
        tracing::info!("Usando nome token come parametro di ricerca LunarCrush: {}", topic);
        topic.clone()
    };
    
    let result = lunarcrush.research_topic(&search_topic)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to research topic: {}", e))?;
    
    Ok(result)
}

impl LunarCrushApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: LunarCrushApiClient::new(api_key, None),
        }
    }

    pub fn from_env() -> Result<Self> {
        let client = LunarCrushApiClient::new(
            std::env::var("LUNARCRUSH_API_KEY").unwrap(),
            Some("https://lunarcrush.com/api4/public".to_string()),
        );
        Ok(Self { client })
    }

    pub async fn research_topic(
        &self,
        topic: &str,
    ) -> Result<serde_json::Value, LunarCrushApiError> {
        // Verificare se il topic sembra essere un indirizzo Solana o di qualsiasi token
        let is_address = topic.len() > 30;
        
        // Elaborazione del parametro di ricerca finale
        let search_topic = if is_address {
            // È già un indirizzo completo, lo usiamo direttamente
            topic.to_string()
        } else {
            // È un nome di token o un altro identificativo
            topic.to_string()
        };
        
        tracing::info!("Ricerca LunarCrush per topic/indirizzo: {}", search_topic);
        
        let topic_info = match self.fetch_topic_info(&search_topic).await {
            Ok(info) => info,
            Err(e) => {
                // Se non troviamo informazioni con l'indirizzo, creiamo una risposta minimale
                if is_address || topic.contains("FeR8") || topic.contains("pump") {
                    tracing::warn!("Token non riconosciuto: {}, creando risposta minima", topic);
                    let mut topic_obj = Topic::default();
                    topic_obj.topic = topic.to_string();
                    topic_obj.title = format!("{} (token)", topic);
                    topic_obj.types_count = Some(std::collections::HashMap::new());
                    topic_obj.types_interactions = Some(std::collections::HashMap::new());
                    topic_obj.types_sentiment = Some(std::collections::HashMap::new());
                    topic_obj.types_sentiment_detail = Some(std::collections::HashMap::new());
                    topic_obj.num_contributors = Some(0);
                    topic_obj.num_posts = Some(0);
                    topic_obj.categories = Some(vec!["solana".to_string(), "token".to_string()]);
                    topic_obj
                } else {
                    return Err(e);
                }
            }
        };
        
        let posts = match self.fetch_topic_posts(&search_topic).await {
            Ok(p) => p,
            Err(_) => ApiResponse { 
                data: vec![], 
                config: None 
            }
        };
        
        let creators = match self.fetch_topic_creators(&search_topic).await {
            Ok(c) => c,
            Err(_) => ApiResponse { 
                data: vec![], 
                config: None 
            }
        };

        if std::env::var("RUST_LOG").unwrap_or_default() == "debug" {
            // Creare la directory debug se non esiste
            let _ = std::fs::create_dir_all("debug");
            
            let _ = std::fs::write(
                "debug/lunarcrush_topic.json",
                serde_json::to_string(&topic_info).unwrap(),
            );
            let _ = std::fs::write(
                "debug/lunarcrush_posts.json",
                serde_json::to_string(&posts.data).unwrap(),
            );
            let _ = std::fs::write(
                "debug/lunarcrush_creators.json",
                serde_json::to_string(&creators.data).unwrap(),
            );
        }

        let res = serde_json::json!({
            "topic": topic_info,
            "posts": posts.data,
            "creators": creators.data,
        });

        Ok(res)
    }

    pub async fn fetch_topic_info(&self, topic: &str) -> Result<Topic, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/v1", topic);
        let response = self.client.request::<ApiResponse<Topic>>(&endpoint, None).await?;
        Ok(response.data)
    }

    pub async fn fetch_topic_posts(&self, topic: &str) -> Result<ApiResponse<Vec<Post>>, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/posts/v1", topic);
        let response = self.client.request::<ApiResponse<Vec<Post>>>(&endpoint, None).await?;
        Ok(response)
    }

    pub async fn fetch_topic_creators(&self, topic: &str) -> Result<ApiResponse<Vec<Creator>>, LunarCrushApiError> {
        let endpoint = format!("/topic/{}/creators/v1", topic);
        let response = self.client.request::<ApiResponse<Vec<Creator>>>(&endpoint, None).await?;
        Ok(response)
    }
}

// Summary types for combined data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostSummary {
    pub id: String,
    pub post_type: String,
    pub post_title: String,
    pub post_link: String, 
    pub post_image: Option<String>,
    pub post_created: u64,
    pub post_sentiment: f64,
    pub creator_name: String,
    pub creator_display_name: Option<String>,
    pub creator_followers: u32,
    pub creator_avatar: Option<String>,
    pub interactions_24h: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TopicResearch {
    pub topic: Topic,
    pub posts: Vec<PostSummary>,
    pub creators: Vec<Creator>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[timed::timed]
    #[tokio::test]
    async fn lunarcrush_e2e_bitcoin() {
        let lunarcrush = LunarCrushApi::from_env().unwrap();
        let summary = lunarcrush.research_topic("bitcoin").await.unwrap();

        println!("{:#?}", summary);
    }
} 