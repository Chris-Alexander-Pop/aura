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
    "429": {
      description: "Rate limited (offensive-security builds)",
      content: {
        "application/json": {
          schema: {
            type: "object",
            properties: {
              ok: { type: "boolean", enum: [false] },
              error: { type: "string" },
              code: { type: "string", enum: ["rate_limited"] }
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
      description: ("HTTP bridge for ags-sidecar JSON-RPC methods (" + ($methods | length | tostring) + " registered). GET for parameterless calls; POST with a JSON object body for params. Path example: `/api/Power.GetBatteryState`. Push: WebSocket `/ws`. Interactive docs: `/docs`.")
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
          description: "Sidecar version and whether `Security.Offensive.*` RPCs exist in this binary.",
          responses: {
            "200": {
              description: "Meta",
              content: {
                "application/json": {
                  schema: {
                    type: "object",
                    properties: {
                      version: { type: "string" },
                      offensiveEnabled: { type: "boolean" }
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
          operationId: "callRpcGet",
          tags: ["RPC"],
          summary: "Invoke RPC (no params)",
          parameters: [
            {
              name: "method",
              in: "path",
              required: true,
              description: "Full RPC name (dots allowed in one path segment).",
              schema: { type: "string", enum: $methods },
              example: "Power.GetBatteryState"
            }
          ],
          responses: $responses
        },
        post: {
          operationId: "callRpcPost",
          tags: ["RPC"],
          summary: "Invoke RPC (with params)",
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
