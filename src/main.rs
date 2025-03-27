use rbtreemap::RBTreeMap;

fn main() {
    println!("Hello, world!");
    let mut map: RBTreeMap<i32, &str> = RBTreeMap::new();
    map.insert(3, "a");
    map.insert(2, "b");
    map.insert(1, "c");
    for (k, v) in map.iter() {
        println!("{}: {}", k, v);
    }
}
