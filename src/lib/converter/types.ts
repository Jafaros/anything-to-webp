export interface EncoderSettings {
  mode: "lossy" | "lossless" | "nearLossless";
  preset: "default" | "photo" | "picture" | "drawing" | "icon" | "text";
  quality: number;
  method: number;
  alphaQuality: number;
  nearLossless: number;
  metadata: "none" | "icc" | "exif" | "xmp" | "all";
  width: number;
  height: number;
  crop: { x: number; y: number; width: number; height: number } | null;
  targetSizeKb: number;
  targetPsnr: number;
  passes: number;
  filterStrength: number;
  sharpness: number;
  sns: number;
  autoFilter: boolean;
  sharpYuv: boolean;
  exact: boolean;
  multithreading: boolean;
  lowMemory: boolean;
}

export interface InputFile {
  path: string;
  name: string;
  bytes: number;
  extension: string;
  warning: string | null;
  error: string | null;
}

export interface ConversionResult {
  outputPath: string;
  inputBytes: number;
  outputBytes: number;
  elapsedMs: number;
  warning: string | null;
}

export interface QueueItem extends InputFile {
  status: "ready" | "converting" | "done" | "error";
  result?: ConversionResult;
}
