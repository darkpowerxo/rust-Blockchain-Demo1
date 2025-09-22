use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
    routing::get,
    Router,
};
use serde_json::json;
use std::sync::Arc;

use crate::api::ApiState;

pub fn routes() -> Router<Arc<ApiState>> {
    Router::new()
        .route("/", get(get_api_docs))
        .route("/swagger", get(get_swagger_ui))
}

/// Get API documentation homepage
async fn get_api_docs(
    State(_state): State<Arc<ApiState>>,
) -> Result<Html<String>, StatusCode> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blockchain Demo API Documentation</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; background: #f8f9fa; }
        .container { max-width: 1200px; margin: 0 auto; background: white; padding: 40px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #2d3748; margin-bottom: 30px; }
        h2 { color: #4a5568; border-bottom: 2px solid #e2e8f0; padding-bottom: 10px; margin-top: 40px; }
        .endpoint-group { margin: 20px 0; }
        .endpoint { background: #f7fafc; padding: 15px; margin: 10px 0; border-radius: 6px; border-left: 4px solid #4299e1; }
        .method { font-weight: bold; color: white; padding: 4px 8px; border-radius: 4px; font-size: 12px; }
        .get { background: #38a169; }
        .post { background: #e53e3e; }
        .put { background: #d69e2e; }
        .delete { background: #e53e3e; }
        .description { color: #718096; margin-top: 8px; }
        .swagger-link { display: inline-block; background: #4299e1; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin-top: 20px; }
        .swagger-link:hover { background: #3182ce; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Blockchain Demo API</h1>
        <p>A comprehensive REST API for blockchain interactions including multi-chain support, DeFi protocols, DEX trading, wallet management, and advanced security features.</p>
        
        <a href="/docs/swagger" class="swagger-link">📖 Interactive API Documentation (Swagger UI)</a>
        
        <h2>🔗 Chain Management</h2>
        <div class="endpoint-group">
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/chains</code>
                <div class="description">List all supported blockchain networks</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/chains/switch</code>
                <div class="description">Switch to different blockchain network</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/chains/{chain_id}</code>
                <div class="description">Get detailed chain information</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/chains/{chain_id}/gas</code>
                <div class="description">Get current gas prices</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/chains/{chain_id}/balance/{address}</code>
                <div class="description">Get wallet balance on specific chain</div>
            </div>
        </div>

        <h2>💰 Wallet Management</h2>
        <div class="endpoint-group">
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/wallets/connect/metamask</code>
                <div class="description">Connect MetaMask wallet</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/wallets/connect/walletconnect</code>
                <div class="description">Connect via WalletConnect protocol</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/wallets/create/local</code>
                <div class="description">Create new local wallet</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/wallets/{address}/sign/transaction</code>
                <div class="description">Sign blockchain transaction</div>
            </div>
        </div>

        <h2>🔄 DEX Trading</h2>
        <div class="endpoint-group">
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/dex</code>
                <div class="description">List supported DEX protocols</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/dex/quote</code>
                <div class="description">Get swap quote from DEX</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/dex/swap</code>
                <div class="description">Execute token swap</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/dex/{dex}/liquidity/add</code>
                <div class="description">Add liquidity to pool</div>
            </div>
        </div>

        <h2>🏦 DeFi Protocols</h2>
        <div class="endpoint-group">
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/defi</code>
                <div class="description">List supported DeFi protocols</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/defi/{protocol}/supply</code>
                <div class="description">Supply assets to lending protocol</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/defi/{protocol}/borrow</code>
                <div class="description">Borrow assets from protocol</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/defi/{protocol}/stats</code>
                <div class="description">Get protocol statistics and TVL</div>
            </div>
        </div>

        <h2>🛡️ Security & Analytics</h2>
        <div class="endpoint-group">
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/security/analyze</code>
                <div class="description">Analyze transaction for security risks</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/security/report</code>
                <div class="description">Generate security analysis report</div>
            </div>
            <div class="endpoint">
                <span class="method post">POST</span> <code>/api/security/emergency/alert</code>
                <div class="description">Trigger emergency security alert</div>
            </div>
            <div class="endpoint">
                <span class="method get">GET</span> <code>/api/security/threats/{address}</code>
                <div class="description">Check address for known threats</div>
            </div>
        </div>
    </div>
</body>
</html>
"#;
    
    Ok(Html(html.to_string()))
}

/// Get OpenAPI specification
async fn get_openapi_spec(
    State(_state): State<Arc<ApiState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let spec = json!({
        "info": {
            "title": "Blockchain Demo API",
            "version": "1.0.0",
            "description": "Comprehensive REST API for blockchain interactions including multi-chain support, DeFi protocols, DEX trading, wallet management, and advanced security features."
        },
        "servers": [
            {
                "url": "http://localhost:3000/api",
                "description": "Local development server"
            }
        ],
        "paths": {
            "/chains": {
                "get": {
                    "tags": ["Chain Management"],
                    "summary": "List supported blockchain networks",
                    "responses": {
                        "200": {
                            "description": "List of supported chains",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "$ref": "#/components/schemas/ChainInfo"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/wallets/connect/metamask": {
                "post": {
                    "tags": ["Wallet Management"],
                    "summary": "Connect MetaMask wallet",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/WalletConnectionRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Wallet connected successfully"
                        }
                    }
                }
            },
            "/dex/quote": {
                "post": {
                    "tags": ["DEX Trading"],
                    "summary": "Get swap quote",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/SwapRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Swap quote returned"
                        }
                    }
                }
            },
            "/security/analyze": {
                "post": {
                    "tags": ["Security"],
                    "summary": "Analyze transaction security",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/SecurityAnalysisRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Security analysis completed"
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "ChainInfo": {
                    "type": "object",
                    "properties": {
                        "chain_id": {"type": "integer"},
                        "name": {"type": "string"},
                        "rpc_url": {"type": "string"},
                        "native_currency": {"type": "object"}
                    }
                },
                "WalletConnectionRequest": {
                    "type": "object",
                    "properties": {
                        "account_address": {"type": "string"},
                        "chain_id": {"type": "integer"}
                    }
                },
                "SwapRequest": {
                    "type": "object",
                    "properties": {
                        "token_in": {"type": "string"},
                        "token_out": {"type": "string"},
                        "amount_in": {"type": "string"}
                    }
                },
                "SecurityAnalysisRequest": {
                    "type": "object",
                    "properties": {
                        "transaction_hash": {"type": "string"},
                        "chain_id": {"type": "integer"}
                    }
                }
            }
        }
    });
    
    Ok(Json(spec))
}

/// Get Swagger UI
async fn get_swagger_ui(
    State(_state): State<Arc<ApiState>>,
) -> Result<Html<String>, StatusCode> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blockchain Demo API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui.css" />
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@4.15.5/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/docs/openapi.json',
            dom_id: '#swagger-ui',
            deepLinking: true,
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.presets.standalone
            ],
            plugins: [
                SwaggerUIBundle.plugins.DownloadUrl
            ]
        });
    </script>
</body>
</html>
"#;
    
    Ok(Html(html.to_string()))
}
