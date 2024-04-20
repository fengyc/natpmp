fn main() {
    cc::Build::new()
        .file("src/getgateway.c")
        .file("src/const.c")
        .include("src/")
        .compile("getgateway");
}
