import argparse
import json
import math
import os
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

from scipy.optimize import differential_evolution

NEWTON_ITERATIONS = 6


def calc_positions(data, sm, e, i, node, periapsis, m_0, p):
    node_angles = [math.cos(node - 3 * math.pi / 2), math.sin(node - 3 * math.pi / 2)]
    inclined_angle = math.cos(i)
    beta = e / (1 + math.sqrt(1 - e ** 2))
    predicted_positions = []

    for point in data:
        t = point['t']

        mean_anomaly = m_0 + (2 * math.pi / p) * (t - 2000)

        eccentric_anomaly = mean_anomaly

        for _ in range(NEWTON_ITERATIONS):
            eccentric_anomaly += (mean_anomaly - eccentric_anomaly + e * math.sin(eccentric_anomaly)) / (1 - e * math.cos(eccentric_anomaly))

        true_anomaly = eccentric_anomaly + 2 * math.atan(beta * math.sin(eccentric_anomaly) / (1 - beta * math.cos(eccentric_anomaly)))

        r = sm * (1 - e * math.cos(eccentric_anomaly))
        planar_angles = [math.cos(true_anomaly + periapsis), math.sin(true_anomaly + periapsis)]

        predicted_positions.append([
            r * (planar_angles[0] * node_angles[0] - inclined_angle * planar_angles[1] * node_angles[1]),
            r * (inclined_angle * planar_angles[1] * node_angles[0] + planar_angles[0] * node_angles[1]),
        ])

    return predicted_positions


def calc_loss(parameters, data):
    predicted_positions = calc_positions(data, 1, parameters[1], parameters[2], parameters[3], parameters[4], parameters[5], parameters[6])
    parameter_squared = 0
    resultant = 0

    for point in range(len(data)):
        parameter_squared += predicted_positions[point][0] ** 2 * data[point]['weight']
        parameter_squared += predicted_positions[point][1] ** 2 * data[point]['weight']

        resultant += predicted_positions[point][0] * data[point]['x'] * data[point]['weight']
        resultant += predicted_positions[point][1] * data[point]['y'] * data[point]['weight']

    sm = resultant / parameter_squared
    if sm < 0:
        sm = 0

    parameters[0] = sm

    error = 0
    for index in range(len(data)):
        error += (data[index]['x'] - parameters[0] * predicted_positions[index][0]) ** 2 * data[index]['weight']
        error += (data[index]['y'] - parameters[0] * predicted_positions[index][1]) ** 2 * data[index]['weight']

    return error


def fit_orbit(data, period_bound):
    bounds = [(0, 0), (0, 0.95), (0, math.pi), (0, 2 * math.pi), (0, 2 * math.pi), (0, 2 * math.pi), (period_bound[0], period_bound[1])]
    result = differential_evolution(calc_loss, bounds, args=(data,))
    parameters = result.x.tolist()
    calc_loss(parameters, data)  # fills in the semi-major axis via least-squares
    return parameters


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
                body = json.loads(raw_body)
                parameters = fit_orbit(body["data"], body["periodBound"])
                status = 200
                payload = json.dumps(parameters).encode("utf-8")
            except Exception as exc:
                status = 500
                payload = json.dumps({"error": str(exc)}).encode("utf-8")

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
