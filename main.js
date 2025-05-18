import { set_orbit } from "./desmos.js";

let data;
let state;
let calculator

async function readData() {
    const timestamp = new Date().getTime();
    const response_data = await fetch(`./binary-data.json?cache_bust=${timestamp}`);
    data = await response_data.json();
    data = data['00022+2705 BU  733AB']['data'];

    const response_state = await fetch(`./state.json?cache_bust=${timestamp}`);
    state = await response_state.json();
}

async function loadCalc() {
    var elt = document.getElementById('calculator');
    calculator = Desmos.Calculator3D(elt);

    calculator.setState(state);
    calculator.updateSettings({"expressions": false});
}

export async function initOptimiziaton() {
    fetch('https://ko2hf5sz9g.execute-api.us-west-2.amazonaws.com/process', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(data)
        })
    .then(response => response.json())
    .then(result => {
        set_orbit(state, data, result);
        calculator.setState(state);
        console.log("fitted!");
    })
    .catch(error => console.error('Error:', error));
}

window.addEventListener("load", async () => {
    await readData();
    await loadCalc();
});
