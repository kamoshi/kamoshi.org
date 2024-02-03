<script lang="ts">
  import { getKanji, type KKLCEntry } from './data.svelte.ts';

  let state = $state<Promise<KKLCEntry>>(new Promise(() => {}));

  $effect(() => void (state = getKanji()));
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
        {state.keys.join(', ')}
      </div>
      <div class="info-on">
        {state.onyomi.join(', ')}
      </div>
      <div class="info-kun">
        {state.kunyomi.join(', ')}
      </div>
    </div>
  </div>
  <table class="examples">
    <tbody>
    {#each state.examples as [meaning, example]}
      <tr>
        <td class="examples-ja">
          <ruby>
            {#each example as [expr, ruby]}{expr}<rt>{ruby||''}</rt>{/each}
          </ruby>
        </td>
        <td class="examples-en">
          {meaning}
        </td>
      </tr>
    {/each}
    </tbody>
  </table>
{/await}
</div>
