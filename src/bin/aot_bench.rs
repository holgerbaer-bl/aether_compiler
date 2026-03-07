use knoten_core::natives::registry;

fn main() {
    println!(
        "{}",
        String::from("=== KnotenCore Chronos Benchmark (AOT) ===")
    );
    {
        let timer = registry::registry_now();
        let mut i: i64 = 0;
        let mut acc: i64 = 0;
        while i < 1_000_000 {
            acc = acc + i;
            i = i + 1;
        }
        let elapsed = registry::registry_elapsed_ms(timer);
        println!("{}", String::from("--- Result ---"));
        println!("{}", acc);
        println!("{}", String::from("Elapsed (ms):"));
        println!("{}", elapsed);
        registry::registry_release(timer);
    }
}
