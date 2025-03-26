use super::LunarCrushApiError;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::time::Duration;

pub struct LunarCrushApiClient {
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LunarCrushApiResponseData {
    pub error: u32,
    pub message: String,
}

impl std::fmt::Display for LunarCrushApiResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.error, self.message)
    }
}

// Questo è qui solo per compatibilità con la struttura esistente
// Per la API di LunarCrush usiamo direttamente LunarCrushApiError
pub type LunarCrushApiResponseError = LunarCrushApiResponseData;

impl LunarCrushApiClient {
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        Self {
            api_key,
            base_url: base_url
                .unwrap_or_else(|| "https://lunarcrush.com/api4/public".to_string()),
        }
    }

    pub async fn request<T>(&self, endpoint: &str, params: Option<HashMap<String, String>>) -> Result<T, LunarCrushApiError>
    where
        T: DeserializeOwned + Send + Sync,
    {
        let url = format!("{}{}", self.base_url, endpoint);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build().map_err(|e| LunarCrushApiError::RequestError(e))?;
        
        let mut request_builder = client.get(&url);
        
        // Add API key to headers
        request_builder = request_builder.header("Authorization", format!("Bearer {}", self.api_key));
        
        // Add query parameters if provided
        if let Some(query_params) = params {
            request_builder = request_builder.query(&query_params);
        }
        
        // Execute the request
        let response = request_builder.send().await.map_err(|e| LunarCrushApiError::RequestError(e))?;
        
        // Check status code
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.map_err(|e| LunarCrushApiError::RequestError(e))?;
            tracing::error!("LunarCrush API errore: Status {} - {}", status, error_text);
            
            // Prova a deserializzare come una risposta di errore LunarCrush
            if let Ok(error_data) = serde_json::from_str::<LunarCrushApiResponseData>(&error_text) {
                return Err(LunarCrushApiError::ApiError(error_data));
            }
            
            // Altrimenti crea un errore generico
            return Err(LunarCrushApiError::InvalidInput(anyhow::anyhow!(
                "API error: status code {} - {}",
                status, error_text
            )));
        }
        
        // Get response body as text first to better debug any parsing errors
        let body_text = response.text().await.map_err(|e| LunarCrushApiError::RequestError(e))?;
        
        // Verifica che il body non sia vuoto
        if body_text.trim().is_empty() {
            tracing::error!("LunarCrush API ha restituito una risposta vuota");
            return Err(LunarCrushApiError::InvalidInput(anyhow::anyhow!(
                "Risposta API vuota"
            )));
        }
        
        // Log dell'intero body per debug
        tracing::debug!("LunarCrush API Response: {}", body_text);
        
        // Tentativo di deserializzazione con gestione dettagliata degli errori
        match serde_json::from_str::<T>(&body_text) {
            Ok(data) => Ok(data),
            Err(e) => {
                tracing::error!("Errore di deserializzazione JSON: {} - Body: {}", e, body_text);
                
                // Controlliamo se il JSON è valido anche se non corrisponde al tipo atteso
                if let Ok(_) = serde_json::from_str::<serde_json::Value>(&body_text) {
                    // Il JSON è valido ma non ha la struttura attesa
                    Err(LunarCrushApiError::DeserializeError(
                        e,
                        body_text
                    ))
                } else {
                    // Il JSON non è valido
                    Err(LunarCrushApiError::DeserializeError(
                        e,
                        body_text
                    ))
                }
            }
        }
    }
} 