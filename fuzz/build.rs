fn main() {
    cc::Build::new()
        .include("highwayhash")
        .file("highwayhash/c/highwayhash.c")
        .compile("libhighway");
}
