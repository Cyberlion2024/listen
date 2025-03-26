use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Topic {
    pub topic: String,
    pub title: String,
    pub topic_rank: Option<u32>,
    pub related_topics: Option<Vec<String>>,
    pub types_count: Option<HashMap<String, u32>>,
    pub types_interactions: Option<HashMap<String, u32>>,
    pub types_sentiment: Option<HashMap<String, u32>>,
    pub types_sentiment_detail: Option<HashMap<String, SentimentDetail>>,
    pub interactions_24h: Option<u64>,
    pub num_contributors: Option<u32>,
    pub num_posts: Option<u32>,
    pub categories: Option<Vec<String>>,
    pub trend: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SentimentDetail {
    pub positive: u32,
    pub neutral: u32,
    pub negative: u32,
}

impl Topic {
    pub fn get_sentiment_summary(&self) -> Option<String> {
        if let Some(sentiment) = &self.types_sentiment {
            let mut summary = String::from("Sentiment summary:\n");
            
            for (network, score) in sentiment {
                let sentiment_text = match score {
                    0..=40 => "very negative",
                    41..=45 => "negative",
                    46..=54 => "neutral",
                    55..=70 => "positive",
                    _ => "very positive",
                };
                
                summary.push_str(&format!("- {}: {} ({}%)\n", network, sentiment_text, score));
            }
            
            Some(summary)
        } else {
            None
        }
    }
    
    pub fn get_engagement_summary(&self) -> Option<String> {
        if let Some(interactions) = &self.types_interactions {
            let mut summary = String::from("Engagement summary:\n");
            
            for (network, count) in interactions {
                let engagement_text = match count {
                    0..=1_000 => "very low",
                    1_001..=10_000 => "low",
                    10_001..=100_000 => "medium",
                    100_001..=1_000_000 => "high",
                    _ => "very high",
                };
                
                summary.push_str(&format!("- {}: {} ({} interactions)\n", 
                    network, 
                    engagement_text,
                    count));
            }
            
            Some(summary)
        } else {
            None
        }
    }
    
    pub fn get_trend_summary(&self) -> String {
        match &self.trend {
            Some(trend) => match trend.as_str() {
                "up" => "The topic is trending upward with increasing engagement",
                "down" => "The topic is trending downward with decreasing engagement",
                "flat" => "The topic engagement is stable",
                _ => "Trend information is unavailable",
            }.to_string(),
            None => "Trend information is unavailable".to_string(),
        }
    }
} 