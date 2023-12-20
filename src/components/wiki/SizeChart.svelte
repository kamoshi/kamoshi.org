<script lang="ts">
import Chart from "chart.js/auto";
import { Maybe } from "purify-ts";


const data = {
  labels: ["April", "May", "June"],
  datasets: [
    {
      label: "Markdown",
      data: [2173, 2430, 2455],
    },
    {
      label: "SCSS",
      data: [910, 1116, 1138],
    },
    {
      label: "TypeScript",
      data: [242, 517, 1011],
    },
    {
      label: "Astro",
      data: [535, 845, 946]
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

$effect(() => {
  Maybe
    .fromNullable(document.getElementById("project-size-chart") as HTMLCanvasElement)
    .chainNullable(e => e.getContext("2d"))
    .ifJust(ctx => new Chart(ctx, { type: "line", data, options }))
})
</script>


<div style="width: 100%; height: 16em">
  <canvas id="project-size-chart"></canvas>
</div>
