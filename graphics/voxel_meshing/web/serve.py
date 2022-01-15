import http.server
import socketserver

Handler = http.server.SimpleHTTPRequestHandler
Handler.extensions_map.update({
  '.wasm': 'application/wasm',
  '.js': 'application/javascript',
})

socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("", 8000), Handler) as httpd:
  httpd.allow_reuse_address = True
  httpd.serve_forever()
