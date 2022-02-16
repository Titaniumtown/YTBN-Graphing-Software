class ChartManager {}

const canvas = document.getElementById("canvas");
const coord = document.getElementById("coord");
const math_function = document.getElementById("math_function");
const status = document.getElementById("status");

const minX = document.getElementById("minX");
const maxX = document.getElementById("maxX");
const minY = document.getElementById("minY");
const maxY = document.getElementById("maxY");
const num_interval = document.getElementById("num_interval");
const area_msg = document.getElementById("area-msg");
const resolution = document.getElementById("resolution");

let darkMode = false;

let chart = null;
let chart_manager = null;

/** Main entry point */
export function main() {
    setupUI();
    setupCanvas();
}

/** This function is used in `bootstrap.js` to setup imports. */
export function setup(WasmChart) {
    ChartManager = WasmChart;
    ChartManager.init_panic_hook(); // Allows `panic!()` to log in the browser's console
}

/** Add event listeners. */
function setupUI() {
    
    // Handles browser color preferences
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
        darkMode = true;
    }

    // Watches for changes in color preferences
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', event => {
        darkMode = event.matches;
        updatePlot();
    });

    status.innerText = "WebAssembly loaded!";
    math_function.addEventListener("change", updatePlot);
    minX.addEventListener("input", updatePlot);
    maxX.addEventListener("input", updatePlot);
    minY.addEventListener("input", updatePlot);
    maxY.addEventListener("input", updatePlot);
    num_interval.addEventListener("input", updatePlot);
    resolution.addEventListener("input", updatePlot);


    window.addEventListener("resize", setupCanvas);
    window.addEventListener("mousemove", onMouseMove);
}

function setupCanvas() {
    const aspectRatio = canvas.width / canvas.height;
    const size = canvas.parentNode.offsetWidth * 0.8;
    canvas.style.width = size + "px";
    canvas.style.height = size / aspectRatio + "px";
    canvas.width = size;
    canvas.height = size / aspectRatio;
    updatePlot();
}

function onMouseMove(event) {
    if (chart) {
		var text = "Mouse is outside Chart.";

		if (event.target == canvas) {
			let actualRect = canvas.getBoundingClientRect();
			let logicX = event.offsetX * canvas.width / actualRect.width;
			let logicY = event.offsetY * canvas.height / actualRect.height;
			const point = chart.coord(logicX, logicY);
			text = (point) 
				? `(${point.x.toFixed(3)}, ${point.y.toFixed(3)})`
				: text;
		}
        coord.innerText = text;
    }
}

function updatePlot() {
    status.style.color = "grey";
    status.innerText = `Rendering y=${math_function.value}...`;
    
    if (chart_manager == null) {
        chart_manager = ChartManager.new(math_function.value, Number(minX.value), Number(maxX.value), Number(minY.value), Number(maxY.value), Number(num_interval.value), Number(resolution.value));
    }

    const test_result = ChartManager.test_func(math_function.value);
    if (test_result != "") {
        const error_recommendation = ChartManager.error_recommend(test_result);
        status.style.color = "red";
        if (error_recommendation == "") {
            status.innerText = test_result;
        } else {
            status.innerText = `${test_result}\nTip: ${error_recommendation}`
        }
        return;
    }

    try {
        const start = performance.now();
        chart = chart_manager.update(canvas, math_function.value, Number(minX.value), Number(maxX.value), Number(minY.value), Number(maxY.value), Number(num_interval.value), Number(resolution.value), false); // TODO: improve darkmode support
        const end = performance.now();

        area_msg.innerText = `Estimated Area: ${chart.get_area()}`;
    
        status.innerText = `Rendered ${math_function.innerText} in ${Math.ceil(end - start)}ms`;
    } catch(err) {
        status.style.color = "red";
        status.innerText = `Error! check console logs for more detail`;
    }
}
