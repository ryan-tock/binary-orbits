import json
import math
import numpy
from scipy.optimize import differential_evolution

NEWTON_ITERATIONS = 6

def calc_positions(data, sm, e, i, node, periapsis, m_0, p):
    node_angles = [math.cos(node - 3 * math.pi / 2), math.sin(node - 3 * math.pi / 2)]
    inclined_angle = math.cos(i)
    beta = e / (1 + math.sqrt(1 - e**2))
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

        predicted_positions.append([])
        predicted_positions[-1].append(r * (planar_angles[0] * node_angles[0] - inclined_angle * planar_angles[1] * node_angles[1]))
        predicted_positions[-1].append(r * (inclined_angle * planar_angles[1] * node_angles[0] + planar_angles[0] * node_angles[1]))
    
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

def lambda_handler(event, context):
    body = json.loads(event['body'])
    data = body['data']
    period_bound = body['periodBound']
    bounds = [(0,0), (0, 0.95), (0, math.pi), (0, 2 * math.pi), (0, 2 * math.pi), (0, 2 * math.pi), (period_bound[0], period_bound[1])]
    result = differential_evolution(calc_loss, bounds, args=(data,))
    parameters = result.x.tolist()
    _ = calc_loss(parameters, data) # to get the semi major axis from least squares regression

    return {
        'statusCode': 200,
        'body': json.dumps(parameters)
    }