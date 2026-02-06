<script lang="ts">
  import { getKanji, type KKLCEntry } from "./storage";

  let state = $state<Promise<KKLCEntry>>(new Promise(() => {}));

  async function getKanjiSVG(char: string) {
    const codePoint = char.codePointAt(0);

    if (codePoint === undefined) {
      throw new Error(`Invalid character input: ${char}`);
    }

    const hexCode = codePoint.toString(16).toLowerCase().padStart(5, "0");
    const url = `/static/svg/kanji/${hexCode}.svg`;

    const data = await fetch(url);
    const text = await data.text();

    return text
      .substring(text.indexOf("<svg"))
      .replaceAll(/<g id="kvg:StrokeNumbers[\s\S]*?<\/g>/g, "");
  }

  $effect(() => void (state = getKanji()));
</script>

<h2 class="p-card__heading">Kanji of the day</h2>

<div class="daily-kanji">
  {#await state}
    <div class="spinner-wrap">
      <div class="spinner">
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
        <div></div>
      </div>
    </div>
  {:then state}
    <div class="info">
      <div class="info-box">
        <div class="info-id">
          #{state.id}
        </div>
        {#await getKanjiSVG(state.char)}
          ...
        {:then svg}
          {@html svg}
        {/await}
      </div>
      <div class="info-meta">
        <div class="info-key">
          {state.keys.join(", ")}
        </div>
        <div class="info-on">
          {state.onyomi.join(", ")}
        </div>
        <div class="info-kun">
          {state.kunyomi.join(", ")}
        </div>
      </div>
    </div>

    <section class="table">
      {#each state.examples as [meaning, example]}
        <div class="cell-ja">
          <ruby>
            {#each example as [expr, ruby]}{expr}<rt>{ruby || ""}</rt>{/each}
          </ruby>
        </div>
        <div class="cell-en">
          {meaning}
        </div>
      {/each}
    </section>
  {/await}
</div>
