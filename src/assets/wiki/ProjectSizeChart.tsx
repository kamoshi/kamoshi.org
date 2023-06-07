import { Maybe } from "purify-ts";
import { onMount } from "solid-js";
import Chart from "chart.js/auto";


const data = {
  labels: ["June"],
  datasets: [
    {
      label: "Markdown",
      data: [2455],
    },
    {
      label: "SCSS",
      data: [1138],
    },
    {
      label: "TypeScript",
      data: [1011],
    },
    {
      label: "Astro",
      data: [946]
    }
  ]
};

const options = {
  responsive: true,
  maintainAspectRatio: false,
  scales: {
    y: {
      beginAtZero: true,
    }
  }
};

export default function ProjectSizeChart() {

  onMount(() => Maybe
    .fromNullable(document.getElementById("project-size-chart") as HTMLCanvasElement)
    .chainNullable(e => e.getContext("2d"))
    .ifJust(ctx => new Chart(ctx, { type: "line", data, options }))
  )

  return (
    <div style="width: 100%; height: 16em">
      <canvas id="project-size-chart"></canvas>
    </div>
  )
}
