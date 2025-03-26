export interface Topic {
  topic: string;
  title: string;
  topic_rank?: number;
  related_topics?: string[];
  types_count?: Record<string, number>;
  types_interactions?: Record<string, number>;
  types_sentiment?: Record<string, number>;
  types_sentiment_detail?: Record<string, SentimentDetail>;
  interactions_24h?: number;
  num_contributors?: number;
  num_posts?: number;
  categories?: string[];
  trend?: string;
}

export interface SentimentDetail {
  positive: number;
  neutral: number;
  negative: number;
}

export interface Post {
  id: string;
  post_type: string;
  post_title: string;
  post_link: string;
  post_image?: string;
  post_created: number;
  post_sentiment: number;
  creator_id?: string;
  creator_name: string;
  creator_display_name?: string;
  creator_followers?: number;
  creator_avatar?: string;
  interactions_24h: number;
  interactions_total?: number;
}

export interface Creator {
  creator_id: string;
  creator_name: string;
  creator_avatar?: string;
  creator_followers: number;
  creator_rank: number;
  interactions_24h: number;
}

export interface LunarCrushResponse {
  topic: Topic;
  posts: Post[];
  creators: Creator[];
} 