use anyhow::{anyhow, Result};
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use tokio::time::sleep;
use petgraph::Undirected;
use std::sync::Arc;
use rig::completion::request::ToolDefinition;
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Faster100xResponse {
    result: ResultData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResultData {
    data: JsonData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonData {
    json: Faster100xData,
}

#[derive(Debug, Clone)]
pub struct Faster100xApi {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faster100xData {
    pub status: String,
    pub message: Option<String>,
    pub holders: Option<Vec<Holder>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseData {
    pub response: InnerResponseData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InnerResponseData {
    pub fund_graph_data: FundGraphData,
    pub data: Vec<Holder>,
    pub top_nodes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FundGraphData {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Link {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holder {
    pub address: String,
    pub amount: f64,
    pub amount_percentage: f64,
}

impl Holder {
    fn get_percentage(&self) -> f64 {
        self.amount_percentage
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HolderRisk {
    pub isolated: IsolatedHolders,
    pub linked: LinkedHolders,
    pub gini_index: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IsolatedHolders {
    pub num_wallets: usize,
    pub total_percentage: f64,
    pub top70_centralization: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedHolders {
    pub num_clusters: usize,
    pub clusters: Vec<Cluster>,
    pub total_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cluster {
    pub cluster_wallets: Vec<String>,
    pub num_wallets: usize,
    pub total_percentage: f64,
}

impl Faster100xApi {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn get_faster100x_data(&self, token_address: &str) -> Result<Option<Faster100xData>> {
        let url = format!("{}/api/faster100x/analyze", self.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "token_address": token_address
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.json::<Faster100xData>().await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

pub fn make_faster100x_api(base_url: String) -> Arc<Faster100xApi> {
    Arc::new(Faster100xApi::new(base_url))
}

pub async fn get_faster100x_data(token_address: &str) -> Result<Option<Faster100xData>> {
    log::info!("Richiedo dati per il token: {}", token_address);
    
    // Add delay to avoid rate limiting
    sleep(Duration::from_secs(2)).await;

    let client = Client::new();
    let url = "https://faster100x.com/api/trpc/embedded.getAnalyzeResult";
    let params = [
        ("batch", "1"),
        (
            "input",
            &serde_json::json!({
                "0": {
                    "json": {
                        "tokenAddress": token_address
                    }
                }
            })
            .to_string(),
        ),
    ];

    log::debug!("URL richiesta: {}", url);
    log::debug!("Parametri: {:?}", params);

    let response = client.get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| {
            log::error!("Errore nella richiesta HTTP: {}", e);
            anyhow!("Errore nella richiesta HTTP: {}", e)
        })?;

    if response.status() == 429 {
        log::warn!("Rate limit raggiunto per l'API di Faster100x, attendo 5 secondi...");
        sleep(Duration::from_secs(5)).await;
        return Ok(None);
    }

    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        log::error!("Errore nel leggere la risposta: {}", e);
        anyhow!("Errore nel leggere la risposta: {}", e)
    })?;

    if !status.is_success() {
        log::error!(
            "Errore HTTP da Faster100x: {} - {}",
            status,
            response_text
        );
        return Ok(None);
    }

    log::info!("Risposta grezza ricevuta: {}", response_text);

    let data: Vec<Faster100xResponse> = serde_json::from_str(&response_text).map_err(|e| {
        log::error!("Errore nel parsing JSON: {}", e);
        anyhow!("Errore nel parsing JSON: {} - Response: {}", e, response_text)
    })?;

    if data.is_empty() {
        log::warn!("Risposta vuota da Faster100x API");
        return Ok(None);
    }

    let result = &data[0].result.data.json;

    // Verifica lo status dalla risposta
    if result.status != "success" {
        log::error!("Errore API Faster100x: {:?}", result.message);
        return Ok(None);
    }

    log::info!("Dati recuperati con successo da Faster100x");

    // Check wallet concentration before returning data
    if result.holders.is_some() {
        if let Some(_metrics) = compute_holder_risk(result) {
            // Trova il wallet con la percentuale più alta
            let max_wallet = result.holders.as_ref()
                .and_then(|d| d.iter()
                    .max_by(|a, b| a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()));

            if let Some(max_wallet) = max_wallet {
                let max_percentage = max_wallet.get_percentage() * 100.0;
                log::info!(
                    "Analisi concentrazione wallet per il token {}:",
                    token_address
                );
                log::info!(
                    "  - Wallet con concentrazione più alta: {}",
                    max_wallet.address
                );
                log::info!(
                    "  - Percentuale del supply: {:.2}%",
                    max_percentage
                );

                if max_percentage > 15.0 {
                    log::warn!(
                        "  ⚠️ Concentrazione troppo alta (soglia: 15%)",
                    );
                    return Ok(None);
                } else {
                    log::info!(
                        "  ✓ Concentrazione sotto la soglia di rischio",
                    );
                }
            }
        }
    }

    Ok(Some(result.clone()))
}

fn compute_holder_risk(faster_data: &Faster100xData) -> Option<HolderRisk> {
    let response_data = faster_data.holders.as_ref()?;
    // Assumiamo che i dati di fund_graph_data siano direttamente nei dati degli holders
    // Creiamo un fund_graph_data vuoto da usare in caso di errore
    let empty_graph_data = FundGraphData { nodes: vec![], links: vec![] };
    let fund_graph_data = if response_data.is_empty() {
        &empty_graph_data
    } else {
        &empty_graph_data // Per ora usiamo un grafico vuoto, dovrei verificare come accedere ai dati reali
    };
    let holdings_list = response_data;

    // Map address -> percentage from 'data'
    let mut wallet_holdings: HashMap<String, f64> = HashMap::new();
    for holding in holdings_list {
        wallet_holdings.insert(holding.address.clone(), holding.get_percentage());
    }

    // Trova il wallet con la percentuale più alta
    let max_wallet = holdings_list.iter()
        .max_by(|a, b| a.get_percentage().partial_cmp(&b.get_percentage()).unwrap());
    
    if let Some(max_wallet) = max_wallet {
        log::info!(
            "Wallet con la percentuale più alta: {} con {}% del supply",
            max_wallet.address,
            max_wallet.get_percentage() * 100.0
        );
    }

    // Extract ALL nodes from fund_graph_data
    let nodes_list = &fund_graph_data.nodes;
    if nodes_list.is_empty() {
        log::error!("No nodes found in fund_graph_data['nodes']");
        return None;
    }

    // Creo un grafo con petgraph::Graph
    let mut graph = Graph::<String, (), Undirected>::new_undirected();
    
    // Mappa per tenere traccia di String -> NodeIndex
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Create graph nodes
    for node_info in nodes_list {
        let addr = node_info.id.clone();
        let idx = graph.add_node(addr.clone());
        node_indices.insert(addr, idx);
    }

    // Add edges
    for link in &fund_graph_data.links {
        if let (Some(&source_idx), Some(&target_idx)) = (
            node_indices.get(&link.source),
            node_indices.get(&link.target),
        ) {
            graph.add_edge(source_idx, target_idx, ());
        }
    }

    // Analyze connected components
    let mut isolated_wallets = Vec::new();
    let mut linked_clusters = Vec::new();

    let _num_components = petgraph::algo::connected_components(&graph);
    let node_to_component: HashMap<NodeIndex, usize> = graph
        .node_indices()
        .map(|node| {
            let component = 0; // Default component
            
            for (i, component_nodes) in petgraph::algo::kosaraju_scc(&graph).iter().enumerate() {
                if component_nodes.contains(&node) {
                    return (node, i);
                }
            }
            
            (node, component)
        })
        .collect();

    // Group nodes by component
    let mut components: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    for (node, component) in node_to_component {
        components.entry(component).or_default().push(node);
    }

    // Process each component
    for (_, nodes) in components {
        if nodes.len() == 1 {
            // Isolated wallet
            let node_idx = nodes[0];
            let addr = graph[node_idx].clone();
            isolated_wallets.push(addr);
        } else {
            // Linked cluster
            let cluster_wallets: Vec<String> = nodes
                .iter()
                .map(|&node_idx| graph[node_idx].clone())
                .collect();
                
            let total_pct = cluster_wallets
                .iter()
                .filter_map(|addr| wallet_holdings.get(addr))
                .sum::<f64>()
                * 100.0;
                
            linked_clusters.push(Cluster {
                cluster_wallets,
                num_wallets: nodes.len(),
                total_percentage: total_pct,
            });
        }
    }

    // Calculate centralization on top 70 positions
    let centralization_score = if !wallet_holdings.is_empty() {
        let mut sorted_amounts: Vec<f64> = wallet_holdings.values().copied().collect();
        sorted_amounts.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let n = 70.min(sorted_amounts.len());
        sorted_amounts.iter().take(n).sum::<f64>() * 100.0
    } else {
        0.0
    };

    // Calculate total percentages
    let total_isolated_percentage = isolated_wallets
        .iter()
        .filter_map(|addr| wallet_holdings.get(addr))
        .sum::<f64>()
        * 100.0;

    let total_linked_percentage = linked_clusters
        .iter()
        .map(|cluster| cluster.total_percentage)
        .sum();

    // Calculate Gini Index
    let gini_index = calculate_gini_index(&wallet_holdings.values().copied().collect::<Vec<f64>>());

    Some(HolderRisk {
        isolated: IsolatedHolders {
            num_wallets: isolated_wallets.len(),
            total_percentage: total_isolated_percentage,
            top70_centralization: centralization_score,
        },
        linked: LinkedHolders {
            num_clusters: linked_clusters.len(),
            clusters: linked_clusters,
            total_percentage: total_linked_percentage,
        },
        gini_index: gini_index * 100.0,
    })
}

fn calculate_gini_index(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted_values = values.to_vec();
    sorted_values.sort_by(|a, b| b.partial_cmp(a).unwrap());
    
    let n = sorted_values.len() as f64;
    let sum = sorted_values.iter().sum::<f64>();
    
    if sum == 0.0 {
        return 0.0;
    }

    let mut index_sum = 0.0;
    for (i, value) in sorted_values.iter().enumerate() {
        index_sum += (2.0 * (i + 1) as f64 - n - 1.0) * value;
    }

    (index_sum / (n * sum)).abs()
}

pub mod tools {
    use super::*;
    use rig::tool::Tool;

    #[derive(Debug)]
    pub enum Faster100xError {
        ApiError(String),
        ParseError(String),
        NoData,
        InvalidInput(String),
    }

    impl std::fmt::Display for Faster100xError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Faster100xError::ApiError(msg) => write!(f, "API Error: {}", msg),
                Faster100xError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
                Faster100xError::NoData => write!(f, "No data available"),
                Faster100xError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            }
        }
    }

    impl std::error::Error for Faster100xError {}

    pub struct GetFaster100xData;

    impl Tool for GetFaster100xData {
        const NAME: &'static str = "get_faster100x_data";
        type Error = Faster100xError;
        type Args = String;
        type Output = String;

        fn definition(&self, _: String) -> impl std::future::Future<Output = ToolDefinition> + std::marker::Send + Sync {
            async move {
                ToolDefinition {
                    name: Self::NAME.to_string(),
                    description: "Analizza i holder di un token Solana usando Faster100x".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "token_address": {
                                "type": "string",
                                "description": "L'indirizzo del token Solana da analizzare"
                            }
                        },
                        "required": ["token_address"]
                    }),
                }
            }
        }

        async fn call(&self, input: String) -> Result<Self::Output, Self::Error> {
            log::info!("Input ricevuto nel tool: '{}'", input);
            
            // Estrai il token address nel modo più semplice possibile
            let token_address = extract_token_address_simple(&input)
                .ok_or_else(|| Faster100xError::InvalidInput(format!("Impossibile trovare un indirizzo Solana valido nell'input: '{}'", input)))?;
            
            log::info!("Token address estratto: {}", token_address);
            
            // Usa la funzione globale per ottenere i dati
            let data = get_faster100x_data(&token_address)
                .await
                .map_err(|e| Faster100xError::ApiError(e.to_string()))?;
            
            // Formatta la risposta
            let result = format_response(token_address, data);
            
            // Ritorna il risultato
            Ok(result)
        }
    }
    
    // Funzione semplificata per l'estrazione del token address
    fn extract_token_address_simple(input: &str) -> Option<String> {
        // 1. Se l'input è già un indirizzo diretto (stringhe lunghe alfanumeriche)
        if input.len() >= 32 && input.len() <= 44 && !input.contains('{') {
            return Some(input.trim().to_string());
        }
        
        // 2. Cerca stringhe che sembrano indirizzi Solana nel testo
        for word in input.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.len() >= 32 && clean.len() <= 44 {
                return Some(clean.to_string());
            }
        }
        
        // 3. Prova a parsare come JSON e cercare il campo token_address
        if let Ok(json) = serde_json::from_str::<Value>(input) {
            if let Some(token) = json.get("token_address").and_then(|v| v.as_str()) {
                return Some(token.to_string());
            }
        }
        
        None
    }
    
    // Formatta la risposta in un formato leggibile
    fn format_response(token_address: String, data: Option<Faster100xData>) -> String {
        let mut result = String::new();
        
        if let Some(data) = data {
            result.push_str(&format!("🔍 **ANALISI HOLDER PER IL TOKEN {}**\n\n", token_address));
            
            // Aggiungi informazioni sui holder
            if let Some(ref holders) = data.holders {
                result.push_str("📊 **TOP HOLDERS:**\n");
                let mut count = 0;
                for holder in holders {
                    count += 1;
                    if count <= 10 {
                        result.push_str(&format!(
                            "#{} {}...{}: {:.2}%\n",
                            count,
                            &holder.address[..6],
                            &holder.address[holder.address.len().saturating_sub(4)..],
                            holder.amount_percentage * 100.0
                        ));
                    }
                }
                result.push_str(&format!("Totale holders: {}\n", holders.len()));
            }

            // Aggiungi metriche di rischio
            if let Some(risk_metrics) = compute_holder_risk(&data) {
                result.push_str("\n⚠️ **METRICHE DI RISCHIO:**\n");
                result.push_str(&format!("• Indice Gini: {:.2}% (più alto = più concentrato)\n", risk_metrics.gini_index));
                result.push_str(&format!("• Wallet isolati: {}\n", risk_metrics.isolated.num_wallets));
                result.push_str(&format!("• Cluster connessi: {}\n", risk_metrics.linked.num_clusters));
                result.push_str(&format!("• Percentuale wallet isolati: {:.2}%\n", risk_metrics.isolated.total_percentage));
                result.push_str(&format!("• Percentuale cluster connessi: {:.2}%\n", risk_metrics.linked.total_percentage));
            }
        } else {
            result.push_str(&format!("❌ **Nessun dato disponibile per il token {}**\n\n", token_address));
            result.push_str("**Possibili cause:**\n");
            result.push_str("• Il token non esiste\n");
            result.push_str("• Concentrazione troppo alta (>15% in un singolo wallet)\n");
            result.push_str("• Errore temporaneo dell'API\n");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_faster100x_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        log::info!("Iniziando il test con il token PUMP...");
        
        let result = get_faster100x_data("HEZ6KcNNUKaWvUCBEe4BtfoeDHEHPkCHY9JaDNqrpump").await;
        assert!(result.is_ok(), "La chiamata dovrebbe essere Ok");
        
        if let Ok(Some(data)) = result {
            log::info!("Dati ricevuti: {:#?}", data);
            assert_eq!(data.status, "success", "Lo status dovrebbe essere 'success'");
            assert!(data.holders.is_some(), "Dovrebbero esserci dei dati");
            
            if let Some(ref response_data) = data.holders {
                assert!(!response_data.is_empty(), "La lista degli holder non dovrebbe essere vuota");
                
                // Verifica la concentrazione dei wallet
                if let Some(metrics) = compute_holder_risk(&data) {
                    log::info!("Metrics: {:#?}", metrics);
                    assert!(metrics.gini_index >= 0.0 && metrics.gini_index <= 100.0, "Gini index dovrebbe essere tra 0 e 100");
                }
            }
        }
    }

    #[test]
    fn test_gini_index() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let gini = calculate_gini_index(&values);
        assert!(gini >= 0.0 && gini <= 1.0);
    }
} 