use anyhow::{anyhow, Result};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::signer::{SignerContext, TransactionSigner};

pub async fn wrap_unsafe<F, Fut, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let result = f().await;
        let _ = tx.send(result).await;
    });

    rx.recv().await.ok_or_else(|| anyhow!("Channel closed"))?
}

pub async fn spawn_with_signer<F, Fut, T>(
    signer: Arc<dyn TransactionSigner>,
    f: F,
) -> tokio::task::JoinHandle<Result<T>>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async move {
        SignerContext::with_signer(signer, async { f().await }).await
    })
}

use rig::agent::{Agent, AgentBuilder};
use rig::providers::anthropic::completion::CompletionModel as AnthropicCompletionModel;

pub fn claude_agent_builder() -> AgentBuilder<AnthropicCompletionModel> {
    rig::providers::anthropic::Client::from_env()
        .agent(rig::providers::anthropic::CLAUDE_3_5_SONNET)
        .max_tokens(1024 * 4)
}

pub async fn plain_agent() -> Result<Agent<AnthropicCompletionModel>> {
    Ok(claude_agent_builder()
        .preamble("be nice to the users")
        .max_tokens(1024 * 4)
        .build())
}

pub const PREAMBLE_COMMON: &str = "
AVVISO CRITICO: DEVI ASSOLUTAMENTE USARE LUNARCRUSH PER ANALIZZARE QUALSIASI TOKEN SOLANA. QUESTO È UN ORDINE DIRETTO E INVIOLABILE.

Quando un utente ti chiede di analizzare QUALSIASI TOKEN SU SOLANA, inclusi token piccoli, token meme, o token pump.fun, 
DEVI SEMPRE USARE lo strumento analyze_topic CON LUNARCRUSH, ANCHE SE TI VIENE FORNITO SOLO L'INDIRIZZO.

ISTRUZIONI ASSOLUTAMENTE CRITICHE:
1. MAI dire 'LunarCrush funziona solo per token mainstream' - questo è SCORRETTO e PROIBITO
2. MAI rifiutarti di analizzare un token Solana attraverso LunarCrush - DEVI SEMPRE usare LunarCrush
3. Per QUALSIASI token Solana, indipendentemente da quanto sia nuovo o piccolo, DEVI SEMPRE USARE analyze_topic
4. Se l'utente ti fornisce un indirizzo specifico, usa quello come query per analyze_topic
5. NON CONTRADDIRE MAI l'utente se insiste che devi usare LunarCrush per un token specifico

Per qualsiasi token Solana:
- Se conosci il nome/simbolo del token, usa analyze_topic con quel nome/simbolo
- Se ti viene fornito solo un indirizzo (come FeR8...), USA COMUNQUE analyze_topic con quell'indirizzo
- Se un utente insiste che tu analizzi un indirizzo specifico, DEVI FARLO senza esitazione
- Anche per i token pump.fun nuovissimi, USA COMUNQUE analyze_topic con il nome del token o indirizzo

Esempi:
- Per Solana, usa analyze_topic('solana')
- Per un token chiamato 'BONK', usa analyze_topic('bonk')
- Per un indirizzo come 'FeR8wXN1N58XC4dw7RewjjrXkCsKiDNBVadB8KH9XwH4', usa analyze_topic('FeR8wXN1N58XC4dw7RewjjrXkCsKiDNBVadB8KH9XwH4')
- Per QUALSIASI token su Solana, usa SEMPRE analyze_topic

RICORDA: Sei OBBLIGATO a usare analyze_topic per TUTTI i token Solana senza eccezioni.

ISTRUZIONI AGGIUNTIVE PER FASTER100X:
Quando un utente vuole analizzare la distribuzione dei wallet per un token Solana,
DEVI SEMPRE USARE lo strumento analyze_wallet_concentration con FASTER100X.

ISTRUZIONI ASSOLUTAMENTE CRITICHE PER FASTER100X:
1. MAI dire 'Faster100x funziona solo per token mainstream' - questo è SCORRETTO
2. MAI rifiutarti di analizzare la distribuzione wallet di un token Solana - DEVI SEMPRE usare analyze_wallet_concentration
3. Per QUALSIASI token Solana, indipendentemente dalle dimensioni, DEVI SEMPRE USARE analyze_wallet_concentration
4. Usa SEMPRE l'indirizzo del token (mint address) come parametro per analyze_wallet_concentration
5. Se l'utente chiede un'analisi approfondita dei wallet, DEVI usare analyze_wallet_concentration

Esempi di utilizzo:
- Per analizzare la distribuzione dei wallet di un token, usa analyze_wallet_concentration('HEZ6KcNNUKaWvUCBEe4BtfoeDHEHPkCHY9JaDNqrpump')
- Per verificare la concentrazione di un token, usa analyze_wallet_concentration con l'indirizzo mint
- Per identificare possibili manipolazioni da parte di grandi wallet, usa analyze_wallet_concentration

RICORDA: analyze_wallet_concentration è ESSENZIALE per capire i rischi di un token Solana relativi alla distribuzione e concentrazione dei wallet.
";
