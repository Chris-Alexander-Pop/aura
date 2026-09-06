#!/usr/bin/env bash
# Generate sidecar/openapi.json from rpc-manifest.json (run after generate-rpc-manifest.sh).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MANIFEST="$ROOT/sidecar/rpc-manifest.json"
OUT="$ROOT/sidecar/openapi.json"

if [[ ! -f "$MANIFEST" ]]; then
  echo "Missing $MANIFEST — run ./scripts/generate-rpc-manifest.sh first"
  exit 1
fi

VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/sidecar/Cargo.toml" | head -1)"
VERSION="${VERSION:-0.1.0}"

jq -n \
  --arg version "$VERSION" \
  --slurpfile m "$MANIFEST" \
  '
  ($m[0]) as $methods |
  ({
    "200": {
      description: "RPC success",
      content: {
        "application/json": {
          schema: {
            type: "object",
            required: ["ok", "data"],
            properties: {
              ok: { type: "boolean", enum: [true] },
              data: { description: "Method-specific result" }
            }
          }
        }
      }
    },
    "401": {
      description: "Missing or invalid X-Aura-Token",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" },
              code: { type: "string", enum: ["unauthorized"] }
            }
          }
        }
      }
    },
    "403": {
      description: "Non-loopback Origin",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" },
              code: { type: "string", enum: ["forbidden_origin"] }
            }
          }
        }
      }
    },
    "405": {
      description: "GET is not accepted for RPC",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" },
              code: { type: "string", enum: ["method_not_allowed"] }
            }
          }
        }
      }
    },
    "404": {
      description: "Unknown method",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" },
              code: { type: "string", enum: ["method_not_found"] },
              method: { type: "string" }
            }
          }
        }
      }
    },
    "500": {
      description: "Handler error",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" }
            }
          }
        }
      }
    }
  }) as $responses |
  {
    openapi: "3.1.0",
    info: {
      title: "Aura Sidecar API",
      version: $version,
      description: ("HTTP bridge for ags-sidecar JSON-RPC methods (" + ($methods | length | tostring) + " registered). RPC is POST-only with header `X-Aura-Token` (from `GET /api/meta`). Path example: `/api/Power.GetBatteryState`. Push: WebSocket `/ws?token=…`. Interactive docs: `/docs`. Loopback bind only.")
    },
    servers: [
      { url: "http://127.0.0.1:9080", description: "Default local bind (DEFAULT_HTTP_ADDR)" }
    ],
    tags: (
      ["Meta", "RPC"]
      + ($methods | map(split(".")[0]) | unique | sort)
      | map({ name: . })
    ),
    paths: {
      "/api/meta": {
        get: {
          operationId: "getMeta",
          tags: ["Meta"],
          summary: "Server metadata",
          description: "Sidecar version, loaded extras, and the HTTP RPC token for this process.",
          responses: {
            "200": {
              description: "Meta",
              content: {
                "application/json": {
                  schema: {
                    type: "object",
                    required: ["version", "http_token"],
                    properties: {
                      version: { type: "string" },
                      extensions: { type: "array" },
                      http_token: { type: "string", description: "Send as X-Aura-Token on POST /api/{method} and as ?token= on /ws" }
                    }
                  }
                }
              }
            }
          }
        }
      },
      "/api/{method}": {
        get: {
          operationId: "callRpcGetRejected",
          tags: ["RPC"],
          summary: "Rejected — RPC is POST-only",
          parameters: [
            {
              name: "method",
              in: "path",
              required: true,
              schema: { type: "string", enum: $methods }
            }
          ],
          responses: {
            "405": $responses["405"]
          }
        },
        post: {
          operationId: "callRpcPost",
          tags: ["RPC"],
          summary: "Invoke RPC",
          description: "Requires header X-Aura-Token (from GET /api/meta). Empty body for parameterless methods.",
          parameters: [
            {
              name: "method",
              in: "path",
              required: true,
              schema: { type: "string", enum: $methods },
              example: "Storage.Set"
            }
          ],
          requestBody: {
            required: false,
            content: {
              "application/json": {
                schema: {
                  type: "object",
                  additionalProperties: true,
                  description: "Method-specific parameters"
                }
              }
            }
          },
          responses: $responses
        }
      }
    },
    "x-rpc-methods": $methods,
    "x-websocket": {
      "/ws": {
        description: "Subscribe-only WebSocket. Server sends JSON text: `{ \"method\": \"Namespace.Event\", \"params\": { ... } }`."
      }
    }
  }
  ' > "$OUT"

COUNT="$(jq '.["x-rpc-methods"] | length' "$OUT")"
echo "Wrote $OUT ($COUNT RPC methods; browse at http://127.0.0.1:9080/docs)"
