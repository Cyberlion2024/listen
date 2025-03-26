use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Creator {
    pub creator_id: String,
    pub creator_name: String,
    pub creator_avatar: Option<String>,
    pub creator_followers: u32,
    pub creator_rank: u32,
    pub interactions_24h: u32,
}

impl Creator {
    pub fn get_network(&self) -> &'static str {
        if let Some(network) = self.creator_id.split("::").next() {
            match network {
                "twitter" => "Twitter/X",
                "reddit" => "Reddit",
                "youtube" => "YouTube",
                "tiktok" => "TikTok",
                _ => "Unknown",
            }
        } else {
            "Unknown"
        }
    }
    
    pub fn get_influence_level(&self) -> &'static str {
        match self.creator_followers {
            0..=1_000 => "micro influencer",
            1_001..=10_000 => "small influencer",
            10_001..=100_000 => "medium influencer",
            100_001..=1_000_000 => "large influencer",
            _ => "celebrity",
        }
    }
    
    pub fn get_engagement_level(&self) -> &'static str {
        match self.interactions_24h {
            0..=100 => "very low",
            101..=1_000 => "low",
            1_001..=10_000 => "moderate",
            10_001..=100_000 => "high",
            _ => "very high",
        }
    }
    
    pub fn get_unique_id(&self) -> Option<String> {
        self.creator_id.split("::").nth(1).map(String::from)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatorsResponse {
    pub creators: Vec<Creator>,
} 