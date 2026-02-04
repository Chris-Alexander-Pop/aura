use crate::types::{error_codes, JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use serde_json;
use std::io::{self, BufRead, BufReader, Write};
use tokio::sync::mpsc;

pub struct RpcServer {
    request_tx: mpsc::UnboundedSender<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>,
    notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
}

impl RpcServer {
    pub fn new(
        request_tx: mpsc::UnboundedSender<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>,
        notification_tx: mpsc::UnboundedSender<JsonRpcNotification>,
    ) -> Self {
        Self {
            request_tx,
            notification_tx,
        }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;

            if bytes_read == 0 {
                // EOF
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Try to parse as JSON-RPC request
            match serde_json::from_str::<JsonRpcRequest>(line) {
                Ok(request) => {
                    // Validate JSON-RPC version
                    if request.jsonrpc != "2.0" {
                        if let Some(id) = request.id {
                            self.send_error(
                                id,
                                error_codes::INVALID_REQUEST,
                                "Invalid jsonrpc version".to_string(),
                            )?;
                        }
                        continue;
                    }

                    // Send request to handler
                    let (response_tx, mut response_rx) = mpsc::unbounded_channel();
                    if self.request_tx.send((request, response_tx)).is_err() {
                        break; // Handler dropped
                    }

                    // Wait for response
                    if let Some(response) = response_rx.recv().await {
                        self.send_response(&response)?;
                    }
                }
                Err(e) => {
                    // Try to parse as notification (no id field)
                    if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(line) {
                        if notification.jsonrpc == "2.0" {
                            let _ = self.notification_tx.send(notification);
                        }
                    } else {
                        // Invalid JSON, send parse error
                        self.send_error(
                            serde_json::Value::Null,
                            error_codes::PARSE_ERROR,
                            format!("Parse error: {}", e),
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    fn send_response(&self, response: &JsonRpcResponse) -> Result<()> {
        let json = serde_json::to_string(response)?;
        println!("{}", json);
        io::stdout().flush()?;
        Ok(())
    }

    fn send_error(
        &self,
        id: serde_json::Value,
        code: i32,
        message: String,
    ) -> Result<()> {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id: Some(id),
        };
        self.send_response(&response)
    }
}

pub fn create_error_response(
    id: Option<serde_json::Value>,
    code: i32,
    message: String,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
        id,
    }
}

pub fn create_success_response(
    id: Option<serde_json::Value>,
    result: serde_json::Value,
) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id,
    }
}
