import { set_orbit, set_variable } from "./desmos.js";

let data;
let activeData;

const fileInput = document.getElementById("file-input");
const keyDropdown = document.getElementById("key-dropdown");
const optimizeButton = document.getElementById("optimize");
const periodLowInput = document.getElementById("period-low");
const periodHighInput = document.getElementById("period-high");

var periodLow = 5;
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