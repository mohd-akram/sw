extern crate gcc;

fn main() {
    gcc::Build::new()
        .file("src/getppid.c")
        .compile("process");
}
