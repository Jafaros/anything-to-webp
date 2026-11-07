<script lang="ts">
  import { RotateCcw, SlidersHorizontal, ChevronDown } from "@lucide/svelte";
  import type { EncoderSettings } from "$lib/converter/types";
  import { defaultSettings, profiles } from "$lib/converter/settings";
  import RangeField from "./RangeField.svelte";
  import NumberField from "./NumberField.svelte";
  import Toggle from "./Toggle.svelte";

  let {
    settings = $bindable(),
    disabled = false,
  }: { settings: EncoderSettings; disabled?: boolean } = $props();
  let tab = $state<"encoding" | "image" | "advanced">("encoding");
</script>

<aside
  class="rounded-2xl border border-line bg-white shadow-[0_3px_18px_-12px_#243a29]"
>
  <div class="flex items-center justify-between px-6 pt-6 pb-5">
    <h2 class="flex items-center gap-2.5 text-sm font-semibold">
      <SlidersHorizontal size={16} class="text-leaf" /> Export settings
    </h2>
    <button
      class="rounded-md p-1.5 text-muted hover:bg-paper hover:text-ink"
      title="Reset all settings"
      aria-label="Reset all settings"
      {disabled}
      onclick={() => (settings = structuredClone(defaultSettings))}
      ><RotateCcw size={14} /></button
    >
  </div>
  <div
    class="mx-6 flex gap-5 border-b border-line"
    role="tablist"
    aria-label="Export settings"
  >
    {#each ["encoding", "image", "advanced"] as item}
      <button
        id={`tab-${item}`}
        role="tab"
        aria-selected={tab === item}
        aria-controls="settings-content"
        class={`border-b-2 pb-3 text-xs font-medium capitalize ${tab === item ? "border-leaf text-leaf" : "border-transparent text-muted hover:text-ink"}`}
        onclick={() => (tab = item as typeof tab)}>{item}</button
      >
    {/each}
  </div>
  <div
    id="settings-content"
    role="tabpanel"
    tabindex="0"
    aria-labelledby={`tab-${tab}`}
  >
    <fieldset {disabled} class="space-y-4 p-5">
      {#if tab === "encoding"}
        <div>
          <p class="section-label mb-3">Start with a profile</p>
          <div class="grid grid-cols-2 gap-2">
            {#each Object.values(profiles) as profile}
              <button
                type="button"
                class="rounded-lg border border-line px-3 py-2.5 text-left text-xs font-medium hover:border-leaf hover:bg-green-50"
                onclick={() =>
                  (settings = {
                    ...structuredClone(defaultSettings),
                    ...profile.settings,
                  })}
                title={profile.description}>{profile.label}</button
              >
            {/each}
          </div>
        </div>
        <label class="block text-xs font-medium"
          >Compression mode
          <select class="field mt-2" bind:value={settings.mode}
            ><option value="lossy">Lossy · smaller files</option><option
              value="lossless">Lossless · preserve pixels</option
            ><option value="nearLossless">Near-lossless · subtle changes</option
            ></select
          >
        </label>
        <RangeField
          label={settings.mode === "lossy"
            ? "Image quality"
            : "Compression effort"}
          bind:value={settings.quality}
          hint={settings.mode === "lossy"
            ? "80 is a good balance of size and detail."
            : "Higher effort makes smaller files and takes longer."}
        />
        {#if settings.mode === "nearLossless"}<RangeField
            label="Near-lossless fidelity"
            bind:value={settings.nearLossless}
            hint="100 preserves pixels. Lower values allow more changes."
          />{/if}
        <RangeField
          label="Encoding method"
          bind:value={settings.method}
          max={6}
          hint="0 is fastest. 6 spends more time finding a smaller file."
        />
        <label class="block text-xs font-medium"
          >Content preset
          <select class="field mt-2" bind:value={settings.preset}
            ><option value="default">Default · general purpose</option><option
              value="photo">Photo</option
            ><option value="picture">Picture</option><option value="drawing"
              >Drawing</option
            ><option value="icon">Icon</option><option value="text">Text</option
            ></select
          >
        </label>
        <p
          class="rounded-lg bg-paper px-3 py-2.5 text-[11px] leading-relaxed text-muted"
        >
          Settings apply to every ready image in the queue. Profiles reset all
          settings, including resize and crop.
        </p>
      {:else if tab === "image"}
        <div>
          <p class="section-label mb-3">Resize</p>
          <div class="grid grid-cols-2 gap-3">
            <NumberField
              label="Width (px)"
              bind:value={settings.width}
            /><NumberField label="Height (px)" bind:value={settings.height} />
          </div>
          <p class="mt-2 text-[11px] leading-relaxed text-muted">
            Both 0 keeps the original size. Set one dimension to 0 to preserve
            the aspect ratio.
          </p>
        </div>
        <label class="flex items-center justify-between text-xs font-medium"
          >Enable crop<input
            type="checkbox"
            checked={settings.crop !== null}
            onchange={(event) =>
              (settings.crop = event.currentTarget.checked
                ? { x: 0, y: 0, width: 100, height: 100 }
                : null)}
          /></label
        >
        {#if settings.crop}<div class="grid grid-cols-2 gap-3">
            <NumberField
              label="Left (px)"
              bind:value={settings.crop.x}
            /><NumberField
              label="Top (px)"
              bind:value={settings.crop.y}
            /><NumberField
              label="Crop width"
              min={1}
              bind:value={settings.crop.width}
            /><NumberField
              label="Crop height"
              min={1}
              bind:value={settings.crop.height}
            />
          </div>
          <p class="text-[11px] text-muted">
            Crop is applied before resize and must fit every source.
          </p>{/if}
        <div class="border-t border-line pt-5">
          <RangeField
            label="Transparency quality"
            bind:value={settings.alphaQuality}
            hint="100 keeps the alpha channel lossless."
          />
        </div>
        <Toggle
          label="Preserve invisible RGB values"
          bind:checked={settings.exact}
          hint="Retain color data in fully transparent pixels."
        />
        <label class="block text-xs font-medium"
          >Keep metadata<select
            class="field mt-2"
            bind:value={settings.metadata}
            ><option value="none">None · recommended for privacy</option><option
              value="icc">Color profile (ICC)</option
            ><option value="exif">Camera information (EXIF)</option><option
              value="xmp">Editing information (XMP)</option
            ><option value="all">All metadata</option></select
          ><span
            class="mt-2 block text-[11px] leading-relaxed font-normal text-muted"
            >Metadata retention uses native PNG, JPEG, TIFF and WebP inputs.
            Animated PNG/WebP may be rejected in this mode. Other formats are
            normalized and lose metadata.</span
          ></label
        >
      {:else}
        <Toggle
          label="Multithreaded encoding"
          bind:checked={settings.multithreading}
          hint="Let cwebp use multiple CPU threads."
        />
        <fieldset
          disabled={settings.mode !== "lossy"}
          class="space-y-5 disabled:opacity-40"
        >
          <p class="section-label border-t border-line pt-5">Lossy tuning</p>
          <Toggle
            label="Sharper color conversion"
            bind:checked={settings.sharpYuv}
          />
          <Toggle
            label="Automatic filter strength"
            bind:checked={settings.autoFilter}
          />
          <Toggle
            label="Reduce memory use"
            bind:checked={settings.lowMemory}
            hint="Effective for method 3 and higher; slower encoding."
          />
          <RangeField
            label="Filter strength"
            bind:value={settings.filterStrength}
          />
          <RangeField
            label="Filter sharpness"
            bind:value={settings.sharpness}
            max={7}
            hint="0 is sharpest. 7 is least sharp."
          />
          <RangeField label="Spatial noise shaping" bind:value={settings.sns} />
          <details class="group border-t border-line pt-4">
            <summary
              class="flex cursor-pointer list-none items-center justify-between text-xs font-semibold"
              >Target output <ChevronDown
                size={14}
                class="group-open:rotate-180"
              /></summary
            >
            <div class="mt-4 space-y-4">
              <NumberField
                label="Target size (KB)"
                max={1000000}
                bind:value={settings.targetSizeKb}
                hint="0 disables. A target is an estimate, not a hard limit."
              /><NumberField
                label="Target PSNR (dB)"
                max={100}
                step={0.1}
                bind:value={settings.targetPsnr}
                hint="0 disables. Choose size or PSNR, never both."
              /><NumberField
                label="Target search passes"
                min={1}
                max={10}
                bind:value={settings.passes}
              />
            </div>
          </details>
        </fieldset>
        {#if settings.mode !== "lossy"}<p class="text-[11px] text-muted">
            Lossy tuning is available in lossy mode.
          </p>{/if}
      {/if}
    </fieldset>
  </div>
</aside>
