extern crate cc;

fn main() {
    cc::Build::new().file("src/getppid.c").compile("process");
}
