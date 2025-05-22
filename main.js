import { set_orbit, set_variable } from "./desmos.js";

let data;
let activeData;

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

var periodLow = 2;
var periodHigh = 40;

export async function initOptimiziaton() {
    console.log("beginning fit");
    optimizeButton.disabled = true;
    fetch('https://ko2hf5sz9g.execute-api.us-west-2.amazonaws.com/process', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: "{\"data\": " + JSON.stringify(activeData) + ", \"periodBound\": [" + periodLow + ", " + periodHigh + "]}"
    })
    .then(response => response.json())
    .then(result => {
        set_orbit(activeData, result);
        set_variable("s_{howErrorLines}", 1);

        semiMajor.innerText = "" + result[0]
        eccentricity.innerText = "" + result[1]
        inclination.innerText = "" + (result[2] * 180 / Math.PI)
        node.innerText = "" + (result[3] * 180 / Math.PI)
        periapsis.innerText = "" + (result[4] * 180 / Math.PI)
        meanAnomaly.innerText = "" + (result[5] * 180 / Math.PI)
        period.innerText = "" + result[6]

        console.log("fitted!");
        optimizeButton.disabled = false;
    })
    .catch(error => {
        console.error('Error:', error);
        optimizeButton.disabled = false;
    });
}

fileInput.addEventListener('change', event => {
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

keyDropdown.addEventListener('change', (event) => {
    var active_orbit = event.target.value

    if (active_orbit && data[active_orbit]) {
        activeData = data[active_orbit]['data'];
        set_orbit(activeData, [0, 0, 0, 0, 0, 0, 1]);
        set_variable("s_{howErrorLines}", 0);
        set_variable("h_{ighlightedPoints}", "\\left[\\right]");
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
        set_variable("e_{nableDataFixing}", 0);
        set_variable("f_{lipX}", 0);
        set_variable("f_{lipY}", 0);
        set_variable("r_{emove}", 0);
        set_variable("h_{ighlight}", 0);
    }
});

removePoint.addEventListener('change', (event) => {
    if (removePoint.checked) {
        set_variable("e_{nableDataFixing}", 1);
        set_variable("f_{lipX}", 0);
        set_variable("f_{lipY}", 0);
        set_variable("r_{emove}", 1);
        set_variable("h_{ighlight}", 0);
    }
});

highlightPoint.addEventListener('change', (event) => {
    if (highlightPoint.checked) {
        set_variable("e_{nableDataFixing}", 1);
        set_variable("f_{lipX}", 0);
        set_variable("f_{lipY}", 0);
        set_variable("r_{emove}", 0);
        set_variable("h_{ighlight}", 1);
    }
});

flipX.addEventListener('change', (event) => {
    if (flipX.checked) {
        set_variable("e_{nableDataFixing}", 1);
        set_variable("f_{lipX}", 1);
        set_variable("f_{lipY}", 0);
        set_variable("r_{emove}", 0);
        set_variable("h_{ighlight}", 0);
    }
});

flipY.addEventListener('change', (event) => {
    if (flipY.checked) {
        set_variable("e_{nableDataFixing}", 1);
        set_variable("f_{lipX}", 0);
        set_variable("f_{lipY}", 1);
        set_variable("r_{emove}", 0);
        set_variable("h_{ighlight}", 0);
    }
});

flipXY.addEventListener('change', (event) => {
    if (flipXY.checked) {
        set_variable("e_{nableDataFixing}", 1);
        set_variable("f_{lipX}", 1);
        set_variable("f_{lipY}", 1);
        set_variable("r_{emove}", 0);
        set_variable("h_{ighlight}", 0);
    }
});