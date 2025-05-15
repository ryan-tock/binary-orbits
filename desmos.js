export function read_variable(state, variable) {
    let expressions = state['expressions']['list']
    for (const i in expressions) {
        if (Object.hasOwn(expressions[i], 'latex')) {
            if (expressions[i]['latex'].startsWith(variable + "=")) {
                return expressions[i]['latex'].split("=")[1];
            }
        }
    }
}

export function set_variable(state, variable, value) {
    let expressions = state['expressions']['list']
    for (const i in expressions) {
        if (Object.hasOwn(expressions[i], 'latex')) {
            if (expressions[i]['latex'].startsWith(variable + "=")) {
                expressions[i]['latex'] = expressions[i]['latex'].split("=")[0] + "=" + value.toString();
            }
        }
    }
}

export function read_data(state) {
    var data = [];

    let t = JSON.parse(read_variable(state, 't_{0}').split("\\left")[1].split("\\right")[0] + "]");
    let x = JSON.parse(read_variable(state, 'x_{0}').split("\\left")[1].split("\\right")[0] + "]");
    let y = JSON.parse(read_variable(state, 'y_{0}').split("\\left")[1].split("\\right")[0] + "]");
    let weights = JSON.parse(read_variable(state, 'w_{eights}').split("\\left")[1].split("\\right")[0] + "]");
    let methods = JSON.parse(read_variable(state, 'm_{ethod}').split("\\left")[1].split("\\right")[0] + "]");

    for (const i in t) {
        data.push({
            't': t[i],
            'x': x[i],
            'y': y[i],
            'weight': weights[i],
            'method': methods[i]
        })
    }

    return data;
}

export function set_orbit(state, parameters) {
    set_variable(state, "a", parameters[0]);
    set_variable(state, "e_{0}", parameters[1]);
    set_variable(state, "i", parameters[2]);
    set_variable(state, "\\Omega", parameters[3]);
    set_variable(state, "\\omega", parameters[4]);
    set_variable(state, "M_{0}", parameters[5]);
    set_variable(state, "p", parameters[6]);
}