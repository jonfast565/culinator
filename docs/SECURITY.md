# Local service security

The local HTTP server is reachable only on IPv4 loopback and uses an ephemeral port. Its API is protected by a random per-launch bearer token and exact origin validation.

The token is held in memory and injected into the current WebView. It is not written to SQLite or application logs. CORS allows only configured origins, methods, and the `Authorization` and `Content-Type` headers.

This design reduces accidental access from unrelated browser pages. It is not intended to defend against malware already executing as the same operating-system user. Remote binding must not be enabled without replacing the launch-token model with a full authentication and TLS design.

## WebSocket authentication

The GUI authenticates the WebSocket handshake with two offered subprotocols: the stable
`culinograph.v1` protocol and a per-launch `culinograph.auth.<token>` value. The server exact-matches
the request Origin and compares the token in constant time before upgrading. It selects only the
stable protocol in the response. The socket binds to loopback on an ephemeral port and the token is
never persisted.
