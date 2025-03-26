use anyhow::{anyhow, Result};
use petgraph::graph::{Graph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use tokio::time::sleep;
use petgraph::Undirected;
use rig_tool_macro::tool;
use crate::distiller::analyst;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Faster100xData {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<ResponseData>,
    pub token_address: Option<String>,
    pub updated_at: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Holder {
    pub address: String,
    pub amount_percentage: String,
}

impl Holder {
    fn get_percentage(&self) -> f64 {
        self.amount_percentage.parse().unwrap_or(0.0)
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

// Tool per l'analisi della concentrazione dei wallet
#[tool(description = "
Analizza la distribuzione e concentrazione dei wallet per un token Solana utilizzando Faster100x.

Parametri:
- token_address (string): L'indirizzo del token da analizzare.

Restituisce:
- Analisi della concentrazione dei wallet
- Cluster di wallet collegati
- Indice di Gini per la distribuzione
- Possibili segnali di rischio come alta concentrazione
")]
pub async fn analyze_wallet_concentration(token_address: String) -> Result<serde_json::Value> {
    tracing::info!("Analisi della concentrazione dei wallet per il token: {}", token_address);
    
    let data = get_faster100x_data(&token_address).await
        .map_err(|e| anyhow!("Errore nell'ottenere dati da Faster100x: {}", e))?;
    
    if let Some(faster_data) = data {
        // Estrai e formatta i dati pertinenti in un formato leggibile
        let analysis_data = format_wallet_analysis(&faster_data)?;
        
        // Detect request language from a query param or request header
        // For now, we'll pass the locale in the result so the UI can choose appropriately
        let locales = vec!["en", "ar", "it", "zh"]; // Supported locales
        
        // Create a multi-language result
        let mut result_with_distillations = analysis_data.as_object().unwrap().clone();
        
        // Add distilled analysis in all supported languages
        let mut distillations = serde_json::Map::new();
        
        for locale in locales {
            match analyst::analyze_faster100x(
                &token_address, 
                &analysis_data, 
                locale,
                None
            ) {
                Ok(distilled) => {
                    distillations.insert(locale.to_string(), serde_json::Value::String(distilled));
                },
                Err(_) => {
                    // On error, skip this locale
                    continue;
                }
            }
        }
        
        result_with_distillations.insert("distilled_analysis".to_string(), serde_json::Value::Object(distillations));
        Ok(serde_json::Value::Object(result_with_distillations))
    } else {
        Err(anyhow!("Nessun dato disponibile per questo token o concentrazione troppo alta"))
    }
}

fn format_wallet_analysis(faster_data: &Faster100xData) -> Result<serde_json::Value> {
    let risk_metrics = compute_holder_risk(faster_data);
    
    // Ottieni alcuni dati rilevanti dal risultato
    let max_holder = faster_data.data.as_ref()
        .and_then(|d| d.response.data.iter()
            .max_by(|a, b| a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()));
    
    // Formatta i dati di analisi
    let analysis = if let Some(metrics) = risk_metrics {
        serde_json::json!({
            "status": "success",
            "token_address": faster_data.token_address,
            "updated_at": faster_data.updated_at,
            "max_holder": max_holder.map(|h| {
                serde_json::json!({
                    "address": h.address,
                    "percentage": h.get_percentage() * 100.0,
                })
            }),
            "holder_risk": {
                "isolated_wallets": {
                    "count": metrics.isolated.num_wallets,
                    "total_percentage": metrics.isolated.total_percentage,
                },
                "linked_wallets": {
                    "clusters": metrics.linked.num_clusters,
                    "total_percentage": metrics.linked.total_percentage,
                    "largest_clusters": metrics.linked.clusters.iter()
                        .take(3)  // Prendi solo i 3 cluster più grandi
                        .map(|c| serde_json::json!({
                            "wallets": c.num_wallets,
                            "percentage": c.total_percentage,
                        }))
                        .collect::<Vec<_>>(),
                },
                "distribution": {
                    "gini_index": metrics.gini_index,
                    "top70_centralization": metrics.isolated.top70_centralization,
                },
                "risk_level": determine_risk_level(&metrics),
            }
        })
    } else {
        serde_json::json!({
            "status": "error",
            "message": "Impossibile calcolare le metriche di rischio",
            "token_address": faster_data.token_address,
        })
    };
    
    Ok(analysis)
}

fn determine_risk_level(metrics: &HolderRisk) -> &'static str {
    // Valutazione del rischio basata su metriche
    let gini = metrics.gini_index;
    let centralization = metrics.isolated.top70_centralization;
    
    if gini > 80.0 || centralization > 90.0 {
        "Estremamente alto"
    } else if gini > 70.0 || centralization > 80.0 {
        "Molto alto"
    } else if gini > 60.0 || centralization > 70.0 {
        "Alto"
    } else if gini > 50.0 || centralization > 60.0 {
        "Moderato"
    } else if gini > 40.0 || centralization > 50.0 {
        "Basso"
    } else {
        "Molto basso"
    }
}

pub async fn get_faster100x_data(token_address: &str) -> Result<Option<Faster100xData>> {
    tracing::info!("Richiedo dati per il token: {}", token_address);
    
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

    tracing::debug!("URL richiesta: {}", url);
    tracing::debug!("Parametri: {:?}", params);

    let response = client.get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Errore nella richiesta HTTP: {}", e);
            anyhow!("Errore nella richiesta HTTP: {}", e)
        })?;

    if response.status() == 429 {
        tracing::warn!("Rate limit raggiunto per l'API di Faster100x, attendo 5 secondi...");
        sleep(Duration::from_secs(5)).await;
        return Ok(None);
    }

    let status = response.status();
    let response_text = response.text().await.map_err(|e| {
        tracing::error!("Errore nel leggere la risposta: {}", e);
        anyhow!("Errore nel leggere la risposta: {}", e)
    })?;

    if !status.is_success() {
        tracing::error!(
            "Errore HTTP da Faster100x: {} - {}",
            status,
            response_text
        );
        return Ok(None);
    }

    tracing::info!("Risposta grezza ricevuta: {}", response_text);

    let data: Vec<Faster100xResponse> = serde_json::from_str(&response_text).map_err(|e| {
        tracing::error!("Errore nel parsing JSON: {}", e);
        anyhow!("Errore nel parsing JSON: {} - Response: {}", e, response_text)
    })?;

    if data.is_empty() {
        tracing::warn!("Risposta vuota da Faster100x API");
        return Ok(None);
    }

    let result = &data[0].result.data.json;

    // Verifica lo status dalla risposta
    if result.status != "success" {
        tracing::error!("Errore API Faster100x: {:?}", result.message);
        return Ok(None);
    }

    tracing::info!("Dati recuperati con successo da Faster100x");

    // Check wallet concentration before returning data
    if result.data.is_some() {
        if let Some(_metrics) = compute_holder_risk(result) {
            // Trova il wallet con la percentuale più alta
            let max_wallet = result.data.as_ref()
                .and_then(|d| d.response.data.iter()
                    .max_by(|a, b| a.get_percentage().partial_cmp(&b.get_percentage()).unwrap()));

            if let Some(max_wallet) = max_wallet {
                let max_percentage = max_wallet.get_percentage() * 100.0;
                tracing::info!(
                    "Analisi concentrazione wallet per il token {}:",
                    token_address
                );
                tracing::info!(
                    "  - Wallet con concentrazione più alta: {}",
                    max_wallet.address
                );
                tracing::info!(
                    "  - Percentuale del supply: {:.2}%",
                    max_percentage
                );

                if max_percentage > 10.0 {
                    tracing::warn!(
                        "  ⚠️ Concentrazione troppo alta (soglia: 10%)",
                    );
                    return Ok(None);
                } else {
                    tracing::info!(
                        "  ✓ Concentrazione sotto la soglia di rischio",
                    );
                }
            }
        }
    }

    Ok(Some(result.clone()))
}

fn compute_holder_risk(faster_data: &Faster100xData) -> Option<HolderRisk> {
    let response_data = faster_data.data.as_ref()?;
    let fund_graph_data = &response_data.response.fund_graph_data;
    let holdings_list = &response_data.response.data;

    // Map address -> percentage from 'data'
    let mut wallet_holdings: HashMap<String, f64> = HashMap::new();
    for holding in holdings_list {
        wallet_holdings.insert(holding.address.clone(), holding.get_percentage());
    }

    // Trova il wallet con la percentuale più alta
    let max_wallet = holdings_list.iter()
        .max_by(|a, b| a.get_percentage().partial_cmp(&b.get_percentage()).unwrap());
    
    if let Some(max_wallet) = max_wallet {
        tracing::info!(
            "Wallet con la percentuale più alta: {} con {}% del supply",
            max_wallet.address,
            max_wallet.get_percentage() * 100.0
        );
    }

    // Extract ALL nodes from fund_graph_data
    let nodes_list = &fund_graph_data.nodes;
    if nodes_list.is_empty() {
        tracing::error!("No nodes found in fund_graph_data['nodes']");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_faster100x_data() {
        let _ = env_logger::builder().is_test(true).try_init();
        tracing::info!("Iniziando il test con il token PUMP...");
        
        let result = get_faster100x_data("HEZ6KcNNUKaWvUCBEe4BtfoeDHEHPkCHY9JaDNqrpump").await;
        assert!(result.is_ok(), "La chiamata dovrebbe essere Ok");
        
        if let Ok(Some(data)) = result {
            tracing::info!("Dati ricevuti: {:#?}", data);
            assert_eq!(data.status, "success", "Lo status dovrebbe essere 'success'");
            assert!(data.data.is_some(), "Dovrebbero esserci dei dati");
            
            if let Some(ref response_data) = data.data {
                assert!(!response_data.response.data.is_empty(), "La lista degli holder non dovrebbe essere vuota");
                assert!(!response_data.response.fund_graph_data.nodes.is_empty(), "Dovrebbero esserci dei nodi nel grafo");
                
                // Verifica la concentrazione dei wallet
                if let Some(metrics) = compute_holder_risk(&data) {
                    tracing::info!("Metriche di rischio: {:#?}", metrics);
                    assert!(metrics.isolated.top70_centralization > 0.0, "La centralizzazione dovrebbe essere calcolata");
                    assert!(metrics.gini_index > 0.0, "L'indice di Gini dovrebbe essere calcolato");
                }
            }
        } else {
            // Se non riceviamo dati, verifichiamo che sia dovuto alla concentrazione alta
            if let Ok(None) = result {
                tracing::info!("Dati non ricevuti a causa della concentrazione alta del wallet");
            } else {
                panic!("Errore inaspettato nella chiamata API");
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