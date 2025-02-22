use crate::core::req_body::ReqBody;
use crate::header::{CONTENT_TYPE, HeaderMap};
use crate::multer::{Field, Multipart};
use crate::{SilentError, StatusCode};
use multimap::MultiMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tempfile::Builder;
use textnonce::TextNonce;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// The extracted text fields and uploaded files from a `multipart/form-data` request.
#[derive(Debug)]
pub struct FormData {
    /// Name-value pairs for plain text fields. Technically, these are form data parts with no
    /// filename specified in the part's `Content-Disposition`.
    pub fields: MultiMap<String, String>,
    /// Name-value pairs for temporary files. Technically, these are form data parts with a filename
    /// specified in the part's `Content-Disposition`.
    #[cfg(feature = "server")]
    pub files: MultiMap<String, FilePart>,
}

impl FormData {
    /// Create new `FormData`.
    #[inline]
    pub fn new() -> FormData {
        FormData {
            fields: MultiMap::new(),
            #[cfg(feature = "server")]
            files: MultiMap::new(),
        }
    }

    /// Parse MIME `multipart/*` information from a stream as a [`FormData`].
    pub(crate) async fn read(headers: &HeaderMap, body: ReqBody) -> Result<FormData, SilentError> {
        let mut form_data = FormData::new();
        if let Some(boundary) = headers
            .get(CONTENT_TYPE)
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| multer::parse_boundary(ct).ok())
        {
            let mut multipart = Multipart::new(body, boundary);
            while let Some(mut field) = multipart.next_field().await? {
                if let Some(name) = field.name().map(|s| s.to_owned()) {
                    if field.headers().get(CONTENT_TYPE).is_some() {
                        form_data
                            .files
                            .insert(name, FilePart::create(&mut field).await?);
                    } else {
                        form_data.fields.insert(name, field.text().await?);
                    }
                }
            }
        }
        Ok(form_data)
    }
}

impl Default for FormData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// A file that is to be inserted into a `multipart/*` or alternatively an uploaded file that
/// was received as part of `multipart/*` parsing.
#[derive(Clone, Debug)]
pub struct FilePart {
    name: Option<String>,
    /// The headers of the part
    headers: HeaderMap,
    /// A temporary file containing the file content
    path: PathBuf,
    /// Optionally, the size of the file.  This is filled when multiparts are parsed, but is
    /// not necessary when they are generated.
    size: u64,
    // The temporary directory the upload was put into, saved for the Drop trait
    temp_dir: Option<PathBuf>,
}

impl FilePart {
    /// Get file name.
    #[inline]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Get file name mutable reference.
    #[inline]
    pub fn name_mut(&mut self) -> Option<&mut String> {
        self.name.as_mut()
    }
    /// Get headers.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    /// Get headers mutable reference.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
    /// Get file path.
    #[inline]
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    /// Get file size.
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }
    /// If you do not want the file on disk to be deleted when Self drops, call this
    /// function.  It will become your responsibility to clean up.
    #[inline]
    pub fn do_not_delete_on_drop(&mut self) {
        self.temp_dir = None;
    }
    /// Save the file to a new location.
    #[inline]
    pub fn save(&self, path: String) -> Result<u64, SilentError> {
        std::fs::copy(self.path(), Path::new(&path)).map_err(|e| SilentError::BusinessError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("Failed to save file: {}", e),
        })
    }

    /// Create a new temporary FilePart (when created this way, the file will be
    /// deleted once the FilePart object goes out of scope).
    #[inline]
    pub async fn create(field: &mut Field<'_>) -> Result<FilePart, SilentError> {
        // Setup a file to capture the contents.
        let mut path = tokio::task::spawn_blocking(|| {
            Builder::new().prefix("silent_http_multipart").tempdir()
        })
        .await
        .expect("Runtime spawn blocking poll error")?
        .into_path();
        let temp_dir = Some(path.clone());
        let name = field.file_name().map(|s| s.to_owned());
        path.push(format!(
            "{}.{}",
            TextNonce::sized_urlsafe(32).unwrap().into_string(),
            name.as_deref()
                .and_then(|name| { Path::new(name).extension().and_then(OsStr::to_str) })
                .unwrap_or("unknown")
        ));
        let mut file = File::create(&path).await?;
        let mut size = 0;
        while let Some(chunk) = field.chunk().await? {
            size += chunk.len() as u64;
            file.write_all(&chunk).await?;
        }
        Ok(FilePart {
            name,
            headers: field.headers().to_owned(),
            path,
            size,
            temp_dir,
        })
    }
}

impl Drop for FilePart {
    fn drop(&mut self) {
        if let Some(temp_dir) = &self.temp_dir {
            let path = self.path.clone();
            let temp_dir = temp_dir.to_owned();
            tokio::task::spawn_blocking(move || {
                std::fs::remove_file(&path).ok();
                std::fs::remove_dir(temp_dir).ok();
            });
        }
    }
}
