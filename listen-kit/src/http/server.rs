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
struct Faster100xRequest {
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

async fn analyze_faster100x(
    data: web::Json<Faster100xRequest>,
) -> actix_web::Result<HttpResponse> {
    let result = faster100x::get_faster100x_data(&data.token_address)
        .await
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(format!("Analysis failed: {}", e))
        })?;
    
    match result {
        Some(data) => Ok(HttpResponse::Ok().json(data)),
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "error",
            "message": "Token non trovato o concentrazione wallet troppo alta"
        })))
    }
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
            .route("/api/faster100x/analyze", web::post().to(analyze_faster100x))
    })
    .bind("0.0.0.0:6969")?
    .run()
    .await
}
