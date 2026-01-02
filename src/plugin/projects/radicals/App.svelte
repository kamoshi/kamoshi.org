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

  const STORAGE_KEY = "kanji-graph-roots";

  let htmlCanvas = $state<HTMLCanvasElement | null>(null);
  let htmlContainer = $state<HTMLDivElement | null>(null);
  let data = $state<Record<string, string[]> | null>(null);
  let inputText = $state(""); // Clear default, we load from storage
  let selectedNode = $state<Node | null>(null); // Removed type annotation in generic to fix Svelte parsing if strict
  let apiData = $state<KanjiApiData | null>(null);
  let loadingApi = $state(false);
  let engine = $state<KanjiGraphEngine | null>(null);
  let isEngineReady = $state(false);

  // Track active roots for persistence
  let activeRoots = $state<Set<string>>(new Set());

  // --- 1. Load Persistence on Init ---
  function loadPersistedState() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (Array.isArray(parsed) && parsed.length > 0) {
          activeRoots = new Set(parsed);
          return;
        }
      }
    } catch (e) {
      console.warn("Failed to load graph state", e);
    }
    // Default fallback if storage is empty
    activeRoots = new Set(["亜"]);
  }

  // --- 2. Save Persistence Helper ---
  function saveState() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify([...activeRoots]));
    } catch (e) {
      console.error("Failed to save state", e);
    }
  }

  $effect(() => {
    fetch(url)
      .then((res) => res.json())
      .then((res) => {
        data = res;
        loadPersistedState(); // Load saved roots once data is fetched
      });
  });

  $effect(() => {
    if (!htmlCanvas || !data || !htmlContainer) return;

    const instance = new KanjiGraphEngine(htmlCanvas, data);

    instance.onNodeSelect = (node) => {
      selectedNode = node;
    };

    instance.onReady = () => {
      isEngineReady = true;

      // RESTORE: Expand all nodes found in localStorage
      if (activeRoots.size > 0) {
        activeRoots.forEach((root) => instance.expandNode(root));
      }
    };

    instance.init().catch((e) => console.error("Engine Init Failed", e));

    const resizeObserver = new ResizeObserver(() => instance.resize());
    resizeObserver.observe(htmlContainer);

    engine = instance;

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

    // Update Engine
    engine.promoteToRoot(inputText);

    // Update Persistence
    activeRoots.add(inputText);
    saveState();

    inputText = "";
  }

  // --- 3. UI Reactivity Fix ---
  function toggleNodeStatus(targetStatus: "visible" | "shadow") {
    if (!selectedNode || !engine) return;

    const id = selectedNode.id;

    const nodeSnapshot = { ...selectedNode };

    if (targetStatus === "visible") {
      engine.expandNode(id);
      activeRoots.add(id);
    } else {
      engine.hideNode(id);
      activeRoots.delete(id);
    }

    saveState();

    // 2. Force the UI to use our snapshot with the updated status
    selectedNode = {
      ...nodeSnapshot,
      status: targetStatus,
    };
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
            onclick={() => toggleNodeStatus("shadow")}
          >
            Remove
          </button>
        {:else}
          <button
            class="action-btn expand"
            onclick={() => toggleNodeStatus("visible")}
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
