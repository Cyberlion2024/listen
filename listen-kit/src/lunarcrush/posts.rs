use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Post {
    pub id: String,
    pub post_type: String,
    pub post_title: String, 
    pub post_link: String,
    pub post_image: Option<String>,
    pub post_created: u64,
    pub post_sentiment: f64,
    pub creator_id: Option<String>,
    pub creator_name: String,
    pub creator_display_name: Option<String>,
    pub creator_followers: Option<u32>,
    pub creator_avatar: Option<String>,
    pub interactions_24h: u32,
    pub interactions_total: Option<u32>,
}

impl Post {
    pub fn get_sentiment_text(&self) -> &'static str {
        match self.post_sentiment as u32 {
            1 => "very negative",
            2 => "negative",
            3 => "neutral",
            4 => "positive",
            5 => "very positive",
            _ => {
                if self.post_sentiment < 2.0 {
                    "very negative"
                } else if self.post_sentiment < 3.0 {
                    "negative"
                } else if self.post_sentiment < 3.5 {
                    "neutral"
                } else if self.post_sentiment < 4.5 {
                    "positive"
                } else {
                    "very positive"
                }
            }
        }
    }
    
    pub fn get_date_formatted(&self) -> String {
        let dt = chrono::DateTime::from_timestamp(self.post_created as i64, 0)
            .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH);
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
    
    pub fn get_engagement_level(&self) -> &'static str {
        match self.interactions_24h {
            0..=10 => "very low",
            11..=100 => "low",
            101..=1000 => "moderate",
            1001..=10000 => "high",
            _ => "very high",
        }
    }
    
    pub fn get_network_name(&self) -> &'static str {
        match self.post_type.as_str() {
            "tweet" => "Twitter/X",
            "reddit-post" => "Reddit",
            "youtube-video" => "YouTube",
            "tiktok-video" => "TikTok",
            "news" => "News",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostsResponse {
    pub posts: Vec<Post>,
} 