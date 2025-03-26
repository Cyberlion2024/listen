import { useState } from "react";
import i18n from "../i18n";
import { Post, Creator, Topic } from "../types/lunarcrush.ts";

const formatNumber = (num: number): string => {
  if (num >= 1000000) {
    return (num / 1000000).toFixed(1) + "M";
  } else if (num >= 1000) {
    return (num / 1000).toFixed(1) + "K";
  } else {
    return num.toString();
  }
};

const getBackgroundSentimentColor = (sentiment: number): string => {
  if (sentiment < 40) return "bg-red-100";
  if (sentiment < 45) return "bg-orange-100";
  if (sentiment < 55) return "bg-gray-100";
  if (sentiment < 70) return "bg-green-100";
  return "bg-green-100";
};

const getSentimentText = (sentiment: number): string => {
  if (sentiment < 40) return i18n.t("very negative");
  if (sentiment < 45) return i18n.t("negative");
  if (sentiment < 55) return i18n.t("neutral");
  if (sentiment < 70) return i18n.t("positive");
  return i18n.t("very positive");
};

const getEngagementText = (engagement: number): string => {
  if (engagement < 1000) return i18n.t("very low");
  if (engagement < 10000) return i18n.t("low");
  if (engagement < 100000) return i18n.t("medium");
  if (engagement < 1000000) return i18n.t("high");
  return i18n.t("very high");
};

// Format a network name to be more user-friendly
const formatNetworkName = (network: string): string => {
  switch (network) {
    case "tweet": return "Twitter/X";
    case "reddit-post": return "Reddit";
    case "youtube-video": return "YouTube";
    case "tiktok-video": return "TikTok";
    default: return network.replace("-", " ").replace(/\b\w/g, l => l.toUpperCase());
  }
};

interface LunarCrushPostProps {
  post: Post;
}

export function LunarCrushPost({ post }: LunarCrushPostProps) {
  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
  };

  const getNetworkIcon = (postType: string) => {
    switch (postType) {
      case "tweet":
        return "X";
      case "reddit-post":
        return "Reddit";
      case "youtube-video":
        return "YouTube";
      case "tiktok-video":
        return "TikTok";
      case "news":
        return "News";
      default:
        return postType;
    }
  };

  return (
    <div className="border-2 border-gray-300 rounded-lg p-4 mb-4 hover:shadow-md transition-shadow bg-white">
      <div className="flex items-start mb-3">
        <a
          href={post.post_link}
          target="_blank"
          rel="noopener noreferrer"
          className="flex-shrink-0"
        >
          {post.creator_avatar ? (
            <img
              src={post.creator_avatar}
              alt={post.creator_name}
              className="w-12 h-12 rounded-full mr-3 border-2 border-gray-300"
            />
          ) : (
            <div className="w-12 h-12 rounded-full mr-3 bg-gray-200 flex items-center justify-center border-2 border-gray-300">
              <span className="text-black text-xl font-bold">
                {post.creator_name.charAt(0).toUpperCase()}
              </span>
            </div>
          )}
        </a>
        <div>
          <div className="flex items-center">
            <span className="font-bold text-black">
              {post.creator_display_name || post.creator_name}
            </span>
            <span className="text-black ml-2">@{post.creator_name}</span>
            <span className="ml-2 px-2 py-1 rounded text-xs bg-blue-200 text-black font-bold border border-blue-300">
              {getNetworkIcon(post.post_type)}
            </span>
          </div>
          <div className="text-black text-sm">
            {formatDate(post.post_created)}
          </div>
        </div>
      </div>

      <div className="mb-3">
        <p className="text-black font-medium">{post.post_title}</p>
        {post.post_image && (
          <img
            src={post.post_image}
            alt="Post content"
            className="mt-2 rounded-lg w-full border border-gray-300"
          />
        )}
      </div>

      <div className="flex justify-between text-sm border-t pt-2">
        <div className="flex items-center">
          <span className="mr-2 text-black font-bold">{i18n.t("Sentiment")}: </span>
          <span className={`px-2 py-1 rounded ${getBackgroundSentimentColor(post.post_sentiment * 20)} text-black font-bold border border-gray-300`}>
            {getSentimentText(post.post_sentiment * 20)} ({(post.post_sentiment * 20).toFixed(0)}%)
          </span>
        </div>
        <div className="flex items-center">
          <span className="mr-2 text-black font-bold">{i18n.t("Interactions")}: </span>
          <span className="font-bold text-black">
            {formatNumber(post.interactions_24h)}
          </span>
        </div>
      </div>
    </div>
  );
}

interface LunarCrushCreatorProps {
  creator: Creator;
}

export function LunarCrushCreator({ creator }: LunarCrushCreatorProps) {
  const getNetworkName = (creatorId: string) => {
    if (creatorId.startsWith("twitter::")) return "Twitter/X";
    if (creatorId.startsWith("reddit::")) return "Reddit";
    if (creatorId.startsWith("youtube::")) return "YouTube";
    if (creatorId.startsWith("tiktok::")) return "TikTok";
    return "Unknown";
  };

  const getNetworkColor = (creatorId: string): string => {
    if (creatorId.startsWith("twitter::")) return "bg-blue-200 text-black border border-blue-300";
    if (creatorId.startsWith("reddit::")) return "bg-orange-200 text-black border border-orange-300";
    if (creatorId.startsWith("youtube::")) return "bg-red-200 text-black border border-red-300";
    if (creatorId.startsWith("tiktok::")) return "bg-purple-200 text-black border border-purple-300";
    return "bg-gray-200 text-black border border-gray-300";
  };

  return (
    <div className="flex items-center p-4 border-2 border-gray-300 rounded-lg mb-3 hover:shadow-md transition-shadow bg-white">
      {creator.creator_avatar ? (
        <img
          src={creator.creator_avatar}
          alt={creator.creator_name}
          className="w-12 h-12 rounded-full mr-4 border-2 border-gray-300"
        />
      ) : (
        <div className="w-12 h-12 rounded-full mr-4 bg-gray-200 flex items-center justify-center border-2 border-gray-300">
          <span className="text-black text-xl font-bold">
            {creator.creator_name.charAt(0).toUpperCase()}
          </span>
        </div>
      )}
      <div className="flex-grow">
        <div className="flex justify-between items-center">
          <span className="font-bold text-black">{creator.creator_name}</span>
          <span className={`px-2 py-1 rounded text-xs font-bold ${getNetworkColor(creator.creator_id)}`}>
            {getNetworkName(creator.creator_id)}
          </span>
        </div>
        <div className="flex justify-between text-sm mt-1">
          <div className="flex items-center">
            <span className="text-black font-bold mr-1">{i18n.t("Followers")}:</span>
            <span className="font-bold text-black">{formatNumber(creator.creator_followers)}</span>
          </div>
          <div className="flex items-center">
            <span className="text-black font-bold mr-1">{i18n.t("Interactions 24h")}:</span>
            <span className="font-bold text-black">
              {formatNumber(creator.interactions_24h)}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

interface LunarCrushDisplayProps {
  topicData: {
    topic: Topic;
    posts: Post[];
    creators: Creator[];
  };
}

export default function LunarCrushDisplay({ topicData }: LunarCrushDisplayProps) {
  const [activeTab, setActiveTab] = useState<"overview" | "posts" | "creators">("overview");
  
  if (!topicData) return <div>{i18n.t("No data available")}</div>;

  const { topic, posts, creators } = topicData;

  // Determina il rank numerico o una descrizione se non disponibile
  const getTopicRank = () => {
    if (!topic.topic_rank) {
      return topic.categories && topic.categories.length > 0 
        ? "Nuovo token" 
        : "Non classificato";
    }
    return topic.topic_rank;
  };

  const renderOverview = () => (
    <div className="space-y-6">
      <div className="bg-gradient-to-r from-blue-50 to-indigo-50 p-6 rounded-lg shadow-sm">
        <h2 className="text-2xl font-bold mb-4 text-black">{topic.title}</h2>
        <div className="grid grid-cols-2 gap-6">
          <div className="bg-white p-4 rounded-lg shadow-sm">
            <p className="text-black flex justify-between">
              <span className="font-medium">{i18n.t("Topic Rank")}:</span> 
              <span className="font-semibold text-black">{getTopicRank()}</span>
            </p>
            <p className="text-black flex justify-between mt-2">
              <span className="font-medium">{i18n.t("Trend")}:</span>
              <span className={
                topic.trend === "up" 
                  ? "text-green-900 font-semibold" 
                  : topic.trend === "down" 
                    ? "text-red-900 font-semibold" 
                    : "text-gray-900 font-semibold"
              }>
                {topic.trend === "up" 
                  ? i18n.t("Upward") 
                  : topic.trend === "down" 
                    ? i18n.t("Downward") 
                    : i18n.t("Flat")}
              </span>
            </p>
          </div>
          <div className="bg-white p-4 rounded-lg shadow-sm">
            <p className="text-black flex justify-between">
              <span className="font-medium">{i18n.t("Posts 24h")}:</span>
              <span className="font-semibold text-black">
                {formatNumber(topic.num_posts || 0)}
              </span>
            </p>
            <p className="text-black flex justify-between mt-2">
              <span className="font-medium">{i18n.t("Contributors")}:</span>
              <span className="font-semibold text-black">
                {formatNumber(topic.num_contributors || 0)}
              </span>
            </p>
          </div>
        </div>
      </div>

      {topic.types_sentiment && (
        <div className="bg-white p-6 rounded-lg shadow-sm">
          <h3 className="text-xl font-bold mb-4 text-black border-b pb-2">
            {i18n.t("Sentiment Analysis")}
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(topic.types_sentiment).map(([network, score]) => (
              <div key={network} className={`p-3 rounded-lg flex justify-between items-center ${getBackgroundSentimentColor(typeof score === 'number' ? score : 0)}`}>
                <span className="font-medium text-black">{formatNetworkName(network)}</span>
                <span className={`px-2 py-1 rounded bg-white text-black`}>
                  {getSentimentText(typeof score === 'number' ? score : 0)} ({typeof score === 'number' ? score : 0}%)
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {topic.types_interactions && (
        <div className="bg-white p-6 rounded-lg shadow-sm">
          <h3 className="text-xl font-bold mb-4 text-black border-b pb-2">
            {i18n.t("Engagement By Network")}
          </h3>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {Object.entries(topic.types_interactions).map(([network, count]) => (
              <div key={network} className="p-3 rounded-lg border flex justify-between items-center">
                <span className="font-medium text-black">{formatNetworkName(network)}</span>
                <div className="flex flex-col items-end">
                  <span className="font-medium text-black">
                    {getEngagementText(typeof count === 'number' ? count : 0)}
                  </span>
                  <span className="text-sm text-black">
                    ({formatNumber(typeof count === 'number' ? count : 0)})
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {creators.length > 0 && (
        <div className="bg-white p-6 rounded-lg shadow-sm">
          <h3 className="text-xl font-bold mb-4 text-black border-b pb-2">
            {i18n.t("Top Creators")}
          </h3>
          <div className="space-y-3">
            {creators.slice(0, 3).map((creator) => (
              <LunarCrushCreator key={creator.creator_id} creator={creator} />
            ))}
            {creators.length > 3 && (
              <button 
                className="text-black hover:text-indigo-800 font-medium"
                onClick={() => setActiveTab("creators")}
              >
                {i18n.t("View all")} {creators.length} {i18n.t("creators")}
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );

  return (
    <div className="w-full">
      <div className="flex border-b mb-6">
        <button
          className={`px-6 py-3 font-medium ${
            activeTab === "overview" 
              ? "border-b-2 border-indigo-500 text-black" 
              : "text-black"
          }`}
          onClick={() => setActiveTab("overview")}
        >
          {i18n.t("Overview")}
        </button>
        <button
          className={`px-6 py-3 font-medium ${
            activeTab === "posts" 
              ? "border-b-2 border-indigo-500 text-black" 
              : "text-black"
          }`}
          onClick={() => setActiveTab("posts")}
        >
          {i18n.t("Posts")} ({posts.length})
        </button>
        <button
          className={`px-6 py-3 font-medium ${
            activeTab === "creators" 
              ? "border-b-2 border-indigo-500 text-black" 
              : "text-black"
          }`}
          onClick={() => setActiveTab("creators")}
        >
          {i18n.t("Creators")} ({creators.length})
        </button>
      </div>

      {activeTab === "overview" && renderOverview()}

      {activeTab === "posts" && (
        <div className="space-y-4">
          {posts.length > 0 ? (
            posts.slice(0, 10).map((post) => (
              <LunarCrushPost key={post.id} post={post} />
            ))
          ) : (
            <div className="text-center p-6 bg-gray-50 rounded-lg">
              <p className="text-black">{i18n.t("No posts available for this topic")}</p>
            </div>
          )}
        </div>
      )}

      {activeTab === "creators" && (
        <div className="space-y-3">
          {creators.length > 0 ? (
            creators.slice(0, 10).map((creator) => (
              <LunarCrushCreator key={creator.creator_id} creator={creator} />
            ))
          ) : (
            <div className="text-center p-6 bg-gray-50 rounded-lg">
              <p className="text-black">{i18n.t("No creators available for this topic")}</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
} 