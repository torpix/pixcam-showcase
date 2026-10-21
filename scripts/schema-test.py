"""Independent JSON Schema validation of real MCP results. Requires jsonschema."""
import json
import subprocess
from jsonschema import Draft202012Validator

binary = "target/debug/pixcam-showcase"
tools = json.loads(subprocess.check_output([binary, "tool-schemas"]))["tools"]
requests = [
    {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": {"name": "schema-test", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
]
examples = {
    "inspect_features": [{}, {"face": "X_MAX", "diameter_mm": 6}],
    "explain_nc_cycle": [{"nc": "G0 X2\nG1 Y3 F100"}, {"nc": "G81 X2"}],
    "validate_coordinate_transform": [
        {"transform": {"rotation": [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
                       "translation_mm": [11, 23, 7]}, "point_mm": [2, 3, 4]},
        {"transform": {"rotation": [[-1, 0, 0], [0, 1, 0], [0, 0, 1]],
                       "translation_mm": [0, 0, 0]}, "point_mm": [1, 2, 3]},
    ],
}
schemas = []
for tool in tools:
    for key in ("inputSchema", "outputSchema"):
        Draft202012Validator.check_schema(tool[key])
    for args in examples[tool["name"]]:
        Draft202012Validator(tool["inputSchema"]).validate(args)
        requests.append({"jsonrpc": "2.0", "id": len(schemas) + 1, "method": "tools/call",
                         "params": {"name": tool["name"], "arguments": args}})
        schemas.append(tool["outputSchema"])
output = subprocess.check_output([binary, "mcp"], input=("\n".join(map(json.dumps, requests)) + "\n").encode())
responses = [json.loads(line) for line in output.splitlines()]
assert len(responses) == 7
for response, schema in zip(responses[1:], schemas):
    result = response["result"]
    assert result["isError"] is False
    Draft202012Validator(schema).validate(result["structuredContent"])
    assert json.loads(result["content"][0]["text"]) == result["structuredContent"]
print("PASS: three input/output schema pairs; six real stdio results including unsupported NC and invalid transform.")
