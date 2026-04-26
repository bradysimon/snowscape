//! Screenshot capture for the [`Emulator`].
//! Allows saving screenshots to a file on test failure.

use std::path::Path;

use super::{Emulator, Error, Result};

/// A captured screenshot from the [`Emulator`].
pub struct Screenshot {
    /// Raw RGBA pixel data.
    pub rgba: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl Screenshot {
    /// Saves the screenshot as a PNG file at the given path.
    ///
    /// Parent directories are created automatically.
    pub fn save_png(&self, path: impl AsRef<Path>) -> Result {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let encoder = image::codecs::png::PngEncoder::new(file);
        image::ImageEncoder::write_image(
            encoder,
            &self.rgba,
            self.width,
            self.height,
            image::ColorType::Rgba8.into(),
        )
        .map_err(|e| Error::Io(std::io::Error::other(e)))?;
        Ok(())
    }
}

impl<P: iced_test::program::Program + 'static> Emulator<P> {
    // MARK: - Screenshots

    /// Takes a screenshot of the current emulator state at the given scale
    /// factor and returns the raw RGBA bytes along with the pixel dimensions.
    pub fn screenshot(&mut self, scale_factor: f32) -> Screenshot {
        use iced_test::core::theme::Base;

        let theme = self
            .inner
            .theme(&self.program)
            .unwrap_or_else(|| <P::Theme as Base>::default(iced_test::core::theme::Mode::None));

        let shot = self.inner.screenshot(&self.program, &theme, scale_factor);

        Screenshot {
            rgba: shot.rgba.to_vec(),
            width: shot.size.width,
            height: shot.size.height,
        }
    }

    /// Saves a PNG screenshot of the current state to the given path.
    pub fn save_screenshot(&mut self, path: impl AsRef<Path>, scale_factor: f32) -> Result {
        let screenshot = self.screenshot(scale_factor);
        screenshot.save_png(path)
    }

    /// Runs a closure and, if it returns an error, automatically saves a
    /// screenshot to the directory configured via
    /// [`Builder::screenshot_on_failure`].
    ///
    /// The screenshot file is named `{label}.png`. If no screenshot directory
    /// was configured, the error is returned as-is.
    pub fn try_with_screenshot<T>(
        &mut self,
        label: &str,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        match f(self) {
            Ok(val) => Ok(val),
            Err(err) => {
                if let Some(dir) = &self.screenshot_dir {
                    let path = dir.join(format!("{label}.png"));
                    let _ = self.save_screenshot(&path, 1.0);
                    eprintln!("screenshot saved to {}", path.display());
                }
                Err(err)
            }
        }
    }
}
