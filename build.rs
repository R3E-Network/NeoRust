fn main() {
    // This build script helps with documentation generation on docs.rs
    
    // Set cfg flags for docs.rs
    if std::env::var("DOCS_RS").is_ok() {
        // This tells rustdoc to show documentation for all features
        println!("cargo:rustc-cfg=docsrs");
    }
} 