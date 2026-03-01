use knoten_core::natives::registry;

fn main() {
    println!("{}", String::from("--- KnotenCore OS File Execution ---"));
    {
    let mut my_file = registry::registry_file_create(String::from("knoten_test.txt"));
    registry::registry_file_write(my_file, String::from("KnotenCore AOT/JIT Persistence Success!"));
    println!("{}", String::from("File explicitly written into Rust OS wrapper block bounds natively."));
    registry::registry_release(my_file);
};
    println!("{}", registry::registry_dump());
}
