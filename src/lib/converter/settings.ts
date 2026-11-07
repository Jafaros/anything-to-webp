import type { EncoderSettings } from "./types";

export const defaultSettings: EncoderSettings = {
  mode: "lossy",
  preset: "default",
  quality: 80,
  method: 4,
  alphaQuality: 100,
  nearLossless: 60,
  metadata: "none",
  width: 0,
  height: 0,
  crop: null,
  targetSizeKb: 0,
  targetPsnr: 0,
  passes: 6,
  filterStrength: 60,
  sharpness: 0,
  sns: 50,
  autoFilter: false,
  sharpYuv: true,
  exact: false,
  multithreading: true,
  lowMemory: false,
};

export const profiles = {
  balanced: {
    label: "Balanced",
    description: "A little smaller. Still beautiful.",
    settings: { quality: 80, method: 4 },
  },
  compact: {
    label: "Smallest",
    description: "More space for everything else.",
    settings: { quality: 65, method: 6 },
  },
  quality: {
    label: "High quality",
    description: "Keep the details that matter.",
    settings: { quality: 92, method: 6 },
  },
  lossless: {
    label: "Lossless",
    description: "Every visible pixel, preserved.",
    settings: { mode: "lossless", quality: 100, method: 6 },
  },
} as const;

const storageKey = "anything-to-webp.settings.v1";

export function loadSettings(): EncoderSettings {
  try {
    const saved: unknown = JSON.parse(
      localStorage.getItem(storageKey) ?? "null",
    );
    if (!saved || typeof saved !== "object")
      return structuredClone(defaultSettings);
    // Only restore known fields of the expected type. Rust validates all ranges before encoding.
    const restored = structuredClone(defaultSettings);
    for (const key of Object.keys(
      defaultSettings,
    ) as (keyof EncoderSettings)[]) {
      if (key === "crop") continue;
      const value = (saved as Record<string, unknown>)[key];
      if (
        typeof value === typeof defaultSettings[key] &&
        (typeof value !== "number" || Number.isFinite(value))
      ) {
        Object.assign(restored, { [key]: value });
      }
    }
    return restored;
  } catch {
    return structuredClone(defaultSettings);
  }
}

export function saveSettings(settings: EncoderSettings): void {
  try {
    localStorage.setItem(storageKey, JSON.stringify(settings));
  } catch {
    /* Conversion remains available if storage is disabled or full. */
  }
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const unit = bytes < 1024 * 1024 ? "KB" : "MB";
  return `${(bytes / (unit === "KB" ? 1024 : 1024 * 1024)).toFixed(1)} ${unit}`;
}
