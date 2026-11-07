use serde::{Deserialize, Serialize};

/// Typed encoder options. Arguments are constructed individually, never through a shell.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EncoderSettings {
    pub mode: String,
    pub preset: String,
    pub quality: f32,
    pub method: u8,
    pub alpha_quality: u8,
    pub near_lossless: u8,
    pub metadata: String,
    pub width: u32,
    pub height: u32,
    pub crop: Option<Crop>,
    pub target_size_kb: u32,
    pub target_psnr: f32,
    pub passes: u8,
    pub filter_strength: u8,
    pub sharpness: u8,
    pub sns: u8,
    pub auto_filter: bool,
    pub sharp_yuv: bool,
    pub exact: bool,
    pub multithreading: bool,
    pub low_memory: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Crop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for EncoderSettings {
    fn default() -> Self {
        Self {
            mode: "lossy".into(),
            preset: "default".into(),
            quality: 80.0,
            method: 4,
            alpha_quality: 100,
            near_lossless: 60,
            metadata: "none".into(),
            width: 0,
            height: 0,
            crop: None,
            target_size_kb: 0,
            target_psnr: 0.0,
            passes: 6,
            filter_strength: 60,
            sharpness: 0,
            sns: 50,
            auto_filter: false,
            sharp_yuv: true,
            exact: false,
            multithreading: true,
            low_memory: false,
        }
    }
}

impl EncoderSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !["lossy", "lossless", "nearLossless"].contains(&self.mode.as_str()) {
            return Err("Unknown compression mode.".into());
        }
        if !["default", "photo", "picture", "drawing", "icon", "text"]
            .contains(&self.preset.as_str())
        {
            return Err("Unknown content preset.".into());
        }
        if !["none", "icc", "exif", "xmp", "all"].contains(&self.metadata.as_str()) {
            return Err("Unknown metadata policy.".into());
        }
        if !self.quality.is_finite()
            || !(0.0..=100.0).contains(&self.quality)
            || self.method > 6
            || self.alpha_quality > 100
            || self.near_lossless > 100
            || self.filter_strength > 100
            || self.sharpness > 7
            || self.sns > 100
            || !(1..=10).contains(&self.passes)
        {
            return Err("An encoder option is outside its supported range.".into());
        }
        if self.width > 16383 || self.height > 16383 {
            return Err("WebP dimensions cannot exceed 16,383 pixels.".into());
        }
        if !self.target_psnr.is_finite()
            || !(0.0..=100.0).contains(&self.target_psnr)
            || self.target_size_kb > 1_000_000
        {
            return Err("Invalid target size or PSNR.".into());
        }
        if self.target_size_kb > 0 && self.target_psnr > 0.0 {
            return Err("Choose either target size or target PSNR.".into());
        }
        if let Some(crop) = &self.crop
            && (crop.width == 0 || crop.height == 0)
        {
            return Err("Crop width and height must be positive.".into());
        }
        Ok(())
    }

    pub fn arguments(&self) -> Vec<String> {
        // cwebp resets several parameters when applying a preset, so it must come first.
        let mut args = vec!["-preset".into(), self.preset.clone()];
        let mut option = |key: &str, value: String| {
            args.extend([key.into(), value]);
        };
        option("-q", self.quality.to_string());
        option("-m", self.method.to_string());
        option("-alpha_q", self.alpha_quality.to_string());
        option("-metadata", self.metadata.clone());
        if self.mode == "nearLossless" {
            option("-near_lossless", self.near_lossless.to_string());
        }
        if self.mode == "lossy" {
            option("-f", self.filter_strength.to_string());
            option("-sharpness", self.sharpness.to_string());
            option("-sns", self.sns.to_string());
            if self.target_size_kb > 0 {
                option("-size", (self.target_size_kb * 1024).to_string());
            }
            if self.target_psnr > 0.0 {
                option("-psnr", self.target_psnr.to_string());
            }
            // -pass by itself silently introduces a PSNR target; only send it with a target.
            if self.target_size_kb > 0 || self.target_psnr > 0.0 {
                option("-pass", self.passes.to_string());
            }
        }
        if self.mode == "lossless" {
            args.push("-lossless".into());
        }
        if let Some(crop) = &self.crop {
            args.extend([
                "-crop".into(),
                crop.x.to_string(),
                crop.y.to_string(),
                crop.width.to_string(),
                crop.height.to_string(),
            ]);
        }
        if self.width > 0 || self.height > 0 {
            args.extend([
                "-resize".into(),
                self.width.to_string(),
                self.height.to_string(),
            ]);
        }
        for (enabled, flag) in [
            (self.multithreading, "-mt"),
            (self.exact, "-exact"),
            (self.low_memory && self.mode == "lossy", "-low_memory"),
            (self.sharp_yuv && self.mode == "lossy", "-sharp_yuv"),
            (self.auto_filter && self.mode == "lossy", "-af"),
        ] {
            if enabled {
                args.push(flag.into());
            }
        }
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_do_not_accidentally_enable_target_psnr() {
        let settings = EncoderSettings::default();
        settings.validate().unwrap();
        let args = settings.arguments();
        assert_eq!(&args[..2], ["-preset", "default"]);
        assert!(!args.iter().any(|arg| arg == "-pass"));
    }

    #[test]
    fn rejects_invalid_and_conflicting_settings() {
        let mut settings = EncoderSettings {
            quality: f32::NAN,
            ..Default::default()
        };
        assert!(settings.validate().is_err());
        settings.quality = 80.0;
        settings.target_size_kb = 100;
        settings.target_psnr = 40.0;
        assert!(settings.validate().is_err());
    }
}
