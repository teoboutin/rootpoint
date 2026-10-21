# Rejected designs of rootpoint-protocol

The designs not kept for rootpoint-protocol, and why: one level-three heading per design, each cited
by the design head whose argument covers it, in `register@rootpoint-protocol@design`.

### Commands as JSON-RPC requests `##json-rpc-commands`

`live`. Each command as a JSON-RPC request, an object with a method, its parameters and an id. It
is easier for a program to parse, and harder for a person or an agent to type by hand at a terminal
or in a script: `{"jsonrpc":"2.0","method":"split","params":{"node":"n2","axis":"beside","at":0.5},"id":1}`
against `split n2 beside 0.5`.
