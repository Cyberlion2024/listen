use crate::distiller::{
    deepseek::make_deepseek_analyst, gemini::make_gemini_analyst,
};

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

// Faster100x analyst trait
#[async_trait::async_trait]
pub trait Faster100xAnalystAgent: AnalystAgent {
    async fn analyze_faster100x(
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

// Semplifichiamo temporaneamente l'analisi Faster100x
pub fn analyze_faster100x(
    query: &str,
    response: &serde_json::Value,
    locale: &str,
    _intent: Option<String>,
) -> Result<String, AnalystError> {
    // Usiamo direttamente un prompt predefinito e l'output JSON grezzo
    if locale == "en" {
        Ok(format!(
            "# Wallet Concentration Analysis for {}

Based on the data from Faster100x, here's what we found:

## Risk Level: {}

### Key Metrics:
- **Gini Index**: {} (measures inequality in token distribution)
- **Top Wallet**: {} of tokens in a single wallet
- **Linked Clusters**: {} groups of connected wallets
- **Top 70% Centralization**: {} wallets control 70% of supply

### Risk Assessment:
{}

{}
",
            query,
            response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("Unknown"),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("gini_index")).and_then(|g| g.as_f64()).unwrap_or(0.0),
            response.get("max_holder").and_then(|h| h.get("percentage")).and_then(|p| p.as_f64()).unwrap_or(0.0),
            response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("clusters")).and_then(|c| c.as_u64()).unwrap_or(0),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("top70_centralization")).and_then(|t| t.as_f64()).unwrap_or(0.0),
            if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("alto") {
                "This token shows concerning concentration patterns that indicate high manipulation risk."
            } else if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("Moderato") {
                "This token has moderate concentration risk. Monitor large holder movements carefully."
            } else {
                "This token appears to have a relatively healthy distribution across wallets."
            },
            if let Some(clusters) = response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("largest_clusters")).and_then(|c| c.as_array()) {
                if !clusters.is_empty() {
                    format!("### Notable Linked Clusters:\n{}", 
                        clusters.iter().take(3).enumerate().map(|(i, cluster)| {
                            format!("- Cluster {}: {} wallets holding {}% of supply", 
                                i+1,
                                cluster.get("wallets").and_then(|w| w.as_u64()).unwrap_or(0),
                                cluster.get("percentage").and_then(|p| p.as_f64()).unwrap_or(0.0)
                            )
                        }).collect::<Vec<_>>().join("\n")
                    )
                } else {
                    "No significant linked wallet clusters detected.".to_string()
                }
            } else {
                "Could not analyze linked wallet clusters.".to_string()
            }
        ))
    } else if locale == "ar" {
        Ok(format!(
            "# تحليل تركيز المحافظ لـ {}

بناءً على البيانات من Faster100x، هذا ما وجدناه:

## مستوى الخطر: {}

### المقاييس الرئيسية:
- **مؤشر جيني**: {} (يقيس عدم المساواة في توزيع الرموز)
- **المحفظة الأكبر**: {}% من الرموز في محفظة واحدة
- **المجموعات المرتبطة**: {} مجموعة من المحافظ المتصلة
- **تركيز أعلى 70%**: {} من المحافظ تتحكم في 70% من المعروض

### تقييم المخاطر:
{}

{}
",
            query,
            response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("غير معروف"),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("gini_index")).and_then(|g| g.as_f64()).unwrap_or(0.0),
            response.get("max_holder").and_then(|h| h.get("percentage")).and_then(|p| p.as_f64()).unwrap_or(0.0),
            response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("clusters")).and_then(|c| c.as_u64()).unwrap_or(0),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("top70_centralization")).and_then(|t| t.as_f64()).unwrap_or(0.0),
            if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("alto") {
                "يُظهر هذا الرمز أنماط تركيز مقلقة تشير إلى مخاطر تلاعب عالية."
            } else if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("Moderato") {
                "هذا الرمز لديه مخاطر تركيز متوسطة. راقب تحركات المالكين الكبار بعناية."
            } else {
                "يبدو أن هذا الرمز يتمتع بتوزيع صحي نسبيًا عبر المحافظ."
            },
            if let Some(clusters) = response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("largest_clusters")).and_then(|c| c.as_array()) {
                if !clusters.is_empty() {
                    format!("### مجموعات المحافظ البارزة:\n{}", 
                        clusters.iter().take(3).enumerate().map(|(i, cluster)| {
                            format!("- المجموعة {}: {} محفظة تحتفظ بـ {}% من المعروض", 
                                i+1,
                                cluster.get("wallets").and_then(|w| w.as_u64()).unwrap_or(0),
                                cluster.get("percentage").and_then(|p| p.as_f64()).unwrap_or(0.0)
                            )
                        }).collect::<Vec<_>>().join("\n")
                    )
                } else {
                    "لم يتم اكتشاف مجموعات محافظ مرتبطة مهمة.".to_string()
                }
            } else {
                "تعذر تحليل مجموعات المحافظ المرتبطة.".to_string()
            }
        ))
    } else {
        Ok(format!(
            "# 钱包集中度分析: {}

基于Faster100x的数据，我们发现：

## 风险等级: {}

### 关键指标:
- **基尼系数**: {} (衡量代币分配不平等程度)
- **最大钱包**: {} 的代币在单个钱包中
- **链接集群**: {} 组关联钱包
- **前70%集中度**: {} 个钱包控制70%的供应量

### 风险评估:
{}

{}
",
            query,
            response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("未知"),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("gini_index")).and_then(|g| g.as_f64()).unwrap_or(0.0),
            response.get("max_holder").and_then(|h| h.get("percentage")).and_then(|p| p.as_f64()).unwrap_or(0.0),
            response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("clusters")).and_then(|c| c.as_u64()).unwrap_or(0),
            response.get("holder_risk").and_then(|r| r.get("distribution")).and_then(|d| d.get("top70_centralization")).and_then(|t| t.as_f64()).unwrap_or(0.0),
            if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("alto") {
                "这个代币显示令人担忧的集中模式，表明存在高操纵风险。"
            } else if response.get("holder_risk").and_then(|r| r.get("risk_level")).and_then(|l| l.as_str()).unwrap_or("").contains("Moderato") {
                "这个代币有中等集中风险。请密切监控大持有者的动向。"
            } else {
                "这个代币在钱包中似乎有相对健康的分布。"
            },
            if let Some(clusters) = response.get("holder_risk").and_then(|r| r.get("linked_wallets")).and_then(|l| l.get("largest_clusters")).and_then(|c| c.as_array()) {
                if !clusters.is_empty() {
                    format!("### 主要关联集群:\n{}", 
                        clusters.iter().take(3).enumerate().map(|(i, cluster)| {
                            format!("- 集群 {}: {} 个钱包持有 {}% 的供应量", 
                                i+1,
                                cluster.get("wallets").and_then(|w| w.as_u64()).unwrap_or(0),
                                cluster.get("percentage").and_then(|p| p.as_f64()).unwrap_or(0.0)
                            )
                        }).collect::<Vec<_>>().join("\n")
                    )
                } else {
                    "未检测到显著的链接钱包集群。".to_string()
                }
            } else {
                "无法分析链接的钱包集群。".to_string()
            }
        ))
    }
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

    pub fn with_faster100x_analyst(
        mut self,
        agent: Box<dyn Faster100xAnalystAgent>,
    ) -> Self {
        self.faster100x_agent = Some(agent);
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

    pub fn from_env_with_locale(
        locale: String,
    ) -> Result<Self, AnalystError> {
        let mut analyst = Self::new(locale.clone());

        // Create appropriate agents based on locale
        let use_deepseek = locale == "zh";

        if use_deepseek {
            let twitter_agent =
                make_deepseek_analyst(AnalystType::Twitter, &locale, None);
            let lunarcrush_agent =
                make_deepseek_analyst(AnalystType::LunarCrush, &locale, None);
            let faster100x_agent =
                make_deepseek_analyst(AnalystType::Faster100x, &locale, None);
            let chart_agent =
                make_deepseek_analyst(AnalystType::Chart, &locale, None);
            let web_agent =
                make_deepseek_analyst(AnalystType::Web, &locale, None);

            analyst = analyst
                .with_twitter_analyst(Box::new(twitter_agent))
                .with_lunarcrush_analyst(Box::new(lunarcrush_agent))
                .with_faster100x_analyst(Box::new(faster100x_agent))
                .with_chart_analyst(Box::new(chart_agent))
                .with_web_analyst(Box::new(web_agent));
        } else {
            let twitter_agent =
                make_gemini_analyst(AnalystType::Twitter, &locale, None);
            let lunarcrush_agent =
                make_gemini_analyst(AnalystType::LunarCrush, &locale, None);
            let faster100x_agent =
                make_gemini_analyst(AnalystType::Faster100x, &locale, None);
            let chart_agent =
                make_gemini_analyst(AnalystType::Chart, &locale, None);
            let web_agent =
                make_gemini_analyst(AnalystType::Web, &locale, None);

            analyst = analyst
                .with_twitter_analyst(Box::new(twitter_agent))
                .with_lunarcrush_analyst(Box::new(lunarcrush_agent))
                .with_faster100x_analyst(Box::new(faster100x_agent))
                .with_chart_analyst(Box::new(chart_agent))
                .with_web_analyst(Box::new(web_agent));
        }

        Ok(analyst)
    }

    pub async fn analyze_faster100x_data(
        &self,
        query: &str,
        response: &serde_json::Value,
        intent: Option<String>,
    ) -> Result<String, AnalystError> {
        // Utilizziamo l'approccio semplificato direttamente
        analyze_faster100x(query, response, &self.locale, intent)
    }
}
