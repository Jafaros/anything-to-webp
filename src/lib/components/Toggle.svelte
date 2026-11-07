<script lang="ts">
  let {
    label,
    checked = $bindable(false),
    hint = "",
  }: { label: string; checked: boolean; hint?: string } = $props();
</script>

<label
  class="flex cursor-pointer items-start justify-between gap-4 py-1 text-xs font-medium"
>
  <span
    >{label}{#if hint}<span
        class="mt-1 block text-[11px] leading-relaxed font-normal text-muted"
        >{hint}</span
      >{/if}</span
  >
  <input class="mt-0.5 shrink-0" type="checkbox" bind:checked />
</label>
