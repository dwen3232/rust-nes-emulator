mod ram;

fn main() {
    println!("Hello, world!");
    println!("{}", 0x10000 - 0x0800 - 0x0800)
}
