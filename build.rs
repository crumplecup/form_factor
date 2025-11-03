fn main() {
    // Load .env file during build process
    // This is needed for build-time dependencies like opencv-sys and tesseract-sys
    // which require LIBCLANG_PATH to be set during compilation
    let _ = dotenvy::dotenv(); // Ignore error if .env doesn't exist

    // Cargo will automatically use the environment variables that are now loaded
    println!("cargo:rerun-if-changed=.env");
}
