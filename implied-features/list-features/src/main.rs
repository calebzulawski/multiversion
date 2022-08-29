#![feature(stdsimd)]

extern crate std_detect;

fn main() {
    for (feature, _) in std_detect::detect::features() {
        println!("{}", feature)
    }
}
