fn main() {
    cc::Build::new()
        .file("highwayhash/c/highwayhash.c")
        .compile("libhighway");
}
