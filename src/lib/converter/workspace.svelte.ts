import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { defaultSettings, loadSettings, saveSettings } from "./settings";
import type {
  ConversionResult,
  EncoderSettings,
  InputFile,
  QueueItem,
} from "./types";

export const supportedExtensions = [
  "png",
  "jpg",
  "jpeg",
  "jfif",
  "tif",
  "tiff",
  "webp",
  "gif",
  "bmp",
  "ico",
  "pnm",
  "pbm",
  "pgm",
  "ppm",
  "pam",
  "tga",
  "dds",
  "hdr",
  "exr",
  "ff",
  "qoi",
  "svg",
];

/** Owns desktop I/O and queue transitions; components only render state and express intent. */
export class Workspace {
  settings = $state<EncoderSettings>(structuredClone(defaultSettings));
  files = $state<QueueItem[]>([]);
  outputDirectory = $state<string | null>(null);
  running = $state(false);
  adding = $state(false);
  stopRequested = $state(false);
  error = $state("");
  encoderVersion = $state("Checking encoder…");
  notice = $state("");

  ready = $derived(this.files.filter((file) => file.status === "ready"));
  completed = $derived(this.files.filter((file) => file.status === "done"));
  totalBytes = $derived(this.files.reduce((sum, file) => sum + file.bytes, 0));
  savedBytes = $derived(
    this.completed.reduce(
      (sum, file) => sum + file.bytes - (file.result?.outputBytes ?? 0),
      0,
    ),
  );

  async initialize() {
    this.settings = loadSettings();
    if (!isTauri()) {
      this.encoderVersion = "Desktop app required";
      return;
    }
    try {
      this.encoderVersion = `cwebp ${await invoke<string>("encoder_version")}`;
    } catch (error) {
      this.encoderVersion = "Encoder unavailable";
      this.error = String(error);
    }
  }

  async chooseFiles() {
    if (!this.requireDesktop()) return;
    try {
      const paths = await open({
        multiple: true,
        title: "Add images",
        filters: [
          { name: "Supported images", extensions: supportedExtensions },
          { name: "All files", extensions: ["*"] },
        ],
      });
      if (paths) await this.addPaths(Array.isArray(paths) ? paths : [paths]);
    } catch (error) {
      this.error = String(error);
    }
  }

  async addPaths(paths: string[]) {
    if (this.running || !paths.length) return;
    this.adding = true;
    this.error = "";
    try {
      const files = await invoke<InputFile[]>("inspect_files", { paths });
      const existing = new Set(
        this.files.map((file) => file.path.toLowerCase()),
      );
      const fresh = files.filter((file) => {
        const key = file.path.toLowerCase();
        if (existing.has(key)) return false;
        existing.add(key);
        return true;
      });
      this.files.push(
        ...fresh.map((file): QueueItem => ({
          ...file,
          status: file.error ? "error" : "ready",
        })),
      );
      this.notice = `${fresh.length} image${fresh.length === 1 ? "" : "s"} added`;
    } catch (error) {
      this.error = String(error);
    } finally {
      this.adding = false;
    }
  }

  async chooseOutput() {
    if (!this.requireDesktop()) return;
    try {
      const directory = await open({
        directory: true,
        title: "Choose output folder",
      });
      if (typeof directory === "string") this.outputDirectory = directory;
    } catch (error) {
      this.error = String(error);
    }
  }

  async convert() {
    if (this.running || !this.ready.length || !this.requireDesktop()) return;
    const settings = $state.snapshot(this.settings);
    try {
      await invoke("validate_settings", { settings });
    } catch (error) {
      this.error = String(error);
      return;
    }
    if (this.running) return;
    this.running = true;
    this.stopRequested = false;
    this.error = "";
    this.notice = "";
    const directory = this.outputDirectory;
    saveSettings(settings);
    // Sequential jobs bound memory usage; cwebp can use multiple threads within each job.
    const pending = this.files.filter((file) => file.status === "ready");
    try {
      for (const file of pending) {
        if (this.stopRequested) break;
        file.status = "converting";
        try {
          file.result = await invoke<ConversionResult>("convert_image", {
            inputPath: file.path,
            outputDirectory: directory,
            settings,
          });
          file.status = "done";
        } catch (error) {
          file.error = String(error);
          file.status = "error";
        }
      }
      this.notice = this.stopRequested
        ? "Stopped. Remaining images are ready to convert."
        : "Batch finished. Your images are ready.";
    } finally {
      this.running = false;
      this.stopRequested = false;
    }
  }

  retryErrors() {
    for (const file of this.files) {
      if (
        file.status === "error" &&
        supportedExtensions.includes(file.extension)
      ) {
        file.status = "ready";
        file.error = null;
      }
    }
  }

  async reveal(path: string) {
    try {
      await revealItemInDir(path);
    } catch (error) {
      this.error = String(error);
    }
  }

  private requireDesktop(): boolean {
    if (isTauri()) return true;
    this.error =
      "Run the desktop app with npm run tauri dev to access local images and the encoder.";
    return false;
  }
}
