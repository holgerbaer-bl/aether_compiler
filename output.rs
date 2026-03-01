use knoten_core::natives::registry;

fn main() {
    println!("{}", String::from("Creating handle"));
    {
    let mut counter_id = registry::registry_create_counter();
    registry::registry_increment(counter_id);
    registry::registry_increment(counter_id);
    registry::registry_increment(counter_id);
    println!("{}", registry::registry_get_value(counter_id));
    registry::registry_release(counter_id);
};
    println!("{}", String::from("Handle should be dropped!"));
}
