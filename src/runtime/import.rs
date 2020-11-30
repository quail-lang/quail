use std::path::{Path, PathBuf};
use super::runtime::RuntimeError;
use std::io;

pub trait ImportResolver {
    fn resolve(&mut self, import_name: &str) -> Result<ResolvedImport, RuntimeError>;
}

pub struct ResolvedImport {
    pub reader: Box<dyn io::Read>,
    pub source: String,
}

pub struct FileImportResolver(PathBuf);

impl FileImportResolver {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().canonicalize().unwrap();
        FileImportResolver(path)
    }
}

impl ImportResolver for FileImportResolver {
    fn resolve(&mut self, import_name: &str) -> Result<ResolvedImport, RuntimeError> {
        let FileImportResolver(base_dir) = self;
        let filename = format!("{}.ql", import_name);
        let filepath = base_dir.join(&filename).to_owned();
        let module_text = std::fs::read_to_string(&filepath)?;

        Ok(ResolvedImport {
            reader: Box::new(io::Cursor::new(module_text)),
            source: filename,
        })
    }
}
