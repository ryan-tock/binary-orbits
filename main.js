import { setOrbit, setVariable, readData } from "./desmos.js";

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

const retainPoint = document.getElementById("retain-point");
const removePoint = document.getElementById("remove-point");
const highlightPoint = document.getElementById("highlight-point");
const flipX = document.getElementById("flip-x");
const flipY = document.getElementById("flip-y");
const flipXY = document.getElementById("flip-xy");

const downloadButton = document.getElementById("download");

var periodLow = 2;
var periodHigh = 40;

optimizeButton.addEventListener('click', (_) => {
    console.log("beginning fit");
    optimizeButton.disabled = true;
    data[activeOrbit]['data'] = readData();
    activeData = data[activeOrbit]['data'];
    fetch('https://ko2hf5sz9g.execute-api.us-west-2.amazonaws.com/process', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: "{\"data\": " + JSON.stringify(activeData) + ", \"periodBound\": [" + periodLow + ", " + periodHigh + "]}"
    })
    .then(response => response.json())
    .then(result => {
        var parameters = result['parameters'];
        var r_squared = result['r_squared'];
        setOrbit(activeData, parameters, r_squared);
        setVariable("s_{howErrorLines}", 1);

        semiMajor.innerText = "" + parameters[0]
        eccentricity.innerText = "" + parameters[1]
        inclination.innerText = "" + (parameters[2] * 180 / Math.PI)
        node.innerText = "" + (parameters[3] * 180 / Math.PI)
        periapsis.innerText = "" + (parameters[4] * 180 / Math.PI)
        meanAnomaly.innerText = "" + (parameters[5] * 180 / Math.PI)
        period.innerText = "" + parameters[6]

        console.log("fitted!");
        optimizeButton.disabled = false;
    })
    .catch(error => {
        console.error('Error:', error);
        optimizeButton.disabled = false;
    });
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

retainPoint.addEventListener('change', (event) => {
    if (retainPoint.checked) {
        setVariable("e_{nableDataFixing}", 0);
        setVariable("f_{lipX}", 0);
        setVariable("f_{lipY}", 0);
        setVariable("r_{emove}", 0);
        setVariable("h_{ighlight}", 0);
    }
});

removePoint.addEventListener('change', (event) => {
    if (removePoint.checked) {
        setVariable("e_{nableDataFixing}", 1);
        setVariable("f_{lipX}", 0);
        setVariable("f_{lipY}", 0);
        setVariable("r_{emove}", 1);
        setVariable("h_{ighlight}", 0);
    }
});

highlightPoint.addEventListener('change', (event) => {
    if (highlightPoint.checked) {
        setVariable("e_{nableDataFixing}", 1);
        setVariable("f_{lipX}", 0);
        setVariable("f_{lipY}", 0);
        setVariable("r_{emove}", 0);
        setVariable("h_{ighlight}", 1);
    }
});

flipX.addEventListener('change', (event) => {
    if (flipX.checked) {
        setVariable("e_{nableDataFixing}", 1);
        setVariable("f_{lipX}", 1);
        setVariable("f_{lipY}", 0);
        setVariable("r_{emove}", 0);
        setVariable("h_{ighlight}", 0);
    }
});

flipY.addEventListener('change', (event) => {
    if (flipY.checked) {
        setVariable("e_{nableDataFixing}", 1);
        setVariable("f_{lipX}", 0);
        setVariable("f_{lipY}", 1);
        setVariable("r_{emove}", 0);
        setVariable("h_{ighlight}", 0);
    }
});

flipXY.addEventListener('change', (event) => {
    if (flipXY.checked) {
        setVariable("e_{nableDataFixing}", 1);
        setVariable("f_{lipX}", 1);
        setVariable("f_{lipY}", 1);
        setVariable("r_{emove}", 0);
        setVariable("h_{ighlight}", 0);
    }
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