import json
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

class MockSearxngHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Mute standard logging output to keep console clean
        pass

    def do_GET(self):
        parsed_url = urlparse(self.path)
        path = parsed_url.path
        
        if path == '/healthz':
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain')
            self.end_headers()
            self.wfile.write(b"OK")
            return
            
        if path == '/search':
            query_params = parse_qs(parsed_url.query)
            q = query_params.get('q', [''])[0]
            categories = query_params.get('categories', ['general'])[0]
            
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            
            if "images" in categories:
                response_data = {
                    "results": [
                        {
                            "title": f"Sailor Moon Cover - Query: {q}",
                            "url": "https://en.wikipedia.org/wiki/Sailor_Moon",
                            "content": "Sailor Moon Volume 1 cover artwork",
                            "category": "images",
                            "thumbnail_src": "https://picsum.photos/id/102/300/300",
                            "img_src": "https://picsum.photos/id/102/600/600"
                        },
                        {
                            "title": f"Sailor Mercury - Query: {q}",
                            "url": "https://en.wikipedia.org/wiki/Sailor_Mercury",
                            "content": "Sailor Mercury character portrait",
                            "category": "images",
                            "thumbnail_src": "https://picsum.photos/id/203/300/300",
                            "img_src": "https://picsum.photos/id/203/600/600"
                        },
                        {
                            "title": f"Sailor Mars - Query: {q}",
                            "url": "https://en.wikipedia.org/wiki/Sailor_Mars",
                            "content": "Sailor Mars character portrait",
                            "category": "images",
                            "thumbnail_src": "https://picsum.photos/id/304/300/300",
                            "img_src": "https://picsum.photos/id/304/600/600"
                        }
                    ]
                }
            else:
                response_data = {
                    "results": [
                        {
                            "title": f"Metasearch result 1 for query: '{q}'",
                            "url": "https://example.com/details-1",
                            "content": f"Here is high-quality aggregated context regarding the search term '{q}'. This is served by the local mock SearXNG mock server.",
                            "category": categories
                        },
                        {
                            "title": f"Metasearch result 2 for query: '{q}'",
                            "url": "https://example.com/details-2",
                            "content": f"More details and simulated search info for '{q}'. Local testing is functional and fast.",
                            "category": categories
                        }
                    ]
                }
            self.wfile.write(json.dumps(response_data).encode('utf-8'))
            return
            
        self.send_response(404)
        self.end_headers()

def run(port=8888):
    server_address = ('127.0.0.1', port)
    httpd = HTTPServer(server_address, MockSearxngHandler)
    print(f"Mock SearXNG server listening on http://127.0.0.1:{port}")
    httpd.serve_forever()

if __name__ == '__main__':
    run()
