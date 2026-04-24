import argparse
import json
import os
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

from lambda_function import lambda_handler


def make_handler(static_dir):
    class OrbitHandler(SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=static_dir, **kwargs)

        def _cors(self):
            self.send_header("Access-Control-Allow-Origin", "*")
            self.send_header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
            self.send_header("Access-Control-Allow-Headers", "content-type")
            self.send_header("Access-Control-Max-Age", "300")

        def end_headers(self):
            self._cors()
            super().end_headers()

        def do_OPTIONS(self):
            self.send_response(204)
            self.end_headers()

        def do_POST(self):
            if self.path != "/process":
                self.send_response(404)
                self.end_headers()
                return

            length = int(self.headers.get("Content-Length", "0"))
            raw_body = self.rfile.read(length).decode("utf-8")

            try:
                result = lambda_handler({"body": raw_body}, None)
                status = result.get("statusCode", 200)
                body = result.get("body", "")
            except Exception as exc:
                status = 500
                body = json.dumps({"error": str(exc)})

            payload = body.encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, format, *args):
            print("[%s] %s" % (self.address_string(), format % args), flush=True)

    return OrbitHandler


def main():
    parser = argparse.ArgumentParser(description="Binary orbit optimization server")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8081)
    parser.add_argument("--static-dir", default=os.path.dirname(os.path.abspath(__file__)))
    args = parser.parse_args()

    handler = make_handler(args.static_dir)
    server = ThreadingHTTPServer((args.host, args.port), handler)
    print(f"serving {args.static_dir} at http://{args.host}:{args.port}/", flush=True)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        server.shutdown()


if __name__ == "__main__":
    main()
