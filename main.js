import { set_orbit } from "./desmos.js";

let data;
let activeData;

export async function initOptimiziaton() {
    console.log("beggining fit");
    fetch('https://ko2hf5sz9g.execute-api.us-west-2.amazonaws.com/process', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(activeData)
        })
    .then(response => response.json())
    .then(result => {
        set_orbit(activeData, result);
        console.log("fitted!");
    })
    .catch(error => console.error('Error:', error));
}

const fileInput = document.getElementById("file-input");
const keyDropdown = document.getElementById("key-dropdown");
const optimizeButton = document.getElementById("optimize");

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
    // if (key && key != "Select an Orbit") {
    //     jsonData[key]['data'] = data;
    // }
    // changeBox.value = "";
    var active_orbit = event.target.value

    if (active_orbit && data[active_orbit]) {
        activeData = data[active_orbit]['data'];
        set_orbit(activeData, [0, 0, 0, 0, 0, 0, 1]);
        optimizeButton.disabled = false;
    }
});