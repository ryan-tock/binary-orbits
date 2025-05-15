let pyodide;

async function initializePyodide() {
    pyodide = await loadPyodide();
    console.log("Pyodide loaded");
}

async function loadPythonScript() {
    await pyodide.loadPackage("numpy");
    await pyodide.loadPackage("scipy");

    const timestamp = new Date().getTime();
    const response = await fetch("myscript.py?cache_bust=${timestamp}");
    const pythonCode = await response.text();
    await pyodide.runPythonAsync(pythonCode);
    console.log("Python script loaded");
}

async function runPythonFunction() {
    const result = pyodide.runPython("optimize([{'t':1890.43,'x':0.15,'y':0.187,'weight':1,'method':1}, {'t':1893.48,'x':0.05,'y':0.235,'weight':1,'method':1}, {'t':1898.44,'x':-0.119,'y':0.276,'weight':1,'method':1}])");
    const parameters = result.toJs();
    console.log(parameters);
}

async function loadCalc() {
    var elt = document.getElementById('calculator');
    var calculator = Desmos.Calculator3D(elt);

    const response = await fetch('./state.json');
    const json = await response.json();
    calculator.setState(JSON.stringify(json));
    calculator.updateSettings({"expressions": false});
}

window.addEventListener("load", async () => {
    await initializePyodide();
    await loadPythonScript();
    await loadCalc();
});
