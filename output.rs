use knoten_core::natives::registry;

fn main() {
    println!("{}", String::from("--- JIT vs AOT Torture Test ---"));
    {
    let mut a = registry::registry_create_counter();
    let mut b = registry::registry_create_counter();
    let mut i = 0;
    while (i < 3) {
    let mut dummy = registry::registry_create_counter();
    registry::registry_release(dummy);
    dummy = 999;
    registry::registry_release(a);
    a = registry::registry_create_counter();
    i = (i + 1);
};
    registry::registry_release(b);
    registry::registry_release(a);
};
    println!("{}", String::from("Final Active Registry Nodes (Must be 0):"));
    println!("{}", registry::registry_dump());
}
