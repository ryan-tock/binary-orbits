let pyodide;

async function initializePyodide() {
    pyodide = await loadPyodide();
    console.log("Pyodide loaded");
}

async function loadPythonScript() {
    const response = await fetch("myscript.py");
    const pythonCode = await response.text();
    await pyodide.runPythonAsync(pythonCode);
    console.log("Python script loaded");
}

async function runPythonFunction() {
    try {
        const result = pyodide.runPython("test()");
        document.getElementById("output").textContent = result;
    } catch (error) {
        console.error("Error executing Python function:", error);
        document.getElementById("output").textContent = "Error: " + error;
    }
}

window.addEventListener("load", async () => {
    await initializePyodide();
    await loadPythonScript();
});
