fn main() {
    // Include newly added migrations in the embedded SQLx migrator on stable Rust.
    println!("cargo:rerun-if-changed=migrations");
}
