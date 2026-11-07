use crate::settings::EncoderSettings;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::Instant,
};
use tempfile::NamedTempFile;

pub const EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "jfif", "tif", "tiff", "webp", "gif", "bmp", "ico", "pnm", "pbm", "pgm",
    "ppm", "pam", "tga", "dds", "hdr", "exr", "ff", "qoi", "svg",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputFile {
    pub path: String,
    pub name: String,
    pub bytes: u64,
    pub extension: String,
    pub warning: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub elapsed_ms: u128,
    pub warning: Option<String>,
}

fn extension(path: &Path) -> String {
    path.extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

pub fn inspect(path: String) -> InputFile {
    let source = Path::new(&path);
    let ext = extension(source);
    let metadata = fs::metadata(source);
    let error = match &metadata {
        Ok(info) if !info.is_file() => Some("Choose an image file, not a folder.".into()),
        Err(error) => Some(format!("Cannot read this file: {error}")),
        _ if !EXTENSIONS.contains(&ext.as_str()) => Some(
            "Unsupported format. HEIC, AVIF, RAW, PSD and PDF are not supported by this build."
                .into(),
        ),
        _ => None,
    };
    InputFile {
        name: source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        bytes: metadata.map(|info| info.len()).unwrap_or_default(),
        extension: ext.clone(),
        warning: match ext.as_str() {
            "gif" | "webp" | "png" => {
                Some("Animated images are exported as a still first frame.".into())
            }
            "svg" => Some(
                "SVG is rasterized at its intrinsic size. External resources are disabled.".into(),
            ),
            "hdr" | "exr" => {
                Some("HDR pixels are converted to 8-bit RGBA without tone mapping.".into())
            }
            "tif" | "tiff" | "ico" => {
                Some("Only the first image or selected icon image is exported.".into())
            }
            _ => None,
        },
        path,
        error,
    }
}

fn run(command: &mut Command) -> Result<Output, String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    command
        .output()
        .map_err(|error| format!("Could not start the bundled cwebp encoder: {error}"))
}

pub fn version(binary: &Path) -> Result<String, String> {
    let output = run(Command::new(binary).arg("-version"))?;
    if !output.status.success() {
        return Err("The encoder version check failed.".into());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .trim()
        .into())
}

/// Decoder fallback broadens cwebp's native inputs. Temporary images are scoped to this job.
fn prepare_input(
    source: &Path,
    directory: &Path,
    preserve_metadata: bool,
) -> Result<(PathBuf, Option<NamedTempFile>), String> {
    let ext = extension(source);
    // Native inputs retain metadata when requested. cwebp rejects animated PNG/WebP.
    if preserve_metadata
        && ["jpg", "jpeg", "jfif", "tif", "tiff", "png", "webp"].contains(&ext.as_str())
    {
        return Ok((source.to_owned(), None));
    }
    let temporary = tempfile::Builder::new()
        .suffix(".png")
        .tempfile_in(directory)
        .map_err(|error| format!("Cannot create temporary image: {error}"))?;
    if ext == "svg" {
        let data = fs::read(source).map_err(|error| error.to_string())?;
        let mut options = resvg::usvg::Options {
            image_href_resolver: resvg::usvg::ImageHrefResolver {
                resolve_data: resvg::usvg::ImageHrefResolver::default_data_resolver(),
                resolve_string: Box::new(|_, _| None),
            },
            ..Default::default()
        };
        options.fontdb_mut().load_system_fonts();
        let tree = resvg::usvg::Tree::from_data(&data, &options)
            .map_err(|error| format!("Invalid SVG: {error}"))?;
        let size = tree.size().to_int_size();
        check_dimensions(size.width(), size.height())?;
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
            .ok_or("Could not allocate the SVG image.")?;
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );
        pixmap
            .save_png(temporary.path())
            .map_err(|error| error.to_string())?;
    } else {
        let mut reader = image::ImageReader::open(source)
            .map_err(|error| error.to_string())?
            .with_guessed_format()
            .map_err(|error| error.to_string())?;
        let mut limits = image::Limits::default();
        limits.max_image_width = Some(16383);
        limits.max_image_height = Some(16383);
        limits.max_alloc = Some(512 * 1024 * 1024);
        reader.limits(limits);
        let mut decoder = reader
            .into_decoder()
            .map_err(|error| format!("Cannot decode this image: {error}"))?;
        use image::ImageDecoder;
        let orientation = decoder
            .orientation()
            .unwrap_or(image::metadata::Orientation::NoTransforms);
        let mut decoded = image::DynamicImage::from_decoder(decoder)
            .map_err(|error| format!("Cannot decode this image: {error}"))?;
        decoded.apply_orientation(orientation);
        check_dimensions(decoded.width(), decoded.height())?;
        decoded
            .to_rgba8()
            .save_with_format(temporary.path(), image::ImageFormat::Png)
            .map_err(|error| format!("Cannot prepare image: {error}"))?;
    }
    Ok((temporary.path().to_owned(), Some(temporary)))
}

fn check_dimensions(width: u32, height: u32) -> Result<(), String> {
    if width == 0
        || height == 0
        || width > 16383
        || height > 16383
        || u64::from(width) * u64::from(height) > 100_000_000
    {
        return Err(
            "Image exceeds the supported dimensions (16,383 per side / 100 megapixels).".into(),
        );
    }
    Ok(())
}

/// Commit without overwriting, even if another process creates the filename during encoding.
fn persist_unique(
    mut file: NamedTempFile,
    directory: &Path,
    stem: &str,
) -> Result<PathBuf, String> {
    for index in 0..100_000 {
        let name = if index == 0 {
            format!("{stem}.webp")
        } else {
            format!("{stem} ({index}).webp")
        };
        let destination = directory.join(name);
        match file.persist_noclobber(&destination) {
            Ok(_) => return Ok(destination),
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                file = error.file;
            }
            Err(error) => return Err(format!("Cannot save converted image: {}", error.error)),
        }
    }
    Err("Too many images with the same output name.".into())
}

pub fn convert(
    binary: &Path,
    input: &Path,
    output_directory: Option<&Path>,
    settings: &EncoderSettings,
) -> Result<ConversionResult, String> {
    settings.validate()?;
    let source = input
        .canonicalize()
        .map_err(|error| format!("Cannot open input: {error}"))?;
    let info = inspect(source.to_string_lossy().into_owned());
    if let Some(error) = info.error {
        return Err(error);
    }
    if info.bytes > 512 * 1024 * 1024 {
        return Err("Input exceeds the 512 MB file limit.".into());
    }
    let directory = output_directory
        .unwrap_or_else(|| source.parent().unwrap())
        .canonicalize()
        .map_err(|error| format!("Cannot open output folder: {error}"))?;
    if !directory.is_dir() {
        return Err("Output location must be a folder.".into());
    }
    let started = Instant::now();
    let (prepared, temporary_input) =
        prepare_input(&source, &directory, settings.metadata != "none")?;
    let (width, height) = image::ImageReader::open(&prepared)
        .map_err(|error| error.to_string())?
        .with_guessed_format()
        .map_err(|error| error.to_string())?
        .into_dimensions()
        .map_err(|error| format!("Cannot inspect image dimensions: {error}"))?;
    check_dimensions(width, height)?;
    if let Some(crop) = &settings.crop
        && (crop
            .x
            .checked_add(crop.width)
            .is_none_or(|right| right > width)
            || crop
                .y
                .checked_add(crop.height)
                .is_none_or(|bottom| bottom > height))
    {
        return Err("The crop rectangle extends outside the input image.".into());
    }
    let output = tempfile::Builder::new()
        .prefix(".webp-")
        .suffix(".webp")
        .tempfile_in(&directory)
        .map_err(|error| format!("Cannot write to output folder: {error}"))?;
    let result = run(Command::new(binary)
        .args(settings.arguments())
        .arg("-o")
        .arg(output.path())
        .arg("--")
        .arg(&prepared))?;
    if !result.status.success() {
        return Err(format!(
            "cwebp failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    let output_bytes = output
        .as_file()
        .metadata()
        .map_err(|error| error.to_string())?
        .len();
    if output_bytes == 0 {
        return Err("The encoder produced an empty file.".into());
    }
    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let output_path = persist_unique(output, &directory, &stem)?;
    let warning = if temporary_input.is_some() && settings.metadata != "none" {
        Some("Metadata cannot be retained for inputs normalized through the Rust decoder.".into())
    } else {
        info.warning
    };
    Ok(ConversionResult {
        output_path: output_path.to_string_lossy().into_owned(),
        input_bytes: info.bytes,
        output_bytes,
        elapsed_ms: started.elapsed().as_millis(),
        warning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_encoder_converts_formats_and_preserves_existing_files() {
        let directory = tempfile::tempdir().unwrap();
        let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/cwebp.exe");
        for ext in ["png", "bmp", "gif", "jpg", "tiff"] {
            let source = directory.path().join(format!("test.{ext}"));
            image::RgbImage::from_pixel(16, 12, image::Rgb([20, 100, 200]))
                .save(&source)
                .unwrap();
            for mode in ["lossy", "lossless", "nearLossless"] {
                let settings = EncoderSettings {
                    mode: mode.into(),
                    width: 8,
                    ..Default::default()
                };
                let result = convert(&binary, &source, None, &settings).unwrap();
                let decoded = image::open(&result.output_path).unwrap();
                assert_eq!((decoded.width(), decoded.height()), (8, 6));
                assert!(source.exists());
            }
        }
        let source = directory.path().join("vector.svg");
        fs::write(&source, r#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="red"/></svg>"#).unwrap();
        let result = convert(&binary, &source, None, &EncoderSettings::default()).unwrap();
        assert_eq!(
            image::image_dimensions(result.output_path).unwrap(),
            (20, 10)
        );
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| extension(&entry.path()) == "webp")
                .count(),
            16
        );
    }

    #[test]
    fn webp_input_crop_alpha_and_unicode_paths_are_safe() {
        let directory = tempfile::tempdir().unwrap();
        let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/cwebp.exe");
        let source = directory.path().join("-žluťoučký obrázek.webp");
        image::RgbaImage::from_pixel(16, 12, image::Rgba([13, 89, 154, 128]))
            .save(&source)
            .unwrap();
        let original = fs::read(&source).unwrap();
        let settings = EncoderSettings {
            mode: "lossless".into(),
            exact: true,
            width: 4,
            crop: Some(crate::settings::Crop {
                x: 2,
                y: 2,
                width: 8,
                height: 8,
            }),
            ..Default::default()
        };
        let result = convert(&binary, &source, None, &settings).unwrap();
        let output = image::open(&result.output_path).unwrap().to_rgba8();
        assert_eq!(output.dimensions(), (4, 4));
        assert_eq!(output.get_pixel(0, 0).0, [13, 89, 154, 128]);
        assert_ne!(Path::new(&result.output_path), source);
        assert_eq!(fs::read(&source).unwrap(), original);

        let invalid = EncoderSettings {
            crop: Some(crate::settings::Crop {
                x: u32::MAX,
                y: 0,
                width: 1,
                height: 1,
            }),
            ..Default::default()
        };
        assert!(
            convert(&binary, &source, None, &invalid)
                .err()
                .expect("invalid crop must fail")
                .contains("crop rectangle")
        );
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn errors_leave_no_partial_files() {
        let directory = tempfile::tempdir().unwrap();
        let binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/cwebp.exe");
        let source = directory.path().join("broken.png");
        fs::write(&source, b"not an image").unwrap();
        assert!(convert(&binary, &source, None, &EncoderSettings::default()).is_err());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
