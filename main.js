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

// --- Background fit prefetch -----------------------------------------------
//
// Whenever the plotted data or period bounds change, kick off a fit in the
// background. When the user hits Optimize Orbit we just await whatever's
// already in flight (or the cached result) and apply it — so the common
// case feels instant. If the inputs change mid-fit we abort the stale
// request and start over.

let pendingFit = null;  // { snapshot, promise, abort }
let scheduleTimer = null;

function fitSnapshot() {
    if (!activeData || activeData.length === 0) return null;
    return JSON.stringify({ data: activeData, lo: periodLow, hi: periodHigh });
}

function startBackgroundFit() {
    scheduleTimer = null;
    const snapshot = fitSnapshot();
    if (snapshot === null) return;
    if (pendingFit && pendingFit.snapshot === snapshot) return;
    if (pendingFit) pendingFit.abort.abort();

    const abort = new AbortController();
    const body = `{"data": ${JSON.stringify(activeData)}, "periodBound": [${periodLow}, ${periodHigh}]}`;
    const promise = fetch(API_ENDPOINT_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body,
        signal: abort.signal,
    }).then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
    });

    pendingFit = { snapshot, promise, abort };
    promise.catch(err => {
        if (err.name !== 'AbortError') console.warn('background fit failed:', err);
    });
}

function scheduleBackgroundFit() {
    if (scheduleTimer) clearTimeout(scheduleTimer);
    scheduleTimer = setTimeout(startBackgroundFit, 250);
}

function syncActiveDataFromGraph() {
    if (!data || !activeOrbit) return false;
    try {
        const fresh = readData();
        const serialized = JSON.stringify(fresh);
        if (serialized === JSON.stringify(activeData)) return false;
        activeData = fresh;
        data[activeOrbit]['data'] = fresh;
        return true;
    } catch {
        return false;  // calculator not ready yet
    }
}

// Desmos mutations (point clicks in Remove / Highlight / Flip modes) don't
// fire DOM events we can hook into, so we poll instead. 300ms is quick
// enough to feel responsive without being a battery hog.
setInterval(() => {
    if (syncActiveDataFromGraph()) scheduleBackgroundFit();
}, 300);

// --- Orbit rendering -------------------------------------------------------

function setOrbit(parameters) {
    for (let parameter in parameterMap) {
        setVariable(parameterMap[parameter].variable, parameters[parameterMap[parameter].index]);
    }
    document.querySelectorAll('input[class="parameter"]').forEach(parameter => {
        if (parameter.id == "passage") return;
        const { index, scale } = parameterMap[parameter.id];
        parameter.value = "" + parameters[index] * scale;
    });
    updatePassage();
}

optimizeButton.addEventListener('click', async () => {
    optimizeButton.disabled = true;
    optimizeButton.innerText = "Fitting...";

    // Grab the latest data in case the user edited just before clicking.
    syncActiveDataFromGraph();

    // If we don't already have a fit in flight matching the current state,
    // fire one off synchronously so `await` has something to wait on.
    if (!pendingFit || pendingFit.snapshot !== fitSnapshot()) {
        if (scheduleTimer) { clearTimeout(scheduleTimer); scheduleTimer = null; }
        startBackgroundFit();
    }

    try {
        const parameters = await pendingFit.promise;
        setVariable("s_{howErrorLines}", 1);
        setOrbit(parameters);
    } catch (err) {
        console.error('fit failed:', err);
        alert(`Fit failed: ${err.message || err}`);
    } finally {
        optimizeButton.disabled = false;
        optimizeButton.innerText = "Optimize Orbit!";
    }
});

// --- Input wiring ----------------------------------------------------------

fileInput.addEventListener('change', (event) => {
    const file = event.target.files[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (e) => {
        try {
            data = JSON.parse(e.target.result);
            populateDropdown();
        } catch {
            alert("invalid json file");
        }
    };
    reader.onerror = () => alert("error reading file");
    reader.readAsText(file);
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

sampleDataCheck.addEventListener('change', () => {
    if (sampleDataCheck.checked) {
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
    activeOrbit = event.target.value;
    if (activeOrbit && data[activeOrbit]) {
        activeData = data[activeOrbit]['data'];
        setOrbit([0, 0, 0, 0, 0, 0, 1]);
        setData(activeData);
        setVariable("s_{howErrorLines}", 0);
        setVariable("h_{ighlightedPoints}", "\\left[\\right]");
        optimizeButton.disabled = false;
        scheduleBackgroundFit();
    }
});

function bindPeriodInput(input, getValue, setValue) {
    input.addEventListener('change', (event) => {
        const v = parseFloat(event.target.value);
        if (!isNaN(v)) setValue(v);
        input.value = "" + getValue();
        scheduleBackgroundFit();
    });
}
bindPeriodInput(periodLowInput,  () => periodLow,  v => { periodLow = v; });
bindPeriodInput(periodHighInput, () => periodHigh, v => { periodHigh = v; });

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
        if (!event.target.checked) return;
        const settings = pointManipulationModes[event.target.id];
        setVariable("r_{emove}",    settings.remove);
        setVariable("h_{ighlight}", settings.highlight);
        setVariable("f_{lipX}",     settings.flipX);
        setVariable("f_{lipY}",     settings.flipY);
    });
});

downloadButton.addEventListener('click', () => {
    data[activeOrbit]['data'] = readData();
    const blob = new Blob([JSON.stringify(data, null, 4)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = "new_data";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
});

document.querySelectorAll('input[class="parameter"]').forEach(parameter => {
    if (parameter.id == "passage") return;
    parameter.addEventListener('change', () => {
        const { variable, scale } = parameterMap[parameter.id];
        setVariable(variable, parseFloat(parameter.value / scale));
    });
});

const mean_anomaly = document.getElementById("mean-anomaly");
const passage = document.getElementById("passage");
const period = document.getElementById("period");
const usingPassage = document.getElementById("using-passage");

usingPassage.addEventListener('change', () => {
    mean_anomaly.parentNode.hidden = usingPassage.checked;
    passage.parentNode.hidden = !usingPassage.checked;
});

mean_anomaly.addEventListener('change', updatePassage);

passage.addEventListener('change', () => {
    const passage_val = parseFloat(passage.value);
    const period_val = parseFloat(period.value);
    const anomaly = 360.0 / period_val * (2000.0 - passage_val);
    mean_anomaly.value = "" + anomaly;
    setVariable("M_{0}", anomaly);
});

function updatePassage() {
    const anomaly = parseFloat(mean_anomaly.value);
    const period_val = parseFloat(period.value);
    passage.value = "" + (2000.0 - period_val * anomaly / 360.0);
}

window.addEventListener("load", async () => {
    const timestamp = new Date().getTime();
    const response_state = await fetch(`./sample-data.json?cache_bust=${timestamp}`);
    sampleData = await response_state.json();
    sampleDataCheck.dispatchEvent(new Event('change', { bubbles: true }));
});
