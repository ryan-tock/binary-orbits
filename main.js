import { setOrbit, setVariable, readData } from "./desmos.js";
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

const semiMajor = document.getElementById("semi-major");
const eccentricity = document.getElementById("eccentricity");
const inclination = document.getElementById("inclination");
const node = document.getElementById("node");
const periapsis = document.getElementById("periapsis");
const meanAnomaly = document.getElementById("mean-anomaly");
const period = document.getElementById("period");

const downloadButton = document.getElementById("download");

var periodLow = parseInt(periodLowInput.value);
var periodHigh = parseInt(periodHighInput.value);

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
    setOrbit(activeData, parameters);
    setVariable("s_{howErrorLines}", 1);

    semiMajor.value = "" + parameters[0]
    eccentricity.value = "" + parameters[1]
    inclination.value = "" + (parameters[2] * 180 / Math.PI)
    node.value = "" + (parameters[3] * 180 / Math.PI)
    periapsis.value = "" + (parameters[4] * 180 / Math.PI)
    meanAnomaly.value = "" + (parameters[5] * 180 / Math.PI)
    period.value = "" + parameters[6]

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
        setOrbit(activeData, [0, 0, 0, 0, 0, 0, 1], 0);
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
    'retain-point': { remove: 0, highlight: 0, flipX: 0, flipY: 0 },
    'remove-point': { remove: 1, highlight: 0, flipX: 0, flipY: 0 },
    'highlight-point': { remove: 0, highlight: 1, flipX: 0, flipY: 0 },
    'flip-x': { remove: 0, highlight: 0, flipX: 1, flipY: 0 },
    'flip-y': { remove: 0, highlight: 0, flipX: 0, flipY: 1 },
    'flip-xy': { remove: 0, highlight: 0, flipX: 1, flipY: 1 },
};

document.querySelectorAll('input[name="point-manip"]').forEach(radio => {
    radio.addEventListener('change', (event) => {
        const mode = event.target.id;
        if (event.target.checked && pointManipulationModes[mode]) {
            const settings = pointManipulationModes[mode];
            setVariable("r_{emove}", settings.remove);
            setVariable("h_{ighlight}", settings.highlight);
            setVariable("f_{lipX}", settings.flipX);
            setVariable("f_{lipY}", settings.flipY);
        }
    });
});

downloadButton.addEventListener('click', (event) => {
    readData();

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

window.addEventListener("load", async () => {
    const timestamp = new Date().getTime(); // can remove after development
    const response_state = await fetch(`./sample-data.json?cache_bust=${timestamp}`);
    sampleData = await response_state.json();
    sampleDataCheck.dispatchEvent(new Event('change', { bubbles: true }));
});