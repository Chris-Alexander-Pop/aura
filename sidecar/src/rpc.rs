use crate::types::{error_codes, JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use serde_json;
use std::io::{self, BufRead, BufReader, Write};
use tokio::sync::mpsc;

pub struct RpcServer {
    request_tx: mpsc::UnboundedSender<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>,
}

impl RpcServer {
    pub fn new(
        request_tx: mpsc::UnboundedSender<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>,
    ) -> Self {
        Self { request_tx }
    }

    pub async fn run(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();
        self.run_on(&mut reader, &mut stdout).await
    }

    /// Process newline-delimited JSON-RPC on arbitrary readers/writers (used by tests).
    pub async fn run_on<R, W>(&self, reader: &mut R, writer: &mut W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line)?;

            if bytes_read == 0 {
                break;
            }

            if !self.process_line(line.trim_end(), writer).await? {
                break;
            }
        }

        Ok(())
    }

    /// Handle one JSON-RPC line. Returns `false` when the handler channel is closed.
    pub async fn process_line<W: Write>(&self, line: &str, writer: &mut W) -> Result<bool> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(true);
        }

        match serde_json::from_str::<JsonRpcRequest>(line) {
            Ok(request) => {
                if request.jsonrpc != "2.0" {
                    if let Some(id) = request.id {
                        Self::write_error(
                            writer,
                            id,
                            error_codes::INVALID_REQUEST,
                            "Invalid jsonrpc version".to_string(),
                        )?;
                    }
                    return Ok(true);
                }

                let (response_tx, mut response_rx) = mpsc::unbounded_channel();
                if self.request_tx.send((request, response_tx)).is_err() {
                    return Ok(false);
                }

                if let Some(response) = response_rx.recv().await {
                    Self::write_response(writer, &response)?;
                }
            }
            Err(e) => {
                Self::write_error(
                    writer,
                    serde_json::Value::Null,
                    error_codes::PARSE_ERROR,
                    format!("Parse error: {}", e),
                )?;
            }
        }

        Ok(true)
    }

    fn write_response<W: Write>(writer: &mut W, response: &JsonRpcResponse) -> Result<()> {
        let json = serde_json::to_string(response)?;
        writeln!(writer, "{}", json)?;
        writer.flush()?;
        Ok(())
    }

    fn write_error<W: Write>(
        writer: &mut W,
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
        Self::write_response(writer, &response)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_registry;
    use serde_json::json;
    use std::io::Cursor;
    use tokio::sync::mpsc;

    #[test]
    fn response_helpers_round_trip() {
        let ok = create_success_response(Some(json!(1)), json!({"v": 2}));
        assert!(ok.error.is_none());
        assert_eq!(ok.result.unwrap()["v"], 2);

        let err = create_error_response(Some(json!(2)), -32603, "boom".into());
        assert!(err.result.is_none());
        assert_eq!(err.error.unwrap().message, "boom");
    }

    #[tokio::test]
    async fn process_line_parse_error_writes_json_rpc_error() {
        let (request_tx, _request_rx) = mpsc::unbounded_channel();
        let server = RpcServer::new(request_tx);
        let mut out = Vec::new();
        server
            .process_line("not json", &mut out)
            .await
            .expect("process_line");
        let response: JsonRpcResponse = serde_json::from_slice(&out).unwrap();
        assert_eq!(response.error.unwrap().code, error_codes::PARSE_ERROR);
    }

    #[tokio::test]
    async fn process_line_invalid_jsonrpc_version() {
        let (request_tx, _request_rx) = mpsc::unbounded_channel();
        let server = RpcServer::new(request_tx);
        let mut out = Vec::new();
        let line = r#"{"jsonrpc":"1.0","method":"Sidecar.GetVersion","id":1}"#;
        server.process_line(line, &mut out).await.unwrap();
        let response: JsonRpcResponse = serde_json::from_slice(&out).unwrap();
        assert_eq!(
            response.error.unwrap().code,
            error_codes::INVALID_REQUEST
        );
    }

    #[tokio::test]
    async fn run_on_stdio_round_trip_get_version() {
        let (request_tx, mut request_rx) =
            mpsc::unbounded_channel::<(JsonRpcRequest, mpsc::UnboundedSender<JsonRpcResponse>)>();
        let registry = build_registry();
        tokio::spawn(async move {
            while let Some((request, response_tx)) = request_rx.recv().await {
                let id = request.id.clone();
                let result = registry.handle_request(request).await;
                let response = match result {
                    Ok(value) => create_success_response(id, value),
                    Err(e) => create_error_response(
                        id,
                        error_codes::INTERNAL_ERROR,
                        e.to_string(),
                    ),
                };
                let _ = response_tx.send(response);
            }
        });

        let server = RpcServer::new(request_tx);
        let input = r#"{"jsonrpc":"2.0","method":"Sidecar.GetVersion","id":1}"#;
        let mut reader = Cursor::new(input.as_bytes());
        let mut writer = Vec::new();
        server.run_on(&mut reader, &mut writer).await.unwrap();

        let line = std::str::from_utf8(&writer).unwrap().trim();
        let response: JsonRpcResponse = serde_json::from_str(line).unwrap();
        assert!(response.error.is_none());
        assert!(response.result.unwrap().get("version").is_some());
    }
}
