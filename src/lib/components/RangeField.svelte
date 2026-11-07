<script lang="ts">
  let {
    label,
    value = $bindable(0),
    min = 0,
    max = 100,
    hint = "",
  }: {
    label: string;
    value: number;
    min?: number;
    max?: number;
    hint?: string;
  } = $props();
</script>

<div class="block text-xs font-medium">
  <span class="mb-3 flex items-center justify-between gap-3"
    ><span>{label}</span><input
      class="w-16 rounded-md border border-line bg-paper px-2 py-1 text-right text-xs tabular-nums"
      type="number"
      aria-label={`${label} value`}
      {min}
      {max}
      bind:value
      required
    /></span
  >
  <input type="range" aria-label={label} {min} {max} step="1" bind:value />
  {#if hint}<span
      class="mt-1.5 block text-[11px] leading-relaxed font-normal text-muted"
      >{hint}</span
    >{/if}
</div>
