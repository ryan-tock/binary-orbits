import json
import math
import numpy
from scipy.optimize import differential_evolution

NEWTON_ITERATIONS = 6

def calc_loss(parameters, data):
    e = parameters[1]
    i = parameters[2]
    node = parameters[3]
    periapsis = parameters[4]
    m_0 = parameters[5]
    p = parameters[6]

    node_angles = [math.cos(node - 3 * math.pi / 2), math.sin(node - 3 * math.pi / 2)]
    inclined_angle = math.cos(i)

    beta = e / (1 + math.sqrt(1 - e**2))

    predicted_positions = []
    parameter_squared = 0
    resultant = 0

    for point in data:
        t = point['t']
        x = point['x']
        y = point['y']
        weight = point['weight']

        mean_anomaly = m_0 + (2 * math.pi / p) * (t - 2000)

        eccentric_anomaly = mean_anomaly

        for _ in range(NEWTON_ITERATIONS):
            eccentric_anomaly += (mean_anomaly - eccentric_anomaly + e * math.sin(eccentric_anomaly)) / (1 - e * math.cos(eccentric_anomaly))

        true_anomaly = eccentric_anomaly + 2 * math.atan(beta * math.sin(eccentric_anomaly) / (1 - beta * math.cos(eccentric_anomaly)))

        r_scaled = 1 - e * math.cos(eccentric_anomaly)
        planar_angles = [math.cos(true_anomaly + periapsis), math.sin(true_anomaly + periapsis)]

        predicted_positions.append([])
        predicted_positions[-1].append(r_scaled * (planar_angles[0] * node_angles[0] - inclined_angle * planar_angles[1] * node_angles[1]))
        predicted_positions[-1].append(r_scaled * (inclined_angle * planar_angles[1] * node_angles[0] + planar_angles[0] * node_angles[1]))

        parameter_squared += predicted_positions[-1][0] ** 2 * weight
        parameter_squared += predicted_positions[-1][1] ** 2 * weight

        resultant += predicted_positions[-1][0] * x * weight
        resultant += predicted_positions[-1][1] * y * weight
    
    sm = resultant / parameter_squared
    if sm < 0:
        sm = 0
    
    parameters[0] = sm

    error = 0
    for index in range(len(data)):
        error += (data[index]['x'] - sm * predicted_positions[index][0]) ** 2 * data[index]['weight']
        error += (data[index]['y'] - sm * predicted_positions[index][1]) ** 2 * data[index]['weight']

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
