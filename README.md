# Anything to WebP

A Windows x64 desktop image converter built with Tauri 2, Rust, Svelte 5,
TypeScript and Tailwind CSS 4. Conversion stays on your computer. The official
Google cwebp 1.6.0 executable is bundled; no separate encoder installation is needed.

## Run and build

Install Node.js 22.12+ (24 LTS recommended), Rust, Microsoft C++ Build Tools and
WebView2 using the [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm ci
npm run tauri dev
```

`npm run dev` previews the interface in a browser; local conversion requires the
desktop app. To build a release installer, use `npm run tauri build -- --bundles nsis`.
For an executable and its resource directory without an installer, use
`npm run tauri build -- --no-bundle`. Keep the `binaries` directory next to the
executable when distributing an unbundled build. Build artifacts are under
`src-tauri/target/release`.

## Use

1. Drop images onto the window, browse files, or press Ctrl+O.
2. Choose a profile or customize Encoding, Image and Advanced settings.
3. Use each original's folder or choose a shared destination.
4. Convert the batch and reveal results in Explorer.

Defaults: lossy quality 80, method 4, alpha quality 100, sharp YUV conversion,
multithreading, original dimensions and no metadata. Settings persist locally.
Profiles reset all parameters, including crop and resize. Crop is applied before
resize; setting one resize dimension to zero preserves the aspect ratio.
Lossless quality controls compression effort rather than visual fidelity.
Target size and PSNR are approximate, mutually exclusive lossy targets.

Existing files and originals are never replaced. Name collisions add a number,
including when the input itself is WebP. Failed conversions remove temporary
files. Batches run sequentially to bound memory use and can be stopped after the
current image. Failed images can be retried without rerunning completed jobs.

## Supported inputs and limits

- PNG, JPEG/JFIF, TIFF, WebP, GIF, BMP, ICO, SVG, PNM/PBM/PGM/PPM/PAM,
  TGA, DDS, HDR, EXR, farbfeld (`.ff`) and QOI.
- HEIC, AVIF, camera RAW, PSD and PDF are not supported by this build. The name
  describes the workflow; support depends on the available decoders.
- Animated inputs export a still first frame; TIFF/ICO export one image.
- SVG rasterizes at its intrinsic size with system fonts. External image/file
  references are disabled. Embedded data images are allowed.
- HDR/EXR normalize to 8-bit RGBA without tone mapping. This is not a color-managed
  or archival conversion pipeline.
- Maximum input: 512 MB, 100 megapixels, 16,383 pixels per side. Rust decoder
  allocations are limited to 512 MB where supported by the decoder.
- The default Rust normalization applies available orientation metadata and strips
  metadata. Retaining metadata instead passes native PNG/JPEG/TIFF/WebP directly
  to cwebp; EXIF rotation is then retained only with EXIF/all, not baked into pixels.
  Animated PNG/WebP may be rejected in this mode. Other formats normalize and
  cannot retain metadata; a warning appears on the result.

## Structure

- `src/lib/converter/workspace.svelte.ts`: reactive queue state and desktop I/O.
- `src/lib/converter/settings.ts` and `types.ts`: defaults, profiles, persistence
  and IPC contracts. Settings validation is authoritative in Rust.
- `src/lib/components`: settings UI and reusable input controls.
- `src/routes/+page.svelte`: workspace composition, native drop listener lifecycle.
- `src-tauri/src/settings.rs`: validated, typed cwebp argument construction.
- `src-tauri/src/converter.rs`: input normalization, process execution, unique
  output commit and scoped temporary files.
- `src-tauri/src/lib.rs`: thin asynchronous IPC commands using blocking workers.
- `src-tauri/binaries`: official encoder, provenance and required license notices.

The frontend has native dialog and reveal permissions, not arbitrary shell access.
Rust invokes the bundled executable with separate arguments and no command shell.
The SvelteKit static adapter and disabled SSR follow the Tauri integration guide.

## Validation

```sh
npm run check
npm run build
npm run test:rust
npm run test:ui
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run format
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Rust tests use the actual bundled Windows encoder, exercise multiple formats and
compression modes, check resize results and ensure errors do not leave outputs.
Run encoder tests on Windows x64. UI tests use installed Google Chrome and the
production build (`npm run build` first). They exercise settings persistence,
queue deduplication, output selection, conversion results, stop/resume and retry
through a mocked native IPC bridge; they do not replace a native WebView smoke test.

## Upstream references

- [Svelte 5 documentation](https://svelte.dev/docs/svelte/overview)
- [Tauri SvelteKit integration](https://v2.tauri.app/start/frontend/sveltekit/)
- [Tailwind with Vite](https://tailwindcss.com/docs/installation/using-vite)
- [cwebp options](https://developers.google.com/speed/webp/docs/cwebp)
- [Official WebP downloads](https://developers.google.com/speed/webp/download)

Encoder source and SHA-256 are recorded in `src-tauri/binaries/NOTICE.md`.
The supplied encoder was upgraded in this repo only; the original external file
was not changed. Other operating systems require their own compatible encoder.
