use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ethers::types::{Address, TransactionRequest};
use chrono::{DateTime, Utc};

use crate::api::ApiState;
use crate::security::{SecurityAnalysisResult, SecurityStatus, EmergencyAlert};
use crate::security::emergency_response::EmergencyLevel;

/// Security analysis request
#[derive(Deserialize)]
pub struct SecurityAnalysisRequest {
    pub transaction: TransactionRequest,
}

/// Security report query parameters
#[derive(Deserialize)]
pub struct SecurityReportQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

/// Emergency alert request
#[derive(Deserialize)]
pub struct EmergencyAlertRequest {
    pub title: String,
    pub description: String,
    pub level: EmergencyLevel,
    pub affected_addresses: Option<Vec<Address>>,
}

/// Security status response
#[derive(Serialize)]
pub struct SecurityStatusResponse {
    pub status: SecurityStatus,
    pub last_updated: DateTime<Utc>,
    pub active_threats: usize,
    pub risk_score: f64,
}

/// Security metrics response
#[derive(Serialize)]
pub struct SecurityMetricsResponse {
    pub transactions_analyzed: u64,
    pub threats_detected: u64,
    pub average_risk_score: f64,
    pub emergency_responses: u64,
    pub last_updated: DateTime<Utc>,
}

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/status", get(get_security_status))
        .route("/analyze", post(analyze_transaction))
        .route("/report", get(generate_security_report))
        .route("/metrics", get(get_security_metrics))
        .route("/emergency/alert", post(trigger_emergency_alert))
        .route("/emergency/alerts", get(get_active_alerts))
        .route("/threats/:address", get(get_address_threats))
}

/// Get current security status
async fn get_security_status(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<SecurityStatusResponse>, StatusCode> {
    let status = state.security.get_security_status().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(SecurityStatusResponse {
        status,
        last_updated: Utc::now(),
        active_threats: 0, // Would get from security manager
        risk_score: 0.1, // Would get from security manager
    }))
}

/// Analyze transaction for security risks
async fn analyze_transaction(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<SecurityAnalysisRequest>,
) -> Result<Json<SecurityAnalysisResult>, StatusCode> {
    match state.security.analyze_transaction(&request.transaction).await {
        Ok(analysis) => Ok(Json(analysis)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Generate comprehensive security report
async fn generate_security_report(
    State(state): State<Arc<ApiState>>,
    Query(query): Query<SecurityReportQuery>,
) -> Result<Json<crate::security::SecurityReport>, StatusCode> {
    let start_time = query.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::days(7));
    let end_time = query.end_time.unwrap_or_else(Utc::now);
    
    let report = state.security.generate_security_report(start_time, end_time).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(report))
}

/// Get security metrics
async fn get_security_metrics(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<SecurityMetricsResponse>, StatusCode> {
    // In a real implementation, this would get actual metrics from the security manager
    Ok(Json(SecurityMetricsResponse {
        transactions_analyzed: 1250,
        threats_detected: 15,
        average_risk_score: 0.15,
        emergency_responses: 2,
        last_updated: Utc::now(),
    }))
}

/// Trigger emergency alert
async fn trigger_emergency_alert(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<EmergencyAlertRequest>,
) -> Result<Json<String>, StatusCode> {
    let alert = EmergencyAlert {
        id: format!("alert_{}", Utc::now().timestamp()),
        level: request.level,
        title: request.title,
        description: request.description,
        affected_addresses: request.affected_addresses.unwrap_or(vec![]),
        affected_protocols: vec![],
        detected_at: Utc::now(),
        resolved_at: None,
        auto_actions_taken: vec![],
        manual_actions_required: vec![],
        estimated_impact: None,
    };
    
    state.security.handle_emergency(alert).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json("Emergency alert triggered successfully".to_string()))
}

/// Get active emergency alerts
async fn get_active_alerts(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<Vec<EmergencyAlert>>, StatusCode> {
    // In a real implementation, this would get active alerts from the emergency response system
    Ok(Json(vec![]))
}

/// Get threats for specific address
async fn get_address_threats(
    State(_state): State<Arc<ApiState>>,
    Path(_address): Path<Address>,
) -> Result<Json<Vec<String>>, StatusCode> {
    // In a real implementation, this would get threats for the specific address
    Ok(Json(vec![]))
}
