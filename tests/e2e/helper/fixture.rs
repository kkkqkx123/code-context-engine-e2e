//! Test fixture loader for integration tests.
//!
//! Integration tests use fixtures under `tests/fixtures/` so they stay
//! separate from the library crate fixtures used by `src/` tests.

use std::io;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

/// Fixture category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FixtureCategory {
    /// Rust project fixtures.
    Rust,
    /// Python project fixtures.
    Python,
    /// Java project fixtures.
    Java,
    /// TypeScript project fixtures.
    TypeScript,
    /// C# project fixtures.
    CSharp,
    /// Go project fixtures.
    Go,
    /// Multi-language project fixtures.
    MultiLanguage,
    /// Document fixtures (markdown, plain text, logs, config files).
    Documents,
}

impl FixtureCategory {
    /// Get the directory name for this category.
    pub fn dir_name(&self) -> &'static str {
        match self {
            FixtureCategory::Rust => "rust",
            FixtureCategory::Python => "python",
            FixtureCategory::Java => "java",
            FixtureCategory::TypeScript => "typescript",
            FixtureCategory::CSharp => "csharp",
            FixtureCategory::Go => "go",
            FixtureCategory::MultiLanguage => "multi_language",
            FixtureCategory::Documents => "documents",
        }
    }
}

/// Fixture specification.
#[derive(Debug, Clone)]
pub struct FixtureSpec {
    /// Fixture category.
    pub category: FixtureCategory,
    /// Subdirectory name, for example `basic` or `review/once_cell`.
    pub subdirectory: String,
}

impl FixtureSpec {
    /// Create a new fixture specification.
    #[allow(dead_code)]
    pub fn new(category: FixtureCategory, subdirectory: impl Into<String>) -> Self {
        Self {
            category,
            subdirectory: subdirectory.into(),
        }
    }

    /// Rust basic project fixture.
    #[allow(dead_code)]
    pub fn rust_basic() -> Self {
        Self::new(FixtureCategory::Rust, "basic")
    }

    /// Rust edge cases fixture.
    pub fn rust_edge_cases() -> Self {
        Self::new(FixtureCategory::Rust, "edge_cases")
    }

    /// Rust review fixture group.
    pub fn rust_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Rust, format!("review/{}", name.into()))
    }

    /// Rust once_cell review fixture.
    pub fn rust_once_cell() -> Self {
        Self::rust_review("once_cell")
    }

    /// Rust index_sidecar review fixture.
    pub fn rust_index_sidecar() -> Self {
        Self::rust_review("index_sidecar")
    }

    /// Rust relation_demo review fixture.
    pub fn rust_relation_demo() -> Self {
        Self::rust_review("relation_demo")
    }

    /// Rust relation_diamond review fixture.
    pub fn rust_relation_diamond() -> Self {
        Self::rust_review("relation_diamond")
    }

    /// Rust type inference generics fixture.
    pub fn rust_type_inference_generics() -> Self {
        Self::new(FixtureCategory::Rust, "type_inference/generics")
    }

    /// Rust type inference control flow fixture.
    pub fn rust_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Rust, "type_inference/control_flow")
    }

    /// Python basic project fixture.
    pub fn python_basic() -> Self {
        Self::new(FixtureCategory::Python, "basic")
    }

    /// Python type inference type hints fixture.
    pub fn python_type_inference_type_hints() -> Self {
        Self::new(FixtureCategory::Python, "type_inference/type_hints")
    }

    /// Python type inference control flow fixture.
    pub fn python_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Python, "type_inference/control_flow")
    }

    /// Java basic project fixture.
    pub fn java_basic() -> Self {
        Self::new(FixtureCategory::Java, "basic")
    }

    /// Java type inference generics fixture.
    pub fn java_type_inference_generics() -> Self {
        Self::new(FixtureCategory::Java, "type_inference/generics")
    }

    /// Java type inference control flow fixture.
    pub fn java_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Java, "type_inference/control_flow")
    }

    /// Java review fixture group.
    pub fn java_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Java, format!("review/{}", name.into()))
    }

    /// Java index_sidecar review fixture.
    pub fn java_index_sidecar() -> Self {
        Self::java_review("index_sidecar")
    }

    /// Python review fixture group.
    pub fn python_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Python, format!("review/{}", name.into()))
    }

    /// Python index_sidecar review fixture.
    pub fn python_index_sidecar() -> Self {
        Self::python_review("index_sidecar")
    }

    /// TypeScript review fixture group.
    pub fn typescript_review(name: impl Into<String>) -> Self {
        Self::new(
            FixtureCategory::TypeScript,
            format!("review/{}", name.into()),
        )
    }

    /// TypeScript index_sidecar review fixture.
    pub fn typescript_index_sidecar() -> Self {
        Self::typescript_review("index_sidecar")
    }

    /// TypeScript type inference generics fixture.
    pub fn typescript_type_inference_generics() -> Self {
        Self::new(FixtureCategory::TypeScript, "type_inference/generics")
    }

    /// TypeScript type inference unions fixture.
    pub fn typescript_type_inference_unions() -> Self {
        Self::new(FixtureCategory::TypeScript, "type_inference/unions")
    }

    /// C# basic project fixture.
    pub fn csharp_basic() -> Self {
        Self::new(FixtureCategory::CSharp, "basic")
    }

    /// C# type inference generics fixture.
    pub fn csharp_type_inference_generics() -> Self {
        Self::new(FixtureCategory::CSharp, "type_inference/generics")
    }

    /// C# type inference control flow fixture.
    pub fn csharp_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::CSharp, "type_inference/control_flow")
    }

    /// Go basic project fixture.
    pub fn go_basic() -> Self {
        Self::new(FixtureCategory::Go, "basic")
    }

    /// Go type inference interfaces fixture.
    pub fn go_type_inference_interfaces() -> Self {
        Self::new(FixtureCategory::Go, "type_inference/interfaces")
    }

    /// Go type inference control flow fixture.
    pub fn go_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Go, "type_inference/control_flow")
    }

    /// Multi-language project fixture.
    pub fn multi_language() -> Self {
        Self::new(FixtureCategory::MultiLanguage, "")
    }

    /// Document fixture group.
    pub fn documents() -> Self {
        Self::new(FixtureCategory::Documents, "")
    }
}

/// Test fixture manager.
pub struct TestFixture {
    temp_dir: TempDir,
}

#[allow(dead_code)]
impl TestFixture {
    /// Get the base fixtures directory for integration tests.
    fn fixtures_base() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
    }

    /// Load a fixture by specification.
    pub fn load(spec: FixtureSpec) -> io::Result<Self> {
        let base = Self::fixtures_base();
        let mut fixture_path = base.join(spec.category.dir_name());

        if !spec.subdirectory.is_empty() {
            fixture_path = fixture_path.join(&spec.subdirectory);
        }

        if !fixture_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Fixture not found: {:?}", fixture_path),
            ));
        }

        let temp_dir = TempDir::new()?;
        Self::copy_dir_all(&fixture_path, temp_dir.path())?;
        Ok(Self { temp_dir })
    }

    /// Load Rust basic project fixture.
    pub fn rust_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_basic())
    }

    /// Load Rust edge cases fixture.
    pub fn rust_edge_cases() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_edge_cases())
    }

    /// Load Rust review fixture.
    pub fn rust_review(name: impl Into<String>) -> io::Result<Self> {
        Self::load(FixtureSpec::rust_review(name))
    }

    /// Load Rust once_cell review fixture.
    pub fn rust_once_cell() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_once_cell())
    }

    /// Load Rust index_sidecar review fixture.
    pub fn rust_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_index_sidecar())
    }

    /// Load Rust relation_demo review fixture.
    pub fn rust_relation_demo() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_relation_demo())
    }

    /// Load Rust relation_diamond review fixture.
    pub fn rust_relation_diamond() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_relation_diamond())
    }

    /// Load Rust type inference generics fixture.
    pub fn rust_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_type_inference_generics())
    }

    /// Load Rust type inference control flow fixture.
    pub fn rust_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_type_inference_control_flow())
    }

    /// Load Python basic project fixture.
    pub fn python_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::python_basic())
    }

    /// Load Python type inference type hints fixture.
    pub fn python_type_inference_type_hints() -> io::Result<Self> {
        Self::load(FixtureSpec::python_type_inference_type_hints())
    }

    /// Load Python type inference control flow fixture.
    pub fn python_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::python_type_inference_control_flow())
    }

    /// Load Java basic project fixture.
    pub fn java_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::java_basic())
    }

    /// Load Java type inference generics fixture.
    pub fn java_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::java_type_inference_generics())
    }

    /// Load Java type inference control flow fixture.
    pub fn java_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::java_type_inference_control_flow())
    }

    /// Load Java index_sidecar review fixture.
    pub fn java_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::java_index_sidecar())
    }

    /// Load Python index_sidecar review fixture.
    pub fn python_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::python_index_sidecar())
    }

    /// Load TypeScript index_sidecar review fixture.
    pub fn typescript_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_index_sidecar())
    }

    /// Load TypeScript type inference generics fixture.
    pub fn typescript_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_type_inference_generics())
    }

    /// Load TypeScript type inference unions fixture.
    pub fn typescript_type_inference_unions() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_type_inference_unions())
    }

    /// Load C# basic project fixture.
    pub fn csharp_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_basic())
    }

    /// Load C# type inference generics fixture.
    pub fn csharp_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_type_inference_generics())
    }

    /// Load C# type inference control flow fixture.
    pub fn csharp_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_type_inference_control_flow())
    }

    /// Load Go basic project fixture.
    pub fn go_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::go_basic())
    }

    /// Load Go type inference interfaces fixture.
    pub fn go_type_inference_interfaces() -> io::Result<Self> {
        Self::load(FixtureSpec::go_type_inference_interfaces())
    }

    /// Load Go type inference control flow fixture.
    pub fn go_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::go_type_inference_control_flow())
    }

    /// Load multi-language project fixture.
    pub fn multi_language() -> io::Result<Self> {
        Self::load(FixtureSpec::multi_language())
    }

    /// Load the document fixture group.
    pub fn documents() -> io::Result<Self> {
        Self::load(FixtureSpec::documents())
    }

    /// Get the root path of the fixture.
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get the root path of the fixture.
    pub fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get a file path relative to the fixture root.
    pub fn file(&self, relative_path: impl AsRef<Path>) -> PathBuf {
        self.temp_dir.path().join(relative_path.as_ref())
    }

    /// Add a new file to the fixture.
    pub fn add_file(
        &self,
        relative_path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.file(relative_path.as_ref());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Modify an existing file in the fixture.
    pub fn modify_file(
        &self,
        relative_path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.file(relative_path.as_ref());
        std::fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Delete a file from the fixture.
    pub fn delete_file(&self, relative_path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.file(relative_path.as_ref());
        std::fs::remove_file(&path)
    }

    /// List all files in the fixture.
    pub fn list_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        Self::collect_files(self.temp_dir.path(), &mut files)?;
        Ok(files)
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
        std::fs::create_dir_all(dst)?;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                Self::copy_dir_all(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let path = entry.path();

            if ty.is_dir() {
                Self::collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }
}

/// Empty fixture for creating test projects from scratch.
#[allow(dead_code)]
pub struct EmptyFixture {
    temp_dir: TempDir,
}

impl EmptyFixture {
    /// Create an empty fixture.
    #[allow(dead_code)]
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    /// Get the root path.
    #[allow(dead_code)]
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Add a file.
    #[allow(dead_code)]
    pub fn add_file(
        &self,
        relative_path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.temp_dir.path().join(relative_path.as_ref());

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Convert to `TestFixture`.
    #[allow(dead_code)]
    pub fn into_test_fixture(self) -> TestFixture {
        TestFixture {
            temp_dir: self.temp_dir,
        }
    }
}

impl cce_e2e_tests::fixture::FixtureAccess for TestFixture {
    fn root(&self) -> &Path {
        TestFixture::root(self)
    }

    fn file<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        TestFixture::file(self, relative_path)
    }

    fn add_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf> {
        TestFixture::add_file(self, relative_path, content)
    }

    fn modify_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf> {
        TestFixture::modify_file(self, relative_path, content)
    }

    fn delete_file<P: AsRef<Path>>(&self, relative_path: P) -> io::Result<()> {
        TestFixture::delete_file(self, relative_path)
    }
}
