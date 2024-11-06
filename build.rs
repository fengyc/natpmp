fn main() {
    cc::Build::new()
        .file("src/getgateway.c")
        .include("src/")
        .compile("getgateway");
}
