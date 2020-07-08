import * as earley from "earley";
import * as svgPanZoom from "svg-pan-zoom";
import * as ace from "ace-builds";
import Split from 'split.js'

const Viz = require('viz.js');

const resizeEvent = new Event("paneresize");
Split(["#editor", "#graph"], {
  sizes: [25, 75],
  onDragEnd: function () {
    const svgOutput = document.getElementById("svg_output");
    if (svgOutput != null) {
      svgOutput.dispatchEvent(resizeEvent);
    }
  }
});

const editor = ace.edit("editor");
editor.getSession().setMode("ace/mode/dot");

const parser = new DOMParser();

const error = document.querySelector("#error");
const kind = document.querySelector("#kind select");
const input = document.querySelector("#input input");
const start = document.querySelector("#start input");
const ntrees = document.querySelector("#ntrees input");
const format = document.querySelector("#format select");

function updateGraph() {
  const output = document.querySelector("#output");
  output.classList.add("working");
  output.classList.remove("error");
  const rules = editor.getSession().getDocument().getValue();
  const n = parseInt(ntrees.value);
  let result;
  try {
    result = earley.parse(rules, input.value, start.value, kind.value, n);
  } catch (msg) {
    output.classList.add("error");
    while (error.firstChild) {
      error.removeChild(error.firstChild);
    }
    error.appendChild(document.createTextNode(msg));
  }
  output.classList.remove("working");

  let svg = output.querySelector("svg");
  if (svg) output.removeChild(svg);
  let text = output.querySelector("#text");
  if (text) output.removeChild(text);
  let img = output.querySelector("img");
  if (img) output.removeChild(img);

  if (!result) return;
  if (kind.value === "chart") { // render text
    const text = document.createElement("div");
    text.id = "text";
    text.appendChild(document.createTextNode(result));
    output.appendChild(text);
  } else if (format.value === "svg") { // render svg
    const svg = parser.parseFromString(Viz(result), "image/svg+xml").documentElement;
    svg.id = "svg_output";
    output.appendChild(svg);
    const panZoom = svgPanZoom(svg, {
      zoomEnabled: true,
      controlIconsEnabled: true,
      fit: true,
      center: true,
      minZoom: 0.1
    });
    svg.addEventListener("paneresize", _ => panZoom.resize(), false);
    window.addEventListener("resize", _ => panZoom.resize());
  } else { // render png
    output.appendChild(Viz.svgXmlToPngImageElement(Viz(result)));
  }
}

editor.on("change", () => updateGraph());
kind.addEventListener("change", () => updateGraph());
input.addEventListener("input", () => updateGraph());
start.addEventListener("input", () => updateGraph());
ntrees.addEventListener("input", () => updateGraph());
format.addEventListener("change", () => updateGraph());

updateGraph();