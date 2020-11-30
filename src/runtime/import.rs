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

pub struct FilePathImportResolver;

impl ImportResolver for FilePathImportResolver {
    fn resolve(&mut self, filepath: &str) -> Result<ResolvedImport, RuntimeError> {
        let module_text = std::fs::read_to_string(&filepath)?;

        Ok(ResolvedImport {
            reader: Box::new(io::Cursor::new(module_text)),
            source: filepath.to_owned(),
        })
    }
}

pub struct ChainedImportResolver(Box<dyn ImportResolver>, Box<dyn ImportResolver>);

impl ChainedImportResolver {
    pub fn new(a: Box<dyn ImportResolver>, b: Box<dyn ImportResolver>) -> Self {
        ChainedImportResolver(a, b)
    }
}

impl ImportResolver for ChainedImportResolver {
    fn resolve(&mut self, import_name: &str) -> Result<ResolvedImport, RuntimeError> {
        let ChainedImportResolver(ira, irb) = self;
        if let Ok(ri) = ira.resolve(import_name) {
            Ok(ri)
        } else {
            irb.resolve(import_name)
        }
    }
}
