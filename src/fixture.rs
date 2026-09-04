//! Test fixture loader for E2E tests
//!
//! Provides utilities for loading and managing test fixtures from the fixtures directory.

use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Fixture category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureCategory {
    /// Rust project fixtures
    Rust,
    /// Python project fixtures
    Python,
    /// Java project fixtures
    Java,
    /// TypeScript project fixtures
    TypeScript,
    /// C# project fixtures
    CSharp,
    /// C++ project fixtures
    Cpp,
    /// Scala project fixtures
    Scala,
    /// Dart project fixtures
    Dart,
    /// Go project fixtures
    Go,
    /// Kotlin project fixtures
    Kotlin,
    /// PHP project fixtures
    Php,
    /// Ruby project fixtures
    Ruby,
    /// JavaScript project fixtures
    JavaScript,
    /// Multi-language project fixtures
    MultiLanguage,
    /// Document fixtures (markdown, plain text, logs, config files)
    Documents,
}

impl FixtureCategory {
    /// Get directory name for this category
    pub fn dir_name(&self) -> &'static str {
        match self {
            FixtureCategory::Rust => "rust",
            FixtureCategory::Python => "python",
            FixtureCategory::Java => "java",
            FixtureCategory::TypeScript => "typescript",
            FixtureCategory::CSharp => "csharp",
            FixtureCategory::Cpp => "cpp",
            FixtureCategory::Scala => "scala",
            FixtureCategory::Dart => "dart",
            FixtureCategory::Go => "go",
            FixtureCategory::Kotlin => "kotlin",
            FixtureCategory::Php => "php",
            FixtureCategory::Ruby => "ruby",
            FixtureCategory::JavaScript => "javascript",
            FixtureCategory::MultiLanguage => "multi_language",
            FixtureCategory::Documents => "documents",
        }
    }
}

/// Fixture specification
#[derive(Debug, Clone)]
pub struct FixtureSpec {
    /// Category of the fixture
    pub category: FixtureCategory,
    /// Subdirectory name (e.g., "basic", "edge_cases")
    pub subdirectory: String,
}

impl FixtureSpec {
    /// Create a new fixture specification
    pub fn new(category: FixtureCategory, subdirectory: impl Into<String>) -> Self {
        Self {
            category,
            subdirectory: subdirectory.into(),
        }
    }

    /// Rust basic project fixture
    pub fn rust_basic() -> Self {
        Self::new(FixtureCategory::Rust, "basic")
    }

    /// Rust edge cases fixture
    pub fn rust_edge_cases() -> Self {
        Self::new(FixtureCategory::Rust, "edge_cases")
    }

    /// Rust review fixture group
    pub fn rust_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Rust, format!("review/{}", name.into()))
    }

    /// Rust once_cell review fixture
    pub fn rust_once_cell() -> Self {
        Self::rust_review("once_cell")
    }

    /// Rust distractor fixture (unrelated code for negative samples)
    pub fn rust_distractor() -> Self {
        Self::new(FixtureCategory::Rust, "distractor")
    }

    /// Rust index_sidecar review fixture
    pub fn rust_index_sidecar() -> Self {
        Self::rust_review("index_sidecar")
    }

    /// Rust relation_demo review fixture
    pub fn rust_relation_demo() -> Self {
        Self::rust_review("relation_demo")
    }

    /// Rust relation_diamond review fixture
    pub fn rust_relation_diamond() -> Self {
        Self::rust_review("relation_diamond")
    }

    /// Rust ripgrep review fixture
    pub fn rust_ripgrep() -> Self {
        Self::rust_review("ripgrep")
    }

    /// Rust type inference generics fixture
    pub fn rust_type_inference_generics() -> Self {
        Self::new(FixtureCategory::Rust, "type_inference/generics")
    }

    /// Rust type inference control flow fixture
    pub fn rust_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Rust, "type_inference/control_flow")
    }

    /// Python basic project fixture
    pub fn python_basic() -> Self {
        Self::new(FixtureCategory::Python, "basic")
    }

    /// Python type inference type hints fixture
    pub fn python_type_inference_type_hints() -> Self {
        Self::new(FixtureCategory::Python, "type_inference/type_hints")
    }

    /// Python type inference control flow fixture
    pub fn python_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Python, "type_inference/control_flow")
    }

    /// Python type inference cross-file fixture
    pub fn python_type_inference_cross_file() -> Self {
        Self::new(FixtureCategory::Python, "type_inference/cross_file")
    }

    /// Java basic project fixture
    pub fn java_basic() -> Self {
        Self::new(FixtureCategory::Java, "basic")
    }

    /// Java type inference generics fixture
    pub fn java_type_inference_generics() -> Self {
        Self::new(FixtureCategory::Java, "type_inference/generics")
    }

    /// Java type inference control flow fixture
    pub fn java_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Java, "type_inference/control_flow")
    }

    /// Java Spring Boot minimal demo fixture
    pub fn java_spring_boot() -> Self {
        Self::java_review("springboot-minimal-demo")
    }

    /// Java jackson-core review fixture
    pub fn java_jackson_core() -> Self {
        Self::java_review("jackson-core")
    }

    /// Java review fixture group
    pub fn java_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Java, format!("review/{}", name.into()))
    }

    /// Java index_sidecar review fixture
    pub fn java_index_sidecar() -> Self {
        Self::java_review("index_sidecar")
    }

    /// Python review fixture group
    pub fn python_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Python, format!("review/{}", name.into()))
    }

    /// Python index_sidecar review fixture
    pub fn python_index_sidecar() -> Self {
        Self::python_review("index_sidecar")
    }

    /// Python Flask review fixture
    pub fn python_flask() -> Self {
        Self::python_review("flask")
    }

    /// TypeScript review fixture group
    pub fn typescript_review(name: impl Into<String>) -> Self {
        Self::new(
            FixtureCategory::TypeScript,
            format!("review/{}", name.into()),
        )
    }

    /// TypeScript index_sidecar review fixture
    pub fn typescript_index_sidecar() -> Self {
        Self::typescript_review("index_sidecar")
    }

    /// TypeScript type inference generics fixture
    pub fn typescript_type_inference_generics() -> Self {
        Self::new(FixtureCategory::TypeScript, "type_inference/generics")
    }

    /// TypeScript type inference unions fixture
    pub fn typescript_type_inference_unions() -> Self {
        Self::new(FixtureCategory::TypeScript, "type_inference/unions")
    }

    /// Multi-language project fixture
    pub fn multi_language() -> Self {
        Self::new(FixtureCategory::MultiLanguage, "")
    }

    /// C# basic project fixture
    pub fn csharp_basic() -> Self {
        Self::new(FixtureCategory::CSharp, "basic")
    }

    /// C# type inference generics fixture
    pub fn csharp_type_inference_generics() -> Self {
        Self::new(FixtureCategory::CSharp, "type_inference/generics")
    }

    /// C# type inference control flow fixture
    pub fn csharp_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::CSharp, "type_inference/control_flow")
    }

    /// C# review fixture group
    pub fn csharp_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::CSharp, format!("review/{}", name.into()))
    }

    /// C# MediatR review fixture
    pub fn csharp_mediatr() -> Self {
        Self::csharp_review("MediatR")
    }

    /// Go basic project fixture
    pub fn go_basic() -> Self {
        Self::new(FixtureCategory::Go, "basic")
    }

    /// Go type inference interfaces fixture
    pub fn go_type_inference_interfaces() -> Self {
        Self::new(FixtureCategory::Go, "type_inference/interfaces")
    }

    /// Go type inference control flow fixture
    pub fn go_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Go, "type_inference/control_flow")
    }

    /// Go review fixture group
    pub fn go_review(name: impl Into<String>) -> Self {
        Self::new(FixtureCategory::Go, format!("review/{}", name.into()))
    }

    /// Go gin review fixture
    pub fn go_gin() -> Self {
        Self::go_review("gin")
    }

    /// Kotlin basic project fixture
    pub fn kotlin_basic() -> Self {
        Self::new(FixtureCategory::Kotlin, "basic")
    }

    /// PHP basic project fixture
    pub fn php_basic() -> Self {
        Self::new(FixtureCategory::Php, "basic")
    }

    /// Ruby basic project fixture
    pub fn ruby_basic() -> Self {
        Self::new(FixtureCategory::Ruby, "basic")
    }

    /// JavaScript review fixture group
    pub fn javascript_review(name: impl Into<String>) -> Self {
        Self::new(
            FixtureCategory::JavaScript,
            format!("review/{}", name.into()),
        )
    }

    /// JavaScript Express review fixture
    pub fn javascript_express() -> Self {
        Self::javascript_review("express")
    }

    /// Document fixture group (markdown / plain text / logs / config files)
    pub fn documents() -> Self {
        Self::new(FixtureCategory::Documents, "")
    }

    /// C++ type inference declarations fixture
    pub fn cpp_type_inference_declarations() -> Self {
        Self::new(FixtureCategory::Cpp, "type_inference/declarations")
    }

    /// Kotlin type inference generics fixture
    pub fn kotlin_type_inference_generics() -> Self {
        Self::new(FixtureCategory::Kotlin, "type_inference/generics")
    }

    /// Kotlin type inference control flow fixture
    pub fn kotlin_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Kotlin, "type_inference/control_flow")
    }

    /// Scala type inference declarations fixture
    pub fn scala_type_inference_declarations() -> Self {
        Self::new(FixtureCategory::Scala, "type_inference/declarations")
    }

    /// Scala type inference control flow fixture
    pub fn scala_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Scala, "type_inference/control_flow")
    }

    /// Ruby type inference constructors fixture
    pub fn ruby_type_inference_constructors() -> Self {
        Self::new(FixtureCategory::Ruby, "type_inference/constructors")
    }

    /// PHP type inference phpdoc fixture
    pub fn php_type_inference_phpdoc() -> Self {
        Self::new(FixtureCategory::Php, "type_inference/phpdoc")
    }

    /// Dart type inference declarations fixture
    pub fn dart_type_inference_declarations() -> Self {
        Self::new(FixtureCategory::Dart, "type_inference/declarations")
    }

    /// Dart type inference control flow fixture
    pub fn dart_type_inference_control_flow() -> Self {
        Self::new(FixtureCategory::Dart, "type_inference/control_flow")
    }

    /// JavaScript type inference narrowing fixture
    pub fn javascript_type_inference_narrowing() -> Self {
        Self::new(FixtureCategory::JavaScript, "type_inference/narrowing")
    }
}

/// Common fixture capabilities used by workflow helpers.
pub trait FixtureAccess {
    /// Get the root path of the fixture.
    fn root(&self) -> &Path;

    /// Get the root path of the fixture.
    fn root_path(&self) -> &Path {
        self.root()
    }

    /// Get a file path relative to the fixture root.
    fn file<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf;

    /// Add a new file to the fixture.
    fn add_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf>;

    /// Modify an existing file in the fixture.
    fn modify_file<P: AsRef<Path>, C: AsRef<[u8]>>(
        &self,
        relative_path: P,
        content: C,
    ) -> io::Result<PathBuf>;

    /// Delete a file from the fixture.
    fn delete_file<P: AsRef<Path>>(&self, relative_path: P) -> io::Result<()>;
}

/// Test fixture manager
///
/// Loads fixtures from the crate-root fixtures directory and copies them to a temporary directory.
pub struct TestFixture {
    /// Temporary directory where fixture is copied
    temp_dir: TempDir,
}

impl FixtureAccess for TestFixture {
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

impl TestFixture {
    /// Get the base fixtures directory
    fn fixtures_base() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    /// Load a fixture by specification
    pub fn load(spec: FixtureSpec) -> io::Result<Self> {
        let base = Self::fixtures_base();
        let mut fixture_path = base.join(spec.category.dir_name());

        if !spec.subdirectory.is_empty() {
            fixture_path = fixture_path.join(&spec.subdirectory);
        }

        // Verify fixture exists
        if !fixture_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Fixture not found: {:?}", fixture_path),
            ));
        }

        // Create temp directory
        let temp_dir = TempDir::new()?;

        // Copy fixture to temp directory
        Self::copy_dir_all(&fixture_path, temp_dir.path())?;

        Ok(Self { temp_dir })
    }

    /// Load Rust basic project fixture
    pub fn rust_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_basic())
    }

    /// Load Rust edge cases fixture
    pub fn rust_edge_cases() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_edge_cases())
    }

    /// Load Rust review fixture
    pub fn rust_review(name: impl Into<String>) -> io::Result<Self> {
        Self::load(FixtureSpec::rust_review(name))
    }

    /// Load Rust once_cell review fixture
    pub fn rust_once_cell() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_once_cell())
    }

    /// Load Rust distractor fixture
    pub fn rust_distractor() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_distractor())
    }

    /// Load Rust index_sidecar review fixture
    pub fn rust_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_index_sidecar())
    }

    /// Load Rust relation_demo review fixture
    pub fn rust_relation_demo() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_relation_demo())
    }

    /// Load Rust relation_diamond review fixture
    pub fn rust_relation_diamond() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_relation_diamond())
    }

    /// Load Rust ripgrep review fixture
    pub fn rust_ripgrep() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_ripgrep())
    }

    /// Load Rust type inference generics fixture
    pub fn rust_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_type_inference_generics())
    }

    /// Load Rust type inference control flow fixture
    pub fn rust_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::rust_type_inference_control_flow())
    }

    /// Load Python basic project fixture
    pub fn python_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::python_basic())
    }

    /// Load Python type inference type hints fixture
    pub fn python_type_inference_type_hints() -> io::Result<Self> {
        Self::load(FixtureSpec::python_type_inference_type_hints())
    }

    /// Load Python type inference control flow fixture
    pub fn python_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::python_type_inference_control_flow())
    }

    /// Load Python type inference cross-file fixture
    pub fn python_type_inference_cross_file() -> io::Result<Self> {
        Self::load(FixtureSpec::python_type_inference_cross_file())
    }

    /// Load Java basic project fixture
    pub fn java_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::java_basic())
    }

    /// Load Java type inference generics fixture
    pub fn java_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::java_type_inference_generics())
    }

    /// Load Java type inference control flow fixture
    pub fn java_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::java_type_inference_control_flow())
    }

    /// Load Java Spring Boot minimal demo fixture
    pub fn java_spring_boot() -> io::Result<Self> {
        Self::load(FixtureSpec::java_spring_boot())
    }

    /// Load Java index_sidecar review fixture
    pub fn java_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::java_index_sidecar())
    }

    /// Load Java jackson-core review fixture
    pub fn java_jackson_core() -> io::Result<Self> {
        Self::load(FixtureSpec::java_jackson_core())
    }

    /// Load Python index_sidecar review fixture
    pub fn python_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::python_index_sidecar())
    }

    /// Load TypeScript index_sidecar review fixture
    pub fn typescript_index_sidecar() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_index_sidecar())
    }

    /// Load TypeScript type inference generics fixture
    pub fn typescript_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_type_inference_generics())
    }

    /// Load TypeScript type inference unions fixture
    pub fn typescript_type_inference_unions() -> io::Result<Self> {
        Self::load(FixtureSpec::typescript_type_inference_unions())
    }

    /// Load C# basic project fixture
    pub fn csharp_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_basic())
    }

    /// Load C# type inference generics fixture
    pub fn csharp_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_type_inference_generics())
    }

    /// Load C# type inference control flow fixture
    pub fn csharp_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_type_inference_control_flow())
    }

    /// Load C# MediatR review fixture
    pub fn csharp_mediatr() -> io::Result<Self> {
        Self::load(FixtureSpec::csharp_mediatr())
    }

    /// Load Go basic project fixture
    pub fn go_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::go_basic())
    }

    /// Load Go type inference interfaces fixture
    pub fn go_type_inference_interfaces() -> io::Result<Self> {
        Self::load(FixtureSpec::go_type_inference_interfaces())
    }

    /// Load Go type inference control flow fixture
    pub fn go_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::go_type_inference_control_flow())
    }

    /// Load Go gin review fixture
    pub fn go_gin() -> io::Result<Self> {
        Self::load(FixtureSpec::go_gin())
    }

    /// Load Kotlin basic project fixture
    pub fn kotlin_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::kotlin_basic())
    }

    /// Load PHP basic project fixture
    pub fn php_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::php_basic())
    }

    /// Load Ruby basic project fixture
    pub fn ruby_basic() -> io::Result<Self> {
        Self::load(FixtureSpec::ruby_basic())
    }

    /// Load JavaScript Express review fixture
    pub fn javascript_express() -> io::Result<Self> {
        Self::load(FixtureSpec::javascript_express())
    }

    /// Load multi-language project fixture
    pub fn multi_language() -> io::Result<Self> {
        Self::load(FixtureSpec::multi_language())
    }

    /// Load C++ type inference declarations fixture
    pub fn cpp_type_inference_declarations() -> io::Result<Self> {
        Self::load(FixtureSpec::cpp_type_inference_declarations())
    }

    /// Load Kotlin type inference generics fixture
    pub fn kotlin_type_inference_generics() -> io::Result<Self> {
        Self::load(FixtureSpec::kotlin_type_inference_generics())
    }

    /// Load Kotlin type inference control flow fixture
    pub fn kotlin_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::kotlin_type_inference_control_flow())
    }

    /// Load Scala type inference declarations fixture
    pub fn scala_type_inference_declarations() -> io::Result<Self> {
        Self::load(FixtureSpec::scala_type_inference_declarations())
    }

    /// Load Scala type inference control flow fixture
    pub fn scala_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::scala_type_inference_control_flow())
    }

    /// Load Ruby type inference constructors fixture
    pub fn ruby_type_inference_constructors() -> io::Result<Self> {
        Self::load(FixtureSpec::ruby_type_inference_constructors())
    }

    /// Load PHP type inference phpdoc fixture
    pub fn php_type_inference_phpdoc() -> io::Result<Self> {
        Self::load(FixtureSpec::php_type_inference_phpdoc())
    }

    /// Load Dart type inference declarations fixture
    pub fn dart_type_inference_declarations() -> io::Result<Self> {
        Self::load(FixtureSpec::dart_type_inference_declarations())
    }

    /// Load Dart type inference control flow fixture
    pub fn dart_type_inference_control_flow() -> io::Result<Self> {
        Self::load(FixtureSpec::dart_type_inference_control_flow())
    }

    /// Load JavaScript type inference narrowing fixture
    pub fn javascript_type_inference_narrowing() -> io::Result<Self> {
        Self::load(FixtureSpec::javascript_type_inference_narrowing())
    }

    /// Load the document fixture group
    pub fn documents() -> io::Result<Self> {
        Self::load(FixtureSpec::documents())
    }

    /// Get the root path of the fixture (in temp directory)
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get the root path of the fixture (alias for root())
    pub fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Get a file path relative to the fixture root
    pub fn file(&self, relative_path: impl AsRef<Path>) -> PathBuf {
        self.temp_dir.path().join(relative_path.as_ref())
    }

    /// Add a new file to the fixture
    pub fn add_file(
        &self,
        relative_path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.file(relative_path.as_ref());

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Modify an existing file in the fixture
    pub fn modify_file(
        &self,
        relative_path: impl AsRef<Path>,
        content: impl AsRef<[u8]>,
    ) -> io::Result<PathBuf> {
        let path = self.file(relative_path.as_ref());
        std::fs::write(&path, content.as_ref())?;
        Ok(path)
    }

    /// Delete a file from the fixture
    pub fn delete_file(&self, relative_path: impl AsRef<Path>) -> io::Result<()> {
        let path = self.file(relative_path.as_ref());
        std::fs::remove_file(&path)
    }

    /// List all files in the fixture
    pub fn list_files(&self) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        Self::collect_files(self.temp_dir.path(), &mut files)?;
        Ok(files)
    }

    /// Copy directory recursively
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

    /// Collect all files recursively
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

/// Empty fixture for creating test projects from scratch
pub struct EmptyFixture {
    /// Temporary directory
    temp_dir: TempDir,
}

impl EmptyFixture {
    /// Create an empty fixture
    pub fn new() -> io::Result<Self> {
        let temp_dir = TempDir::new()?;
        Ok(Self { temp_dir })
    }

    /// Get the root path
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Add a file
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

    /// Convert to TestFixture (for compatibility)
    pub fn into_test_fixture(self) -> TestFixture {
        TestFixture {
            temp_dir: self.temp_dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rust_basic() {
        let fixture = TestFixture::rust_basic().expect("Failed to load fixture");
        assert!(fixture.root().exists());
        assert!(fixture.file("src/main.rs").exists());
        assert!(fixture.file("src/lib.rs").exists());
        assert!(fixture.file("Cargo.toml").exists());
    }

    #[test]
    fn test_list_files() {
        let fixture = TestFixture::rust_basic().expect("Failed to load fixture");
        let files = fixture.list_files().expect("Failed to list files");
        assert!(!files.is_empty());
    }

    #[test]
    fn test_add_file() {
        let fixture = TestFixture::rust_basic().expect("Failed to load fixture");
        let path = fixture
            .add_file("src/new.rs", "fn new() {}")
            .expect("Failed to add file");
        assert!(path.exists());
    }

    #[test]
    fn test_empty_fixture() {
        let fixture = EmptyFixture::new().expect("Failed to create empty fixture");
        let path = fixture
            .add_file("test.rs", "fn test() {}")
            .expect("Failed to add file");
        assert!(path.exists());
    }
}
