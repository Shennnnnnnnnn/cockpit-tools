use crate::models::codex::{CodexAccount, CodexTokens};
use crate::modules::{codex_account, codex_oauth, codex_quota, logger, websocket};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use tauri::AppHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};

const CODEX_OAUTH_RPC_BIND_ADDR: &str = "127.0.0.1:1466";
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);
const CORS_ALLOW_HEADERS: &str = "Content-Type, Authorization, X-API-Key";

#[derive(Debug)]
struct RpcRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

#[derive(Debug, Deserialize)]
struct CallbackPayload {
    #[serde(alias = "loginId")]
    login_id: String,
    #[serde(alias = "callbackUrl")]
    callback_url: String,
}

fn is_loopback_addr(addr: &SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(ip) => ip.is_loopback(),
        IpAddr::V6(ip) => ip.is_loopback(),
    }
}

fn parse_http_request(raw: &[u8]) -> Result<RpcRequest, String> {
    let header_end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| "HTTP 请求头不完整".to_string())?;
    let header_text = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| "HTTP 请求头不是有效 UTF-8".to_string())?;
    let mut lines = header_text.split("\r\n");
    let request_line = lines.next().ok_or_else(|| "HTTP 请求行缺失".to_string())?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("HTTP 请求行格式无效".to_string());
    }

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let body_start = header_end + 4;
    if raw.len().saturating_sub(body_start) < content_length {
        return Err("HTTP 请求体不完整".to_string());
    }

    Ok(RpcRequest {
        method: parts[0].to_ascii_uppercase(),
        path: parts[1].split('?').next().unwrap_or(parts[1]).to_string(),
        headers,
        body: raw[body_start..body_start + content_length].to_vec(),
    })
}

async fn read_request(stream: &mut TcpStream) -> Result<RpcRequest, String> {
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 8192];

    timeout(REQUEST_READ_TIMEOUT, async {
        loop {
            let read = stream
                .read(&mut temp)
                .await
                .map_err(|e| format!("读取 HTTP 请求失败: {}", e))?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);
            if buffer.len() > MAX_REQUEST_BYTES {
                return Err("HTTP 请求过大".to_string());
            }
            if let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                let header_text = std::str::from_utf8(&buffer[..header_end])
                    .map_err(|_| "HTTP 请求头不是有效 UTF-8".to_string())?;
                let content_length = header_text
                    .split("\r\n")
                    .filter_map(|line| line.split_once(':'))
                    .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-length"))
                    .and_then(|(_, value)| value.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                if buffer.len() >= header_end + 4 + content_length {
                    break;
                }
            }
        }
        parse_http_request(&buffer)
    })
    .await
    .map_err(|_| "读取 HTTP 请求超时".to_string())?
}

fn status_reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn json_response(status: u16, body: serde_json::Value) -> Vec<u8> {
    let body = body.to_string();
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        status_reason(status),
        CORS_ALLOW_HEADERS,
        body.as_bytes().len(),
        body
    )
    .into_bytes()
}

fn empty_response(status: u16) -> Vec<u8> {
    format!(
        "HTTP/1.1 {} {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        status,
        status_reason(status),
        CORS_ALLOW_HEADERS
    )
    .into_bytes()
}

async fn save_codex_oauth_tokens(tokens: CodexTokens) -> Result<CodexAccount, String> {
    let account = codex_account::upsert_account(tokens)?;
    if let Err(error) = codex_quota::refresh_account_quota(&account.id).await {
        logger::log_warn(&format!(
            "[CodexOAuthRpc] OAuth 账号已保存但刷新配额失败: account_id={}, error={}",
            account.id, error
        ));
    }
    let loaded =
        codex_account::load_account(&account.id).ok_or_else(|| "账号保存后无法读取".to_string())?;
    websocket::broadcast_data_changed("codex_oauth_rpc");
    Ok(loaded)
}

async fn handle_rpc_request(app_handle: AppHandle, request: RpcRequest) -> Vec<u8> {
    if request.method == "OPTIONS" {
        return empty_response(204);
    }

    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/api/codex/oauth/start") | ("POST", "/api/codex/oauth/start") => {
            match codex_oauth::start_oauth_login(app_handle).await {
                Ok(response) => json_response(200, json!(response)),
                Err(error) => json_response(500, json!({ "error": error })),
            }
        }
        ("POST", "/api/codex/oauth/callback") => {
            let payload = match serde_json::from_slice::<CallbackPayload>(&request.body) {
                Ok(payload) => payload,
                Err(error) => {
                    return json_response(
                        400,
                        json!({ "error": format!("请求体 JSON 无效: {}", error) }),
                    );
                }
            };
            if payload.login_id.trim().is_empty() || payload.callback_url.trim().is_empty() {
                return json_response(400, json!({ "error": "login_id 和 callback_url 不能为空" }));
            }
            if let Err(error) =
                codex_oauth::submit_callback_url(&payload.login_id, &payload.callback_url)
            {
                return json_response(400, json!({ "error": error }));
            }
            match codex_oauth::complete_oauth_login(&payload.login_id).await {
                Ok(tokens) => match save_codex_oauth_tokens(tokens).await {
                    Ok(account) => json_response(200, json!({ "account": account })),
                    Err(error) => json_response(500, json!({ "error": error })),
                },
                Err(error) => json_response(400, json!({ "error": error })),
            }
        }
        ("GET", "/health") => json_response(200, json!({ "ok": true })),
        (_, "/api/codex/oauth/start") | (_, "/api/codex/oauth/callback") => {
            json_response(405, json!({ "error": "method_not_allowed" }))
        }
        _ => json_response(404, json!({ "error": "not_found" })),
    }
}

async fn handle_connection(app_handle: AppHandle, mut stream: TcpStream, addr: SocketAddr) {
    if !is_loopback_addr(&addr) {
        let _ = stream
            .write_all(&json_response(403, json!({ "error": "loopback_only" })))
            .await;
        logger::log_warn(&format!("[CodexOAuthRpc] 拒绝非 loopback 客户端: {}", addr));
        return;
    }

    let response = match read_request(&mut stream).await {
        Ok(request) => {
            logger::log_info(&format!(
                "[CodexOAuthRpc] 收到请求: method={}, path={}, content_type={}",
                request.method,
                request.path,
                request
                    .headers
                    .get("content-type")
                    .map(String::as_str)
                    .unwrap_or("<none>")
            ));
            handle_rpc_request(app_handle, request).await
        }
        Err(error) => json_response(400, json!({ "error": error })),
    };

    let _ = stream.write_all(&response).await;
    let _ = stream.shutdown().await;
}

pub async fn start_server(app_handle: AppHandle) {
    let listener = match TcpListener::bind(CODEX_OAUTH_RPC_BIND_ADDR).await {
        Ok(listener) => listener,
        Err(error) => {
            logger::log_warn(&format!(
                "[CodexOAuthRpc] 无法绑定 {}: {}",
                CODEX_OAUTH_RPC_BIND_ADDR, error
            ));
            return;
        }
    };

    logger::log_info(&format!(
        "[CodexOAuthRpc] 服务已启动: http://{}",
        CODEX_OAUTH_RPC_BIND_ADDR
    ));

    while let Ok((stream, addr)) = listener.accept().await {
        let app_handle_clone = app_handle.clone();
        tokio::spawn(handle_connection(app_handle_clone, stream, addr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_callback_payload_with_snake_or_camel_case() {
        let snake: CallbackPayload = serde_json::from_str(
            r#"{"login_id":"login-1","callback_url":"http://localhost:1455/auth/callback?code=a&state=b"}"#,
        )
        .unwrap();
        assert_eq!(snake.login_id, "login-1");

        let camel: CallbackPayload = serde_json::from_str(
            r#"{"loginId":"login-2","callbackUrl":"http://localhost:1455/auth/callback?code=a&state=b"}"#,
        )
        .unwrap();
        assert_eq!(camel.login_id, "login-2");
        assert!(camel.callback_url.contains("/auth/callback"));
    }

    #[test]
    fn parses_http_request_path_headers_and_body() {
        let raw = b"POST /api/codex/oauth/callback?ignored=1 HTTP/1.1\r\nHost: 127.0.0.1:1466\r\nContent-Type: application/json\r\nContent-Length: 16\r\n\r\n{\"login_id\":\"x\"}";
        let request = parse_http_request(raw).unwrap();
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/api/codex/oauth/callback");
        assert_eq!(
            request.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert_eq!(request.body, br#"{"login_id":"x"}"#);
    }

    #[test]
    fn loopback_filter_accepts_only_loopback_addresses() {
        let loopback: SocketAddr = "127.0.0.1:51400".parse().unwrap();
        let lan: SocketAddr = "192.168.1.2:51400".parse().unwrap();
        assert!(is_loopback_addr(&loopback));
        assert!(!is_loopback_addr(&lan));
    }
}
