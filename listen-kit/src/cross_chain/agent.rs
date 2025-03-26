use anyhow::Result;
use rig::agent::Agent;
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

use crate::{
    common::{claude_agent_builder, PREAMBLE_COMMON},
    cross_chain::tools::{ApproveToken, CheckApproval, GetQuote, Swap},
    data::{FetchPriceActionAnalysis, FetchTopTokens},
    dexscreener::tools::SearchOnDexScreener,
};

pub async fn create_cross_chain_agent(
    preamble: Option<String>,
) -> Result<Agent<AnthropicCompletionModel>> {
    let preamble = preamble.unwrap_or(format!(
        "{} {}",
        "you are a cross-chain trading agent", PREAMBLE_COMMON,
    ));
    let agent_builder = claude_agent_builder()
        .preamble(&preamble)
        .tool(SearchOnDexScreener)
        .tool(GetQuote)
        .tool(Swap)
        .tool(ApproveToken)
        .tool(CheckApproval)
        .tool(FetchPriceActionAnalysis)
        .tool(FetchTopTokens);
    Ok(agent_builder.build())
}
