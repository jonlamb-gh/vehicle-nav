use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Failed to load texture from {}", _0)]
    TextureLoadFromFile(String),
}
