<script lang="ts">
  import { KanjiGraphEngine, type Node } from "./engine";

  let { url } = $props() as { url: string };

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
  let data = $state<Record<string, string[]> | null>(null);
  let inputText = $state("亜");
  let selectedNode: Node | null = $state(null);
  let apiData: KanjiApiData | null = $state(null);
  let loadingApi = $state(false);
  let engine = $state<KanjiGraphEngine | null>(null);
  let isEngineReady = $state(false);

  $effect(() => {
    fetch(url)
      .then((res) => res.json())
      .then((res) => {
        data = res;
      });
  });

  // --- INITIALIZATION EFFECT ---
  $effect(() => {
    if (!htmlCanvas || !data || !htmlContainer) return;

    // 1. Instantiate
    const instance = new KanjiGraphEngine(htmlCanvas, data);

    // 2. Configure Callbacks
    instance.onNodeSelect = (node) => {
      selectedNode = node;
    };

    // 3. Define what happens when ready
    instance.onReady = () => {
      isEngineReady = true;
      // Only expand once the engine is fully ready and viewport exists
      instance.expandNode("亜");
    };

    // 4. Start Async Init
    instance.init().catch((e) => console.error("Engine Init Failed", e));

    // 5. Setup Resize Observer
    const resizeObserver = new ResizeObserver(() => instance.resize());
    resizeObserver.observe(htmlContainer);

    // 6. Save reference
    engine = instance;

    // 7. Cleanup
    return () => {
      resizeObserver.disconnect();
      instance.destroy();
      engine = null;
      isEngineReady = false;
    };
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

  function handleSearch(e: Event) {
    e.preventDefault();
    if (!engine || !isEngineReady || !inputText) return;

    if (data && !data[inputText]) {
      alert("Kanji not found in local dataset");
      return;
    }

    engine.promoteToRoot(inputText);
    inputText = "";
  }
</script>

<div class="app-container" bind:this={htmlContainer}>
  <canvas bind:this={htmlCanvas} class="graph-canvas"></canvas>

  <div class="search-panel">
    <form onsubmit={handleSearch} class="search-form">
      <input
        type="text"
        bind:value={inputText}
        placeholder="Search Kanji (e.g. 亜)"
        maxlength="1"
      />
      <button type="submit" disabled={!isEngineReady}>Go</button>
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
        <button class="close-btn" onclick={() => (selectedNode = null)}
          >×</button
        >
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
            onclick={() => engine?.hideNode(selectedNode!.id)}
          >
            Remove
          </button>
        {:else}
          <button
            class="action-btn expand"
            onclick={() => engine?.expandNode(selectedNode!.id)}
          >
            Expand
          </button>
        {/if}

        <hr />

        {#if loadingApi}
          <div class="loader">Loading...</div>
        {:else if apiData}
          <div class="api-details">
            <p>
              <strong>Meanings:</strong>
              {apiData.meanings.slice(0, 3).join(", ")}
            </p>
            <p><strong>On:</strong> {apiData.on_readings.join("、")}</p>
            <p><strong>Kun:</strong> {apiData.kun_readings.join("、")}</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <div class="legend">
    <div class="legend-item"><span class="dot root"></span> Root</div>
    <div class="legend-item"><span class="dot visible"></span> Loaded</div>
    <div class="legend-item"><span class="dot shadow"></span> Unloaded</div>
  </div>
</div>
