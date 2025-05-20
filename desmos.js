let state;
let calculator;


window.addEventListener("load", async () => {
    var elt = document.getElementById('calculator');
    calculator = Desmos.Calculator3D(elt);
    calculator.updateSettings({"expressions": false});

    const timestamp = new Date().getTime(); // can remove after development
    const response_state = await fetch(`./state.json?cache_bust=${timestamp}`);
    state = await response_state.json();
});

function read_variable(variable) {
    let expressions = state['expressions']['list']
    for (const i in expressions) {
        if (Object.hasOwn(expressions[i], 'latex')) {
            if (expressions[i]['latex'].startsWith(variable + "=")) {
                return expressions[i]['latex'].split("=")[1];
            }
        }
    }
}

function set_variable(variable, value) {
    let expressions = state['expressions']['list']
    for (const i in expressions) {
        if (Object.hasOwn(expressions[i], 'latex')) {
            if (expressions[i]['latex'].startsWith(variable + "=")) {
                expressions[i]['latex'] = expressions[i]['latex'].split("=")[0] + "=" + value.toString();
            }
        }
    }
}

function read_data() {
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

export function set_orbit(data, parameters) {
    set_variable("a", parameters[0]);
    set_variable("e_{0}", parameters[1]);
    set_variable("i", parameters[2]);
    set_variable("\\Omega", parameters[3]);
    set_variable("\\omega", parameters[4]);
    set_variable("M_{0}", parameters[5]);
    set_variable("p", parameters[6]);

    let t = [];
    let x = [];
    let y = [];
    let weights = [];
    let methods = [];

    for (const point of data) {
        t.push(point['t']);
        x.push(point['x']);
        y.push(point['y']);
        weights.push(point['weight']);
        methods.push(point['method']);
    }

    set_variable("t_{0}", "\\left[" + t.join(", ") + "\\right]");
    set_variable("x_{0}", "\\left[" + x.join(", ") + "\\right]");
    set_variable("y_{0}", "\\left[" + y.join(", ") + "\\right]");
    set_variable("w_{eights}", "\\left[" + weights.join(", ") + "\\right]");
    set_variable("m_{ethod}", "\\left[" + methods.join(", ") + "\\right]");

    calculator.setState(state);
}