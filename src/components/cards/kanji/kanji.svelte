<script lang="ts">
  import { fetchKanji, type KKLCEntry } from './data.svelte.ts';

  let state = $state<Promise<KKLCEntry>>(new Promise(() => {}));

  $effect(() => void (state = fetchKanji()));
</script>


<div class="daily-kanji">
{#await state}
  <div class="spinner-wrap">
    <div class="spinner"><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div></div>
  </div>
{:then state}
  <div class="info">
    <div class="info-char">
      {state.char}
    </div>
    <div class="info-meta">
      <div class="info-key">
        {state.meanings.join(', ')}
      </div>
      <div class="info-on">
        {state.on.join(', ')}
      </div>
      <div class="info-kun">
        {state.kun.join(', ')}
      </div>
    </div>
  </div>
  <table>
    <tbody>
    {#each state.examples as [example, meaning]}
      <tr>
        <td>
          <ruby>
            {#each example as [kanji, furigana]}{kanji}<rt>{furigana || ''}</rt>{/each}
          </ruby>
        </td>
        <td>
          {meaning}
        </td>
      </tr>
    {/each}
    </tbody>
  </table>
{/await}
</div>
