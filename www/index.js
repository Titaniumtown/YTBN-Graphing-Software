// If you only use `npm` you can simply
// import { Chart } from "wasm-demo" and remove `setup` call from `bootstrap.js`.
class Chart {}

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

let chart = null;

/** Main entry point */
export function main() {
    setupUI();
    setupCanvas();
}

/** This function is used in `bootstrap.js` to setup imports. */
export function setup(WasmChart) {
    Chart = WasmChart;
}

/** Add event listeners. */
function setupUI() {
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

		if(event.target == canvas) {
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
    status.innerText = `Rendering y=${math_function.value}...`;
    chart = null;
    const start = performance.now();

	chart = Chart.draw(canvas, math_function.value, Number(minX.value), Number(maxX.value), Number(minY.value), Number(maxY.value), Number(num_interval.value), Number(resolution.value));
    const end = performance.now();

    area_msg.innerText = `Estimated Area: ${chart.get_area()}`;

    status.innerText = `Rendered ${math_function.innerText} in ${Math.ceil(end - start)}ms`;
}
