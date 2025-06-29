import { setData, setVariable, readData } from "./desmos.js";
import { API_ENDPOINT_URL } from "./config.js"

let data;
let sampleData;
let activeData;
let activeOrbit;

const sampleDataCheck = document.getElementById("sample-data");
const fileInput = document.getElementById("file-input");
const keyDropdown = document.getElementById("key-dropdown");
const optimizeButton = document.getElementById("optimize");
const periodLowInput = document.getElementById("period-low");
const periodHighInput = document.getElementById("period-high");

const downloadButton = document.getElementById("download");

var periodLow = parseFloat(periodLowInput.value);
var periodHigh = parseFloat(periodHighInput.value);

const parameterMap = {
    'semi-major':   { variable: 'a',       index: 0, scale: 1             },
    'eccentricity': { variable: 'e_{0}',   index: 1, scale: 1             },
    'inclination':  { variable: 'i',       index: 2, scale: 180 / Math.PI },
    'node':         { variable: '\\Omega', index: 3, scale: 180 / Math.PI },
    'periapsis':    { variable: '\\omega', index: 4, scale: 180 / Math.PI },
    'mean-anomaly': { variable: 'M_{0}',   index: 5, scale: 180 / Math.PI },
    'period':       { variable: 'p',       index: 6, scale: 1             }
};

function setOrbit(parameters) {
    for (let parameter in parameterMap) {
        let variable = parameterMap[parameter].variable;
        let index = parameterMap[parameter].index
        setVariable(variable, parameters[index]);
    }

    document.querySelectorAll('input[class="parameter"]').forEach(parameter => {
        let index = parameterMap[parameter.id].index;
        let scale = parameterMap[parameter.id].scale;
        parameter.value = "" + parameters[index] * scale;
    });

    updatePassage();
}

optimizeButton.addEventListener('click', async (_) => {
    console.log("beginning orbital fit");
    optimizeButton.disabled = true;
    data[activeOrbit]['data'] = readData();
    activeData = data[activeOrbit]['data'];
    const response = await fetch(API_ENDPOINT_URL, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: "{\"data\": " + JSON.stringify(activeData) + ", \"periodBound\": [" + periodLow + ", " + periodHigh + "]}"
    });

    if (!response.ok) {
        throw new Error(`HTTP error with status: ${response.status}`);
    }

    const parameters = await response.json();
    setVariable("s_{howErrorLines}", 1);
    setOrbit(parameters);

    console.log("fitted!");
    optimizeButton.disabled = false;
});

fileInput.addEventListener('change', (event) => {
    const file = event.target.files[0];

    if (file) {
        const reader = new FileReader();
        reader.onload = (e) => {
            try {
                data = JSON.parse(e.target.result);
                populateDropdown();
            } catch {
                alert("invalid json file");
            }
        }

        reader.onerror = () => {
            alert("error reading file");
        }

        reader.readAsText(file);
    }
});

function populateDropdown() {
    keyDropdown.innerHTML = '<option value="">Select an Orbit</option>';
    Object.keys(data).forEach((key) => {
        const option = document.createElement('option');
        option.value = key;
        option.textContent = key;
        keyDropdown.appendChild(option);
    });

    keyDropdown.disabled = false;
}

sampleDataCheck.addEventListener('change', (_) => {
    var usingSample = sampleDataCheck.checked;

    if (usingSample) {
        data = JSON.parse(JSON.stringify(sampleData));
        fileInput.hidden = true;
        populateDropdown();
    } else {
        data = undefined;
        fileInput.hidden = false;
        keyDropdown.options.length = 1;
        keyDropdown.disabled = true;
    }
});

keyDropdown.addEventListener('change', (event) => {
    activeOrbit = event.target.value

    if (activeOrbit && data[activeOrbit]) {
        activeData = data[activeOrbit]['data'];
        setOrbit([0, 0, 0, 0, 0, 0, 1]);
        setData(activeData);
        setVariable("s_{howErrorLines}", 0);
        setVariable("h_{ighlightedPoints}", "\\left[\\right]");
        optimizeButton.disabled = false;
    }
});

periodLowInput.addEventListener('change', (event) => {
    var value = event.target.value
    if (!isNaN(parseFloat(value))) {
        periodLow = parseFloat(value);
    }
    periodLowInput.value = "" + periodLow;
});

periodHighInput.addEventListener('change', (event) => {
    var value = event.target.value
    if (!isNaN(parseFloat(value))) {
        periodHigh = parseFloat(value);
    }
    periodHighInput.value = "" + periodHigh;
});

const pointManipulationModes = {
    'retain-point':    { remove: 0, highlight: 0, flipX: 0, flipY: 0 },
    'remove-point':    { remove: 1, highlight: 0, flipX: 0, flipY: 0 },
    'highlight-point': { remove: 0, highlight: 1, flipX: 0, flipY: 0 },
    'flip-x':          { remove: 0, highlight: 0, flipX: 1, flipY: 0 },
    'flip-y':          { remove: 0, highlight: 0, flipX: 0, flipY: 1 },
    'flip-xy':         { remove: 0, highlight: 0, flipX: 1, flipY: 1 },
};

document.querySelectorAll('input[name="point-manip"]').forEach(radio => {
    radio.addEventListener('change', (event) => {
        const mode = event.target.id;
        if (event.target.checked) {
            const settings = pointManipulationModes[mode];
            setVariable("r_{emove}", settings.remove);
            setVariable("h_{ighlight}", settings.highlight);
            setVariable("f_{lipX}", settings.flipX);
            setVariable("f_{lipY}", settings.flipY);
        }
    });
});

downloadButton.addEventListener('click', (event) => {
    data[activeOrbit]['data'] = readData();

    const jsonString = JSON.stringify(data, null, 4);
    const blob = new Blob([jsonString], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = "new_data";

    document.body.appendChild(a);

    a.click();

    document.body.removeChild(a);
});

document.querySelectorAll('input[class="parameter"]').forEach(parameter => {
    parameter.addEventListener('change', (event) => {
        var variable = parameterMap[parameter.id].variable;
        var scale = parameterMap[parameter.id].scale;
        var value = parseFloat(parameter.value / scale);
        setVariable(variable, value);
    });
});

var mean_anomaly = document.getElementById("mean-anomaly");
var passage = document.getElementById("passage");
var period = document.getElementById("period");

mean_anomaly.addEventListener('change', (_) => {
    updatePassage();
});

passage.addEventListener('change', (_) => {
    var passage_val = parseFloat(passage.value);
    var period_val = parseFloat(period.value);
    var anomaly = 360.0 / period_val * (2000.0 - passage_val);

    mean_anomaly.value = "" + anomaly;

    setVariable("M_{0}", anomaly);
});

function updatePassage() {
    var anomaly = parseFloat(mean_anomaly.value);
    var period_val = parseFloat(period.value);
    var passage_val = 2000.0 - period_val * anomaly / 360.0

    passage.value = "" + passage_val;
}

window.addEventListener("load", async () => {
    const timestamp = new Date().getTime(); // can remove after development
    const response_state = await fetch(`./sample-data.json?cache_bust=${timestamp}`);
    sampleData = await response_state.json();
    sampleDataCheck.dispatchEvent(new Event('change', { bubbles: true }));
});