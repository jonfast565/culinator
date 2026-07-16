use async_trait::async_trait;
use base64::Engine;
use culinator_models::{ApplicationError, ImportSettings, OcrEngine, RecipeImage};
use std::{fs, process::Command};

#[derive(Debug, Default, Clone, Copy)]
pub struct TesseractCommandOcr;

#[async_trait]
impl OcrEngine for TesseractCommandOcr {
    async fn extract_text(
        &self,
        images: &[RecipeImage],
        settings: &ImportSettings,
    ) -> Result<Option<String>, ApplicationError> {
        if !settings.use_local_ocr {
            return Ok(None);
        }
        let dir = tempfile::tempdir().map_err(|e| ApplicationError::Internal(e.to_string()))?;
        let mut pages = Vec::new();
        for (index, image) in images.iter().enumerate() {
            let extension = match image.media_type.as_str() {
                "image/png" => "png",
                "image/webp" => "webp",
                _ => "jpg",
            };
            let path = dir.path().join(format!("page-{index}.{extension}"));
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&image.data_base64)
                .map_err(|e| ApplicationError::InvalidInput(format!("invalid image data: {e}")))?;
            fs::write(&path, bytes).map_err(|e| ApplicationError::Internal(e.to_string()))?;
            let output = Command::new(&settings.tesseract_command)
                .arg(&path)
                .arg("stdout")
                .arg("-l")
                .arg("eng")
                .output();
            match output {
                Ok(result) if result.status.success() => {
                    pages.push(String::from_utf8_lossy(&result.stdout).trim().to_owned())
                }
                Ok(_) | Err(_) => return Ok(None),
            }
        }
        let text = pages
            .into_iter()
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n--- page ---\n\n");
        Ok((!text.is_empty()).then_some(text))
    }
}
#[cfg(test)]
mod test;
