use crate::faster100x::Faster100xApi;
use crate::lunarcrush::LunarCrushApi;
use crate::mongo::MongoClient;
use privy::Privy;
use std::sync::Arc;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel;
use crate::common::claude_agent_builder;

pub struct AppState {
    pub privy: Arc<Privy>,
    pub mongo: MongoClient,
    pub lunarcrush_api: Arc<LunarCrushApi>,
    pub faster100x_api: Arc<Faster100xApi>,
    pub agent: Arc<Agent<CompletionModel>>,
}

impl AppState {
    pub fn new(privy: Privy, mongo: MongoClient) -> Self {
        let lunarcrush_api = Arc::new(LunarCrushApi::new(std::env::var("LUNARCRUSH_API_KEY").unwrap_or_default()));
        let faster100x_api = Arc::new(Faster100xApi::new("http://localhost:6969".to_string()));
        let agent = Arc::new(claude_agent_builder().build());

        Self {
            privy: Arc::new(privy),
            mongo,
            lunarcrush_api,
            faster100x_api,
            agent,
        }
    }
}
