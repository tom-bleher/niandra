//! Async art loading utilities

use gtk4::gdk;
use gtk4::gio;
use gtk4::prelude::*;

/// Load album art from a URL and return a texture.
///
/// Supports `file://` URLs (local files) and `http(s)://` URLs.
pub async fn load_art_texture(url: &str) -> Result<gdk::Texture, ArtLoadError> {
    let file = gio::File::for_uri(url);

    let (bytes, _etag) = file
        .load_bytes_future()
        .await
        .map_err(|e| ArtLoadError::LoadFailed(e.to_string()))?;

    let texture = gdk::Texture::from_bytes(&bytes)
        .map_err(|e| ArtLoadError::InvalidImage(e.to_string()))?;

    Ok(texture)
}

/// Create a placeholder paintable using a symbolic icon.
pub fn placeholder_paintable(widget: &impl IsA<gtk4::Widget>) -> Option<gdk::Paintable> {
    let icon_theme = gtk4::IconTheme::for_display(&widget.display());
    let icon_paintable = icon_theme.lookup_icon(
        "media-optical-symbolic",
        &[],
        48,
        1,
        gtk4::TextDirection::None,
        gtk4::IconLookupFlags::empty(),
    );
    Some(icon_paintable.upcast())
}

/// Errors that can occur when loading art.
#[derive(Debug)]
pub enum ArtLoadError {
    /// Failed to load the file/URL.
    LoadFailed(String),
    /// The data is not a valid image.
    InvalidImage(String),
}

impl std::fmt::Display for ArtLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtLoadError::LoadFailed(e) => write!(f, "Failed to load art: {e}"),
            ArtLoadError::InvalidImage(e) => write!(f, "Invalid image data: {e}"),
        }
    }
}

impl std::error::Error for ArtLoadError {}
