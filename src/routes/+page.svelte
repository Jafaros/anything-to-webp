<script lang="ts">
  import { onMount } from "svelte";
  import { isTauri } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import {
    ArrowUpRight,
    ArrowRight,
    Plus,
    X,
    Image,
    Layers,
    FolderOpen,
    Check,
    CircleAlert,
    LoaderCircle,
    ShieldCheck,
    Sparkles,
    Square,
    ChevronRight,
    RefreshCw,
  } from "@lucide/svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import { Workspace } from "$lib/converter/workspace.svelte";
  import { formatBytes, saveSettings } from "$lib/converter/settings";

  const workspace = new Workspace();
  let dragging = $state(false);
  let helpOpen = $state(false);
  let initialized = $state(false);
  const finishedCount = $derived(
    workspace.files.filter(
      (file) => file.status === "done" || file.status === "error",
    ).length,
  );
  const hasErrors = $derived(
    workspace.files.some((file) => file.status === "error"),
  );

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void workspace.initialize().then(() => {
      initialized = true;
    });
    if (isTauri()) {
      void getCurrentWebview()
        .onDragDropEvent(({ payload }) => {
          dragging = payload.type === "over" || payload.type === "enter";
          if (payload.type === "drop") void workspace.addPaths(payload.paths);
        })
        .then((cleanup) => {
          if (disposed) cleanup();
          else unlisten = cleanup;
        })
        .catch(
          (error) => (workspace.error = `Drag and drop unavailable: ${error}`),
        );
    }
    return () => {
      disposed = true;
      unlisten?.();
    };
  });

  $effect(() => {
    if (initialized) saveSettings(workspace.settings);
  });

  function handleKeydown(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "o") {
      event.preventDefault();
      if (!workspace.running && !workspace.adding) void workspace.chooseFiles();
    }
    if (event.key === "Escape") helpOpen = false;
  }
</script>

<svelte:head
  ><title>Anything to WebP</title><meta
    name="description"
    content="A local image conversion workspace powered by Google's cwebp encoder."
  /></svelte:head
>
<svelte:window onkeydown={handleKeydown} />

<div class="min-h-screen">
  <header
    class="flex h-16 items-center justify-between border-b border-line bg-white/80 px-8 lg:px-12"
  >
    <div class="flex items-center gap-3">
      <div
        class="flex size-10 items-center justify-center rounded-xl bg-leaf text-white"
      >
        <Layers size={21} strokeWidth={1.7} />
      </div>
      <div>
        <p class="text-sm font-semibold tracking-tight">
          Anything<span class="font-normal text-muted">&nbsp;to&nbsp;</span>WebP
        </p>
        <p class="mt-0.5 text-[10px] tracking-[0.14em] text-muted uppercase">
          Less weight. Same wonder.
        </p>
      </div>
    </div>
    <div class="flex items-center gap-5">
      <span
        class="hidden items-center gap-2 rounded-full border border-line bg-paper px-3 py-1.5 text-[11px] text-muted sm:flex"
        ><span class="size-1.5 rounded-full bg-leaf"></span>All local. All
        yours.</span
      ><button
        class="text-xs font-medium text-muted hover:text-leaf"
        onclick={() => (helpOpen = !helpOpen)}
        >Format guide <ArrowUpRight size={13} class="ml-1 inline" /></button
      >
    </div>
  </header>

  <main class="mx-auto max-w-[1500px] px-8 py-6 lg:px-12">
    <div class="mb-6 flex items-end justify-between gap-4">
      <div>
        <p class="section-label mb-3 flex items-center gap-2">
          <span class="h-px w-6 bg-leaf"></span> Your image workspace
        </p>
        <h1
          class="text-[32px] leading-tight font-semibold tracking-[-0.045em] lg:text-[38px]"
        >
          Great images. <span class="text-leaf">Lighter footprint.</span>
        </h1>
        <p class="mt-3 text-sm text-muted">
          Turn your images into WebP, with every detail under your control.
        </p>
      </div>
      <span class="mb-1 hidden text-[11px] text-muted xl:block"
        >POWERED BY GOOGLE’S WEBP ENCODER</span
      >
    </div>

    {#if workspace.error}<div
        role="alert"
        class="mb-5 flex items-start justify-between gap-3 rounded-xl border border-red-200 bg-red-50 p-4 text-sm text-red-800"
      >
        <span class="flex items-start gap-2"
          ><CircleAlert size={17} class="mt-0.5 shrink-0" /><span
            class="break-words">{workspace.error}</span
          ></span
        ><button
          aria-label="Dismiss error"
          onclick={() => (workspace.error = "")}><X size={16} /></button
        >
      </div>{/if}

    {#if helpOpen}<section
        class="mb-6 rounded-xl border border-line bg-white p-5 text-xs leading-relaxed"
      >
        <div class="mb-3 flex items-center justify-between">
          <h2 class="text-sm font-semibold">
            A format guide, without the guesswork.
          </h2>
          <button
            aria-label="Close format guide"
            onclick={() => (helpOpen = false)}><X size={16} /></button
          >
        </div>
        <p>
          <strong>Supported:</strong> PNG, JPEG / JFIF, TIFF, WebP, GIF, BMP, ICO,
          SVG, PNM / PBM / PGM / PPM / PAM, TGA, DDS, HDR, EXR, farbfeld and QOI.
        </p>
        <p class="mt-2 text-muted">
          Animated images become a still first frame; multi-page images export
          one image. SVG uses its intrinsic size with no external resources. HDR
          becomes 8-bit without tone mapping. HEIC, AVIF, camera RAW, PSD and
          PDF are not supported. Maximum input: 512 MB, 100 megapixels and
          16,383 pixels per side.
        </p>
        <p class="mt-2 text-muted">
          Native JPEG/TIFF encoding retains source pixel orientation; camera
          rotation remains EXIF metadata only when retained. Images normalized
          by Rust have orientation applied and metadata removed.
        </p>
      </section>{/if}

    <div
      class="grid items-start gap-6 lg:grid-cols-[minmax(0,1fr)_340px] xl:grid-cols-[minmax(0,1fr)_360px]"
    >
      <section class="min-w-0 space-y-4" aria-label="Conversion workspace">
        <button
          class={`group flex w-full flex-col items-center justify-center rounded-2xl border-2 border-dashed px-6 py-6 transition-all ${dragging ? "border-leaf bg-green-100" : "border-[#cbd6c8] bg-[#f0f4ec] hover:border-leaf hover:bg-[#eaf0e5]"}`}
          disabled={workspace.running || workspace.adding}
          onclick={() => workspace.chooseFiles()}
        >
          <span
            class="relative mb-3 flex size-12 items-center justify-center rounded-xl border border-[#d7e1d0] bg-white text-leaf shadow-[0_4px_12px_-7px_#39734d]"
            ><Image size={28} strokeWidth={1.4} /><span
              class="absolute -right-2 -bottom-1 flex size-6 items-center justify-center rounded-lg bg-leaf text-white"
              ><Plus size={14} /></span
            ></span
          >
          <span class="text-base font-semibold tracking-tight"
            >{dragging
              ? "A lighter image starts here"
              : "Drop your images here"}</span
          >
          <span class="mt-2 text-xs text-muted"
            >or <span
              class="font-semibold text-leaf underline decoration-leaf/30 underline-offset-4"
              >browse files</span
            > to get started</span
          >
          <span class="mt-4 text-[10px] tracking-wide text-muted"
            >PNG, JPG, TIFF, SVG, GIF, BMP + more <span class="mx-2">·</span> Ctrl
            + O</span
          >
        </button>

        <div
          class="overflow-hidden rounded-2xl border border-line bg-white shadow-[0_3px_18px_-12px_#243a29]"
        >
          <div
            class="flex items-center justify-between border-b border-line px-5 py-3"
          >
            <div class="flex items-center gap-2.5">
              <h2 class="text-sm font-semibold">Conversion queue</h2>
              <span
                class="rounded-md bg-paper px-2 py-0.5 text-[11px] font-medium tabular-nums text-muted"
                >{workspace.files.length}</span
              >
            </div>
            <div class="flex items-center gap-3">
              <button
                class="text-xs text-muted hover:text-ink"
                disabled={workspace.running || !workspace.files.length}
                onclick={() => {
                  workspace.files = [];
                  workspace.notice = "";
                }}>Clear</button
              ><button
                class="flex items-center gap-1 text-xs font-semibold text-leaf"
                disabled={workspace.running || workspace.adding}
                onclick={() => workspace.chooseFiles()}
                ><Plus size={14} /> Add images</button
              >
            </div>
          </div>
          {#if workspace.files.length === 0}
            <div
              class="flex min-h-32 flex-col items-center justify-center px-6 py-5"
            >
              <div
                class="mb-3 flex size-10 items-center justify-center rounded-full bg-paper text-[#aab4a7]"
              >
                <Layers size={20} strokeWidth={1.4} />
              </div>
              <p class="text-xs font-medium text-muted">
                Good things come in smaller files.
              </p>
              <p class="mt-1.5 text-[11px] text-[#9ba398]">
                Your images will appear here, ready for a fresh start.
              </p>
            </div>
          {:else}
            <ul class="max-h-[360px] divide-y divide-line overflow-auto">
              {#each workspace.files as file (file.path)}
                <li class="flex items-start gap-3 px-5 py-4">
                  <span
                    class={`mt-0.5 flex h-11 w-11 shrink-0 items-center justify-center rounded-lg text-[9px] font-bold uppercase ${file.status === "done" ? "bg-green-50 text-leaf" : "bg-paper text-muted"}`}
                    >{file.status === "done"
                      ? "WEBP"
                      : file.extension.slice(0, 5)}</span
                  >
                  <div class="min-w-0 flex-1">
                    <p class="truncate text-xs font-medium" title={file.path}>
                      {file.name}
                    </p>
                    <div
                      class="mt-1.5 flex flex-wrap items-center gap-2 text-[10px] text-muted"
                    >
                      <span>{formatBytes(file.bytes)}</span
                      >{#if file.result}<ArrowRight size={11} /><span
                          class="font-medium text-leaf"
                          >{formatBytes(file.result.outputBytes)}</span
                        ><span>· {file.result.elapsedMs} ms</span>{/if}<span
                        class={`ml-auto flex items-center gap-1 ${file.status === "done" ? "text-leaf" : file.status === "error" ? "text-red-600" : ""}`}
                        >{#if file.status === "converting"}<LoaderCircle
                            size={11}
                            class="animate-spin"
                          /> Converting{:else if file.status === "done"}<Check
                            size={11}
                          /> Complete{:else if file.status === "error"}<CircleAlert
                            size={11}
                          /> Failed{:else}<span
                            class="size-1 rounded-full bg-[#a6b39e]"
                          ></span> Ready{/if}</span
                      >
                    </div>
                    {#if file.error}<p
                        class="mt-2 break-words text-[11px] leading-relaxed text-red-600"
                      >
                        {file.error}
                      </p>{:else if file.result?.warning || file.warning}<p
                        class="mt-2 text-[10px] leading-relaxed text-muted"
                      >
                        {file.result?.warning || file.warning}
                      </p>{/if}{#if file.result}<button
                        class="mt-2 flex max-w-full items-center gap-1 text-[10px] font-medium text-leaf hover:underline"
                        title={file.result.outputPath}
                        onclick={() =>
                          file.result &&
                          workspace.reveal(file.result.outputPath)}
                        ><FolderOpen size={11} /> Show in folder</button
                      >{/if}
                  </div>
                  <button
                    class="mt-0.5 rounded p-1 text-muted hover:bg-paper"
                    aria-label={`Remove ${file.name}`}
                    disabled={workspace.running}
                    onclick={() =>
                      (workspace.files = workspace.files.filter(
                        (item) => item.path !== file.path,
                      ))}><X size={13} /></button
                  >
                </li>
              {/each}
            </ul>
          {/if}
          <div
            class="flex items-center justify-between border-t border-line bg-paper/50 px-5 py-3 text-[10px] text-muted"
          >
            <span
              >{workspace.files.length
                ? `${formatBytes(workspace.totalBytes)} total input`
                : "No images added yet"}</span
            >{#if hasErrors}<button
                class="flex items-center gap-1 font-medium text-leaf"
                disabled={workspace.running}
                onclick={() => workspace.retryErrors()}
                ><RefreshCw size={11} /> Retry failed</button
              >{:else}<span>Original files stay safe</span>{/if}
          </div>
        </div>

        <div class="rounded-2xl border border-line bg-white p-4">
          <div class="mb-3 flex items-center gap-2">
            <FolderOpen size={16} class="text-leaf" />
            <h2 class="text-sm font-semibold">Save your WebPs</h2>
          </div>
          <div class="flex items-center gap-3">
            <div
              class="min-w-0 flex-1 rounded-lg border border-line bg-paper px-3 py-2.5 text-xs text-muted"
              title={workspace.outputDirectory ??
                "Same folder as each original image"}
            >
              <p class="truncate">
                {workspace.outputDirectory ?? "Same folder as the original"}
              </p>
            </div>
            <button
              class="secondary-button"
              disabled={workspace.running}
              onclick={() => workspace.chooseOutput()}>Browse</button
            >{#if workspace.outputDirectory}<button
                class="p-1 text-muted"
                title="Use original folder"
                aria-label="Use original folder"
                disabled={workspace.running}
                onclick={() => (workspace.outputDirectory = null)}
                ><X size={14} /></button
              >{/if}
          </div>
          <p class="mt-2.5 text-[10px] text-muted">
            Existing files are kept. Duplicate names get a number, like image
            (1).webp.
          </p>
        </div>

        {#if workspace.running}<div class="space-y-2">
            <div class="flex justify-between text-xs text-muted">
              <span
                >{workspace.stopRequested
                  ? "Finishing current image…"
                  : "Making things lighter…"}</span
              ><span>{finishedCount} / {workspace.files.length}</span>
            </div>
            <progress
              class="h-1.5 w-full accent-leaf"
              value={finishedCount}
              max={workspace.files.length}
              aria-label="Batch conversion progress"
            ></progress>
          </div>{/if}

        <div class="flex items-center justify-between gap-3">
          <span class="flex items-center gap-1.5 text-[10px] text-muted"
            ><ShieldCheck size={13} /> Your images never leave this device</span
          >{#if workspace.running}<button
              class="secondary-button"
              disabled={workspace.stopRequested}
              onclick={() => (workspace.stopRequested = true)}
              ><Square size={12} /> Stop after current image</button
            >{:else}<button
              class="flex items-center gap-3 rounded-xl bg-leaf px-6 py-3.5 text-sm font-semibold text-white shadow-[0_4px_12px_-5px_#39734d] hover:bg-[#2d603e] disabled:shadow-none"
              disabled={!workspace.ready.length || workspace.adding}
              onclick={() => workspace.convert()}
              ><Sparkles size={16} /> Convert {workspace.ready.length || ""}
              {workspace.ready.length === 1 ? "image" : "images"}<ArrowRight
                size={16}
              /></button
            >{/if}
        </div>
        <div role="status" aria-live="polite" class="min-h-6 text-xs text-leaf">
          {workspace.notice}{#if workspace.completed.length}<span
              class="ml-2 text-muted"
              >{workspace.savedBytes >= 0
                ? `${formatBytes(workspace.savedBytes)} saved`
                : `${formatBytes(-workspace.savedBytes)} larger than input`} across
              {workspace.completed.length} image{workspace.completed.length ===
              1
                ? ""
                : "s"}.</span
            >{/if}
        </div>
      </section>

      <form onsubmit={(event) => event.preventDefault()}>
        <SettingsPanel
          bind:settings={workspace.settings}
          disabled={workspace.running}
        />
        <p
          class="mt-3 flex items-center justify-center gap-1.5 text-[10px] text-muted"
        >
          <Check size={11} /> Settings remembered on this device
        </p>
      </form>
    </div>
    <footer
      class="mt-5 flex items-center justify-between border-t border-line pt-4 text-[10px] text-muted"
    >
      <span class="flex items-center gap-2"
        ><span class="size-1.5 rounded-full bg-leaf"
        ></span>{workspace.encoderVersion}<span class="mx-1 text-line">/</span> Rust
        + Tauri</span
      ><span class="flex items-center gap-1"
        >Thoughtfully compressed <ChevronRight size={11} /></span
      >
    </footer>
  </main>
</div>
