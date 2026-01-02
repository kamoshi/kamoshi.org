<script lang="ts">
  import { KanjiGraphEngine, type Node } from "./engine";

  // --- API DATA TYPES ---
  interface KanjiApiData {
    kanji: string;
    grade: number | null;
    meanings: string[];
    kun_readings: string[];
    on_readings: string[];
    jlpt: number | null;
  }

  let htmlCanvas = $state<HTMLCanvasElement | null>(null);
  let htmlContainer = $state<HTMLDivElement | null>(null);

  let engine = $derived.by(
    () => htmlCanvas && new KanjiGraphEngine(htmlCanvas),
  );

  let inputText = $state("亜");
  let selectedNode: Node | null = $state(null);
  let apiData: KanjiApiData | null = $state(null);
  let loadingApi = $state(false);

  $effect(() => {
    if (engine) {
      engine.onNodeSelect = (node) => {
        selectedNode = node; // Svelte reacts to this
      };

      // Initial load
      engine.expandNode("亜");

      // Handle window resize
      const resizeObserver = new ResizeObserver(() => engine.resize());
      resizeObserver.observe(htmlContainer);

      // return () => {
      //   resizeObserver.disconnect();
      //   engine.destroy();
      // };
    }
  });

  async function fetchApiData(kanji: string) {
    loadingApi = true;
    apiData = null;
    try {
      const res = await fetch(
        `https://kanjiapi.dev/v1/kanji/${encodeURIComponent(kanji)}`,
      );
      if (res.ok) {
        apiData = await res.json();
      }
    } catch (e) {
      console.error(e);
    } finally {
      loadingApi = false;
    }
  }

  $effect(() => {
    if (selectedNode) {
      fetchApiData(selectedNode.id);
    } else {
      apiData = null;
    }
  });
</script>

<div class="app-container" bind:this={htmlContainer}>
  <canvas bind:this={htmlCanvas} class="graph-canvas"></canvas>

  <div class="search-panel">
    <form onsubmit={() => {}} class="search-form">
      <!-- <Search size={18} class="text-slate-400" /> -->
      <input
        type="text"
        bind:value={inputText}
        placeholder="Search Kanji (e.g. 亜)"
        maxlength="1"
      />
      <button type="submit">Go</button>
    </form>
  </div>

  {#if selectedNode}
    <div class="info-panel">
      <div class="info-header">
        <div class="info-title">
          <span class="kanji-large">{selectedNode.id}</span>
          <span
            class="status-badge"
            class:shadow={selectedNode.status === "shadow"}
          >
            {selectedNode.status}
          </span>
        </div>
        <button class="close-btn" onclick={() => (selectedNode = null)}>
          <!-- <X size={18} /> -->
        </button>
      </div>

      <div class="info-content">
        <div class="radicals">
          <span class="label">Components:</span>
          <div class="tags">
            {#each selectedNode.radicals as r}
              <span class="tag">{r}</span>
            {/each}
          </div>
        </div>

        {#if selectedNode.status === "visible"}
          <button
            class="action-btn delete"
            onclick={() => {
              // const res = removeNode(
              //   selectedNode!.id,
              //   graphData.nodes,
              //   graphData.links,
              // );
              // graphData = res;
              // selectedNode = null;
              // restartSimulation();
            }}
          >
            <!-- <Trash2 size={16} /> Prune Branch -->
          </button>
        {:else}
          <button
            class="action-btn expand"
            onclick={() => {
              // const res = expandNode(
              //   selectedNode!.id,
              //   graphData.nodes,
              //   graphData.links,
              // );
              // graphData = res;
              // restartSimulation();
            }}
          >
            <!-- <BookOpen size={16} /> Expand Node -->
          </button>
        {/if}

        <hr />

        {#if loadingApi}
          <div class="loader">
            <!-- <Loader2 class="animate-spin" /> Loading details... -->
          </div>
        {:else if apiData}
          <div class="api-details">
            <p>
              <strong>Meanings:</strong>
              {apiData.meanings.slice(0, 3).join(", ")}
            </p>
            <p><strong>Onyomi:</strong> {apiData.on_readings.join("、")}</p>
            <p><strong>Kunyomi:</strong> {apiData.kun_readings.join("、")}</p>
            <div class="stats">
              <span>JLPT: N{apiData.jlpt}</span>
              <span>Grade: {apiData.grade}</span>
            </div>
          </div>
        {:else}
          <p class="text-sm text-slate-500">No API data available.</p>
        {/if}
      </div>
    </div>
  {/if}

  <div class="legend">
    <div class="legend-item"><span class="dot root"></span> Root</div>
    <div class="legend-item"><span class="dot visible"></span> Loaded</div>
    <div class="legend-item"><span class="dot shadow"></span> Unloaded</div>
    <div class="legend-item"><span class="line child"></span> Component</div>
  </div>
</div>
