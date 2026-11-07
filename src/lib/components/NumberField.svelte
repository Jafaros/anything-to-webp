<script lang="ts">
  let {
    label,
    value = $bindable(0),
    min = 0,
    max = 16383,
    step = 1,
    hint = "",
  }: {
    label: string;
    value: number;
    min?: number;
    max?: number;
    step?: number;
    hint?: string;
  } = $props();
</script>

<label class="block text-xs font-medium">
  <span class="mb-2 block">{label}</span>
  <input
    class="field tabular-nums"
    type="number"
    {min}
    {max}
    {step}
    bind:value
    required
  />
  {#if hint}<span
      class="mt-1.5 block text-[11px] leading-relaxed font-normal text-muted"
      >{hint}</span
    >{/if}
</label>
