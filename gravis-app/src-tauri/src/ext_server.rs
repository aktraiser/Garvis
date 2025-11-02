// GRAVIS Extension Server - Phase 0 Spike Production-Ready
// Serveur HTTP local sécurisé pour communication avec l'extension Chrome

use axum::{
    routing::{get, post},
    Router, Json,
    http::{StatusCode, HeaderValue, Method},
    extract::ConnectInfo,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, collections::{HashMap, HashSet}, sync::{Arc, Mutex}};
use tauri::{AppHandle, Emitter};
use tower_http::cors::CorsLayer;
use tokio::time::Duration;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use ammonia;
use base64;
use rand;

type HmacSha256 = Hmac<Sha256>;

/// Structure de sécurité pour l'extension
#[derive(Debug)]
pub struct ExtensionSecurity {
    secret: String,
    used_nonces: Arc<Mutex<HashSet<String>>>,
    rate_limiter: Arc<Mutex<HashMap<String, Vec<u64>>>>,
}

impl ExtensionSecurity {
    pub fn new() -> Self {
        use base64::prelude::*;
        let secret = BASE64_STANDARD.encode(rand::random::<[u8; 32]>());
        tracing::info!("🔑 Extension security initialized with new secret");
        
        Self {
            secret,
            used_nonces: Arc::new(Mutex::new(HashSet::new())),
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_token(&self) -> String {
        self.secret.clone()
    }

    pub fn validate_request(&self, payload: &SecureExtractedContent, client_ip: &str) -> Result<(), &'static str> {
        // 1. Rate limiting (10 req/min)
        if !self.check_rate_limit(client_ip) {
            tracing::warn!("Rate limit exceeded for IP: {}", client_ip);
            return Err("Rate limit exceeded");
        }

        // 2. Validation temporelle (±90s)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "Invalid system time")?
            .as_millis() as i64;
        
        if (now - payload.ts).abs() > 90_000 {
            tracing::warn!("Request timestamp too old or from future: {} vs {}", payload.ts, now);
            return Err("Request too old or from future");
        }

        // 3. Nonce replay protection
        {
            let mut nonces = self.used_nonces.lock().unwrap();
            if nonces.contains(&payload.nonce) {
                tracing::warn!("Nonce replay detected: {}", payload.nonce);
                return Err("Nonce already used");
            }
            nonces.insert(payload.nonce.clone());
            
            // Nettoyer les anciens nonces (simple cleanup)
            if nonces.len() > 1000 {
                tracing::info!("Cleaning old nonces cache");
                nonces.clear();
            }
        }

        // 4. Validation signature HMAC
        if !self.verify_signature(payload) {
            tracing::warn!("Invalid HMAC signature for request from: {}", payload.url);
            return Err("Invalid signature");
        }

        // 5. Validation contenu
        if payload.main_content.len() > 50_000 {
            tracing::warn!("Content too large: {} bytes", payload.main_content.len());
            return Err("Content too large");
        }

        // 6. Blocklist domaines sensibles
        let blocked_domains = [
            "banking", "paypal", "stripe", "chrome://", "file://",
            "localhost:3000", "127.0.0.1", "intranet", "internal"
        ];
        
        for domain in blocked_domains {
            if payload.url.to_lowercase().contains(domain) {
                tracing::warn!("Blocked domain detected: {} in {}", domain, payload.url);
                return Err("Blocked domain");
            }
        }

        Ok(())
    }

    fn check_rate_limit(&self, client_ip: &str) -> bool {
        let mut limiter = self.rate_limiter.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let requests = limiter.entry(client_ip.to_string()).or_insert_with(Vec::new);
        
        // Nettoyer les requêtes anciennes (> 1 minute)
        requests.retain(|&time| now - time < 60);
        
        if requests.len() >= 10 {
            return false;
        }
        
        requests.push(now);
        true
    }

    fn verify_signature(&self, payload: &SecureExtractedContent) -> bool {
        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        // Reconstruire le canonical body (clés triées)
        let canonical = match serde_json::to_string(&serde_json::json!({
            "extraction_method": payload.extraction_method,
            "flags": payload.flags,
            "main_content": payload.main_content,
            "nonce": payload.nonce,
            "selected_text": payload.selected_text,
            "title": payload.title,
            "ts": payload.ts,
            "url": payload.url,
            "v": payload.v
        })) {
            Ok(canonical) => canonical,
            Err(_) => return false,
        };
        
        tracing::debug!("🔍 DEBUG - Server canonical payload: {}", &canonical[..200.min(canonical.len())]);
        tracing::debug!("🔑 DEBUG - Server using secret: {}...", &self.secret[..10.min(self.secret.len())]);
        
        mac.update(canonical.as_bytes());
        use base64::prelude::*;
        let expected = BASE64_STANDARD.encode(mac.finalize().into_bytes());
        
        tracing::debug!("📝 DEBUG - Server expected signature: {}", expected);
        tracing::debug!("📝 DEBUG - Client provided signature: {}", payload.sig);
        
        let is_valid = payload.sig == expected;
        if !is_valid {
            tracing::warn!("❌ Signature mismatch - Expected: {}, Got: {}", expected, payload.sig);
        }
        
        is_valid
    }

    /// Vérifier signature HMAC avec body brut
    fn verify_signature_raw(&self, canonical_body: &str, signature: &str) -> bool {
        let mut mac = match HmacSha256::new_from_slice(self.secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };
        
        mac.update(canonical_body.as_bytes());
        use base64::prelude::*;
        let expected = BASE64_STANDARD.encode(mac.finalize().into_bytes());
        
        tracing::info!("📝 DEBUG - Server expected signature: {}", expected);
        tracing::info!("📝 DEBUG - Client provided signature: {}", signature);
        
        let is_valid = signature == expected;
        if !is_valid {
            tracing::warn!("❌ Signature mismatch - Expected: {}, Got: {}", expected, signature);
        }
        
        is_valid
    }

    /// Validation sans HMAC (déjà fait)
    fn validate_request_no_hmac(&self, payload: &SecureExtractedContent, client_ip: &str) -> Result<(), &'static str> {
        // 1. Rate limiting (10 req/min)
        if !self.check_rate_limit(client_ip) {
            return Err("Rate limit exceeded");
        }

        // 2. Nonce replay protection
        {
            let mut nonces = self.used_nonces.lock().unwrap();
            if nonces.contains(&payload.nonce) {
                tracing::warn!("Nonce replay detected: {}", payload.nonce);
                return Err("Nonce already used");
            }
            nonces.insert(payload.nonce.clone());
        }

        // 3. Timestamp validation (15 min window)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        if payload.ts > (now + 900_000) as i64 || payload.ts < (now.saturating_sub(900_000)) as i64 {
            tracing::warn!("Timestamp out of window: {} vs {}", payload.ts, now);
            return Err("Timestamp out of window");
        }

        // 4. Version check
        if payload.v != "1" {
            tracing::warn!("Invalid version: {}", payload.v);
            return Err("Invalid version");
        }

        // 5. Content size limits (2MB)
        if payload.main_content.len() > 2_097_152 {
            tracing::warn!("Content too large: {} bytes", payload.main_content.len());
            return Err("Content too large");
        }

        // 6. Blocklist domaines sensibles
        let blocked_domains = [
            "banking", "paypal", "stripe", "chrome://", "file://",
            "localhost:3000", "127.0.0.1", "intranet", "internal"
        ];
        
        for domain in blocked_domains {
            if payload.url.to_lowercase().contains(domain) {
                tracing::warn!("Blocked domain detected: {} in {}", domain, payload.url);
                return Err("Blocked domain");
            }
        }

        Ok(())
    }
}

/// Payload sécurisé reçu de l'extension
#[derive(Serialize, Deserialize, Debug)]
pub struct SecureExtractedContent {
    pub v: String,
    pub nonce: String,
    pub ts: i64,
    pub title: String,
    pub url: String,
    pub main_content: String,
    pub selected_text: Option<String>,
    pub extraction_method: String,
    pub flags: ContentFlags,
    pub sig: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentFlags {
    pub is_pdf: bool,
    pub is_paywalled: bool,
}

#[derive(Serialize)]
struct ApiResponse {
    ok: bool,
    message: Option<String>,
}

#[derive(Serialize)]
struct PingResponse {
    ok: bool,
    token: String,
    exp: u64,
    message: Option<String>,
}

/// Handler pour déclencher l'extraction depuis AWCS
async fn handle_extension_trigger(
    Json(payload): Json<serde_json::Value>
) -> Result<Json<ApiResponse>, StatusCode> {
    tracing::info!("🔄 Extension trigger request received: {:?}", payload);
    
    // TODO: Déclencher l'extraction de l'onglet actif via JavaScript injection
    // Pour l'instant, retourner une réponse optimiste
    
    Ok(Json(ApiResponse { 
        ok: true, 
        message: Some("Extension trigger acknowledged".to_string()) 
    }))
}

/// Démarrer le serveur extension HTTP local
pub async fn start_extension_server(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let security = Arc::new(ExtensionSecurity::new());
    
    let router = Router::new()
        .route("/api/extension/content", post({
            let app = app.clone();
            let security = security.clone();
            move |ConnectInfo(addr): ConnectInfo<SocketAddr>, body: axum::body::Bytes| async move {
                handle_extension_content_raw(app, security, body, ConnectInfo(addr)).await
            }
        }))
        .route("/api/extension/ping", get({
            let security = security.clone();
            move || async move {
                ping_with_token(security).await
            }
        }))
        .route("/api/extension/trigger", post({
            move |Json(payload): Json<serde_json::Value>| async move {
                handle_extension_trigger(Json(payload)).await
            }
        }))
        .layer(create_cors_layer());

    let addr: SocketAddr = "127.0.0.1:8766".parse()?;
    tracing::info!("🌐 Extension API server starting on http://127.0.0.1:8766");
    
    tokio::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => listener,
            Err(e) => {
                tracing::error!("Failed to bind extension server: {}", e);
                return;
            }
        };
        
        if let Err(e) = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>()
        ).await {
            tracing::error!("Extension server error: {}", e);
        }
    });

    Ok(())
}

/// Handler pour recevoir le contenu de l'extension (utilise body brut pour HMAC)
async fn handle_extension_content_raw(
    app: AppHandle,
    security: Arc<ExtensionSecurity>,
    body: axum::body::Bytes,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let client_ip = addr.ip().to_string();
    
    // Désérialiser le payload
    let mut payload: SecureExtractedContent = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("❌ Invalid JSON payload: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    tracing::info!("📥 Extension content request from: {} ({})", payload.url, client_ip);
    
    // Extraire la signature et la valider avec le body brut (sans la signature)
    let signature = payload.sig.clone();
    payload.sig = String::new(); // Enlever la signature pour la validation
    
    // Re-créer le canonical body EXACTEMENT comme le fait JavaScript
    let canonical_payload = serde_json::json!({
        "extraction_method": payload.extraction_method,
        "flags": payload.flags,
        "main_content": payload.main_content,
        "nonce": payload.nonce,
        "selected_text": payload.selected_text,
        "title": payload.title,
        "ts": payload.ts,
        "url": payload.url,
        "v": payload.v
    });
    let canonical_body = canonical_payload.to_string();
    
    tracing::info!("🔍 DEBUG - Server canonical payload: {}", &canonical_body[..200.min(canonical_body.len())]);
    
    // Validation HMAC avec le body canonique
    if !security.verify_signature_raw(&canonical_body, &signature) {
        tracing::warn!("❌ Invalid HMAC signature for request from: {}", payload.url);
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Validation sécurisée (sans HMAC car déjà fait)
    if let Err(error) = security.validate_request_no_hmac(&payload, &client_ip) {
        tracing::warn!("❌ Security validation failed: {}", error);
        return Err(StatusCode::UNAUTHORIZED);
    }

    handle_extension_content_common(app, payload).await
}

/// Logique commune pour traiter le contenu de l'extension
async fn handle_extension_content_common(
    app: AppHandle,
    mut payload: SecureExtractedContent,
) -> Result<Json<ApiResponse>, StatusCode> {
    // Sanitisation du contenu avec ammonia (préserve UTF-8)
    payload.main_content = sanitize_content_secure(&payload.main_content);

    // Formatage pour GRAVIS chat
    let extraction_source = if payload.flags.is_pdf {
        "📄 PDF"
    } else if payload.flags.is_paywalled {
        "🔒 Paywall"
    } else if payload.extraction_method.contains("selection") {
        "✂️ Sélection"
    } else {
        "🌐 Page"
    };

    // Analyse intelligente du contenu pour extraction structurée
    let structured_data = extract_structured_data(&payload.main_content, &payload.url);
    
    let formatted = format_content_with_intelligence(
        extraction_source,
        &payload.title,
        &payload.url,
        &payload.main_content,
        &structured_data
    );

    tracing::info!("📄 Content formatted: {} chars from {}", 
                   formatted.len(), payload.extraction_method);

    // Émettre vers le frontend
    if let Err(e) = app.emit("extension-content-received", &formatted) {
        tracing::error!("Failed to emit extension content: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Log télémétrique local
    log_telemetry_event(&payload);

    Ok(Json(ApiResponse { 
        ok: true, 
        message: Some("Content injected into GRAVIS chat successfully".to_string()) 
    }))
}

/// Handler pour recevoir le contenu de l'extension (legacy JSON)
async fn handle_extension_content(
    app: AppHandle,
    security: Arc<ExtensionSecurity>,
    Json(mut payload): Json<SecureExtractedContent>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let client_ip = addr.ip().to_string();
    
    tracing::info!("📥 Extension content request from: {} ({})", payload.url, client_ip);
    
    // Validation sécurisée
    if let Err(error) = security.validate_request(&payload, &client_ip) {
        tracing::warn!("❌ Security validation failed: {}", error);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Sanitisation du contenu avec ammonia (préserve UTF-8)
    payload.main_content = sanitize_content_secure(&payload.main_content);

    // Formatage pour GRAVIS chat
    let extraction_source = if payload.flags.is_pdf {
        "📄 PDF"
    } else if payload.flags.is_paywalled {
        "🔒 Paywall"
    } else if payload.extraction_method.contains("selection") {
        "✂️ Sélection"
    } else {
        "🌐 Page"
    };

    let formatted = format!(
        "{} extrait de **{}**\n🔗 {}\n\n{}\n\n**Question à propos de ce contenu :** ",
        extraction_source,
        payload.title,
        payload.url,
        payload.main_content
    );

    tracing::info!("📄 Content formatted: {} chars from {}", 
                   formatted.len(), payload.extraction_method);

    // Émettre vers le frontend
    if let Err(e) = app.emit("extension-content-received", &formatted) {
        tracing::error!("Failed to emit extension content: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Log télémétrique local
    log_telemetry_event(&payload);

    Ok(Json(ApiResponse { 
        ok: true, 
        message: Some("Content injected into GRAVIS chat successfully".to_string()) 
    }))
}

/// Endpoint ping avec token sécurisé
async fn ping_with_token(
    security: Arc<ExtensionSecurity>
) -> Json<PingResponse> {
    Json(PingResponse {
        ok: true,
        token: security.get_token(),
        exp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600, // 1h expiry
        message: Some("GRAVIS Extension API Ready".to_string())
    })
}

/// Sanitisation sécurisée avec ammonia (préserve UTF-8)
fn sanitize_content_secure(content: &str) -> String {
    ammonia::Builder::default()
        .tags(std::collections::HashSet::new()) // Texte pur seulement
        .clean(content)
        .to_string()
}

/// CORS strict (localhost uniquement)
fn create_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://127.0.0.1".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE])
        .max_age(Duration::from_secs(300))
}

/// Télémétrie locale privacy-first
fn log_telemetry_event(payload: &SecureExtractedContent) {
    let domain = payload.url.split('/').nth(2).unwrap_or("unknown");
    
    tracing::info!(
        "📊 Telemetry: method={}, domain={}, size={}, pdf={}, paywall={}",
        payload.extraction_method,
        domain,
        payload.main_content.len(),
        payload.flags.is_pdf,
        payload.flags.is_paywalled
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_security_creation() {
        let security = ExtensionSecurity::new();
        assert!(!security.get_token().is_empty());
    }

    #[test]
    fn test_rate_limiting() {
        let security = ExtensionSecurity::new();
        
        // Premier appel OK
        assert!(security.check_rate_limit("127.0.0.1"));
        
        // 10 appels rapides
        for _ in 0..9 {
            assert!(security.check_rate_limit("127.0.0.1"));
        }
        
        // 11ème appel rejeté
        assert!(!security.check_rate_limit("127.0.0.1"));
    }
}