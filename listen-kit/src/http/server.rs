use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use actix_web::{web, App, HttpServer, HttpResponse};
use privy::Privy;
use serde::Deserialize;

use super::routes::{auth, healthz, stream};
use super::state::AppState;
use crate::mongo::MongoClient;
use crate::lunarcrush;
use crate::faster100x;

#[derive(Deserialize)]
struct AnalyzeRequest {
    topic: String,
}

#[derive(Deserialize)]
struct WalletAnalysisRequest {
    token_address: String,
}

async fn analyze_lunarcrush(
    data: web::Json<AnalyzeRequest>,
) -> actix_web::Result<HttpResponse> {
    let result = lunarcrush::analyze_topic(data.topic.clone())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Analysis failed: {}", e))
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

async fn analyze_wallet_concentration(
    data: web::Json<WalletAnalysisRequest>,
) -> actix_web::Result<HttpResponse> {
    let result = faster100x::analyze_wallet_concentration(data.token_address.clone())
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Wallet analysis failed: {}", e))
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

pub async fn run_server(
    privy: Privy,
    mongo: MongoClient,
) -> std::io::Result<()> {
    let state = web::Data::new(AppState::new(privy, mongo));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .service(healthz)
            .service(stream)
            .service(auth)
            .route("/api/lunarcrush/analyze", web::post().to(analyze_lunarcrush))
            .route("/api/faster100x/analyze", web::post().to(analyze_wallet_concentration))
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
