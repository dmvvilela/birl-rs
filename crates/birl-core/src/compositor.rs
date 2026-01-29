use anyhow::{Context, Result};
use bytes::Bytes;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::Cursor;
use tracing::{debug, info};

/// Composite multiple PNG layers over a base JPEG image
pub struct Compositor {
    base_image: DynamicImage,
}

impl Compositor {
    /// Create a new compositor with a base image
    pub fn new(base_image_data: &[u8]) -> Result<Self> {
        let reader = ImageReader::new(Cursor::new(base_image_data))
            .with_guessed_format()
            .context("Failed to guess image format")?;

        let base_image = reader.decode().context("Failed to decode base image")?;

        debug!("Loaded base image: {}x{}", base_image.width(), base_image.height());

        Ok(Self { base_image })
    }

    /// Add a layer to the composite
    pub fn add_layer(&mut self, layer_data: &[u8]) -> Result<()> {
        let reader = ImageReader::new(Cursor::new(layer_data))
            .with_guessed_format()
            .context("Failed to guess layer format")?;

        let layer = reader.decode().context("Failed to decode layer image")?;

        debug!("Adding layer: {}x{}", layer.width(), layer.height());

        // Ensure the layer matches the base image size
        let layer = if layer.width() != self.base_image.width()
            || layer.height() != self.base_image.height()
        {
            debug!(
                "Resizing layer from {}x{} to {}x{}",
                layer.width(),
                layer.height(),
                self.base_image.width(),
                self.base_image.height()
            );
            layer.resize_exact(
                self.base_image.width(),
                self.base_image.height(),
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            layer
        };

        // Composite the layer over the base using alpha blending
        image::imageops::overlay(&mut self.base_image, &layer, 0, 0);

        Ok(())
    }

    /// Finalize and encode the composite as JPEG
    pub fn finalize(self) -> Result<Bytes> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        self.base_image
            .write_to(&mut cursor, ImageFormat::Jpeg)
            .context("Failed to encode composite as JPEG")?;

        info!("Composite created: {} bytes", buffer.len());

        Ok(Bytes::from(buffer))
    }

    /// Get the width and height of the base image
    pub fn dimensions(&self) -> (u32, u32) {
        (self.base_image.width(), self.base_image.height())
    }
}

/// Composite multiple layers over a base image in one operation
pub fn compose_layers(base_image_data: &[u8], layers: Vec<Bytes>) -> Result<Bytes> {
    let start = std::time::Instant::now();

    let mut compositor = Compositor::new(base_image_data)?;

    for (idx, layer_data) in layers.iter().enumerate() {
        compositor
            .add_layer(layer_data)
            .with_context(|| format!("Failed to add layer {}", idx))?;
    }

    let result = compositor.finalize()?;

    info!("Image composition took {:?}", start.elapsed());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32, r: u8, g: u8, b: u8) -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            width,
            height,
            image::Rgb([r, g, b]),
        ));
        let mut buffer = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Jpeg)
            .unwrap();
        buffer
    }

    fn create_test_layer(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
        let img = DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            width,
            height,
            image::Rgba([r, g, b, a]),
        ));
        let mut buffer = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .unwrap();
        buffer
    }

    #[test]
    fn test_compositor_creation() {
        let base = create_test_image(100, 100, 255, 0, 0);
        let compositor = Compositor::new(&base);
        assert!(compositor.is_ok());
    }

    #[test]
    fn test_compositor_dimensions() {
        let base = create_test_image(100, 100, 255, 0, 0);
        let compositor = Compositor::new(&base).unwrap();
        assert_eq!(compositor.dimensions(), (100, 100));
    }

    #[test]
    fn test_add_layer() {
        let base = create_test_image(100, 100, 255, 0, 0);
        let mut compositor = Compositor::new(&base).unwrap();
        let layer = create_test_layer(100, 100, 0, 255, 0, 128);
        assert!(compositor.add_layer(&layer).is_ok());
    }

    #[test]
    fn test_compose_layers() {
        let base = create_test_image(100, 100, 255, 0, 0);
        let layer1 = create_test_layer(100, 100, 0, 255, 0, 128);
        let layer2 = create_test_layer(100, 100, 0, 0, 255, 128);
        let layers = vec![Bytes::from(layer1), Bytes::from(layer2)];

        let result = compose_layers(&base, layers);
        assert!(result.is_ok());
        let composite = result.unwrap();
        assert!(!composite.is_empty());
    }
}
