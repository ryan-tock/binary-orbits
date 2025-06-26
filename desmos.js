let calculator;
let rSquaredElem = document.getElementById("r_squared");

window.onload = async () => {
    if (typeof Desmos == 'undefined') {
        return;
    }
    
    var elt = document.getElementById('calculator');
    calculator = Desmos.Calculator3D(elt);
    calculator.updateSettings({"expressions": false});
    const timestamp = new Date().getTime(); // can remove after development
    const response_state = await fetch(`./state.json?cache_bust=${timestamp}`);
    var state = await response_state.json();
    calculator.setState(state);

    var rSquared = calculator.HelperExpression({latex: 'R_{squared}'});
    rSquared.observe('numericValue', function() {
        rSquaredElem.innerText = "" + rSquared.numericValue;
    });
}

function readVariable(variable, expressions) {
    for (const i in expressions) {
        if (expressions[i].id == variable) {
            return expressions[i].latex.split("=")[1];
        }
    }
}

export function setVariable(variable, value) {
    calculator.setExpression({ id: variable, latex: variable + "=" + value });
}

export function readData() {
    var expressions = calculator.getExpressions();

    var data = [];

    let t = JSON.parse(readVariable('t_{0}', expressions).split("\\left")[1].split("\\right")[0] + "]");
    let x = JSON.parse(readVariable('x_{0}', expressions).split("\\left")[1].split("\\right")[0] + "]");
    let y = JSON.parse(readVariable('y_{0}', expressions).split("\\left")[1].split("\\right")[0] + "]");
    let weights = JSON.parse(readVariable('w_{eights}', expressions).split("\\left")[1].split("\\right")[0] + "]");
    let methods = JSON.parse(readVariable('m_{ethods}', expressions).split("\\left")[1].split("\\right")[0] + "]");

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

export function setOrbit(data, parameters) {
    setVariable("a", parameters[0]);
    setVariable("e_{0}", parameters[1]);
    setVariable("i", parameters[2]);
    setVariable("\\Omega", parameters[3]);
    setVariable("\\omega", parameters[4]);
    setVariable("M_{0}", parameters[5]);
    setVariable("p", parameters[6]);

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

    setVariable("t_{0}", "\\left[" + t.join(", ") + "\\right]");
    setVariable("x_{0}", "\\left[" + x.join(", ") + "\\right]");
    setVariable("y_{0}", "\\left[" + y.join(", ") + "\\right]");
    setVariable("w_{eights}", "\\left[" + weights.join(", ") + "\\right]");
    setVariable("m_{ethods}", "\\left[" + methods.join(", ") + "\\right]");
}