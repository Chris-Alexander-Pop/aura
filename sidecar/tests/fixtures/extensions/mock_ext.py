#!/usr/bin/env python3
"""NDJSON JSON-RPC mock for sidecar extension tests. No host side effects."""
import json
import sys


def main() -> None:
    for raw in sys.stdin:
        line = raw.strip()
        if not line:
            continue
        req = json.loads(line)
        method = req.get("method")
        req_id = req.get("id")
        if method == "Extension.ListMethods":
            result = {"methods": ["Ext.Ping"]}
        elif method == "Ext.Ping":
            result = {"ok": True, "params": req.get("params")}
        else:
            print(
                json.dumps(
                    {
                        "jsonrpc": "2.0",
                        "id": req_id,
                        "error": {"code": -32601, "message": f"Method not found: {method}"},
                    }
                ),
                flush=True,
            )
            continue
        print(json.dumps({"jsonrpc": "2.0", "id": req_id, "result": result}), flush=True)


if __name__ == "__main__":
    main()
