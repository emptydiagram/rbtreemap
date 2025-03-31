use super::*;

#[test]
fn test_singleton() {
    let mut map = RBTreeMap::new();
    assert_eq!(map.len(), 0);
    map.insert(12, "abc");
    assert_eq!(map.len(), 1);
    assert!(map.get(&12).is_some());
    assert_eq!(map.get(&12), Some(&"abc"));
}

#[test]
fn test_two_elements() {
    let mut map = RBTreeMap::new();
    map.insert(12, "abc");
    map.insert(34, "def");
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&12), Some(&"abc"));
    assert_eq!(map.get(&34), Some(&"def"));
}

#[test]
fn test_three_elements_1() {
    let mut map = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_three_elements_2() {
    let mut map = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(3, "c");
    map.insert(2, "b");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_three_elements_3() {
    let mut map = RBTreeMap::new();
    map.insert(2, "a");
    map.insert(1, "b");
    map.insert(3, "c");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_three_elements_4() {
    let mut map = RBTreeMap::new();
    map.insert(2, "a");
    map.insert(3, "b");
    map.insert(1, "c");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_three_elements_5() {
    let mut map = RBTreeMap::new();
    map.insert(3, "a");
    map.insert(1, "b");
    map.insert(2, "c");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_three_elements_6() {
    let mut map = RBTreeMap::new();
    map.insert(3, "a");
    map.insert(2, "b");
    map.insert(1, "c");
    assert_eq!(map.len(), 3);
}

#[test]
fn test_five_elements_1() {
    let mut map = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    map.insert(4, "d");
    map.insert(5, "e");
    assert_eq!(map.len(), 5);
}

#[test]
fn test_five_elements_2() {
    let mut map = RBTreeMap::new();
    map.insert(3, "a");
    map.insert(1, "b");
    map.insert(2, "c");
    map.insert(5, "d");
    map.insert(4, "e");
    assert_eq!(map.len(), 5);
}

#[test]
fn test_clrs_figure_13_4_insert() {
    let mut map = RBTreeMap::new();
    map.insert(11, "a");
    map.insert(2, "b");
    map.insert(14, "c");
    map.insert(1, "d");
    map.insert(15, "f");
    map.insert(7, "e");
    map.insert(5, "g");
    map.insert(8, "h");
    map.insert(4, "h");
    assert_eq!(map.len(), 9);
    assert_eq!(map.root.as_ref().unwrap().key, 7);
}

#[test]
fn test_first_key() {
    let mut map = RBTreeMap::new();
    assert!(map.first_key_value().is_none());
    map.insert(5, "a");
    map.insert(4, "b");
    map.insert(3, "c");
    map.insert(2, "d");
    map.insert(1, "e");
    let maybe_first = map.first_key_value();
    assert!(maybe_first.is_some());
    let first = maybe_first.unwrap();
    assert_eq!(first.0, &1);
    assert_eq!(first.1, &"e");
}

#[test]
fn test_last_key() {
    let mut map = RBTreeMap::new();
    assert!(map.last_key_value().is_none());
    map.insert(5, "a");
    map.insert(4, "b");
    map.insert(3, "c");
    map.insert(2, "d");
    map.insert(1, "e");
    let maybe_last = map.last_key_value();
    assert!(maybe_last.is_some());
    let last = maybe_last.unwrap();
    assert_eq!(last.0, &5);
    assert_eq!(last.1, &"a");
}

#[test]
fn test_iter_1() {
    let mut map: RBTreeMap<i32, &str> = RBTreeMap::new();
    let iter_vec: Vec<_> = map.iter().collect();
    assert_eq!(iter_vec.len(), 0);
    map.insert(3, "a");
    map.insert(2, "b");
    map.insert(1, "c");
    let iter_vec: Vec<_> = map.iter().collect();
    assert_eq!(iter_vec.len(), 3);
    assert_eq!(iter_vec[0].0, &1);
    assert_eq!(iter_vec[0].1, &"c");
    assert_eq!(iter_vec[1].0, &2);
    assert_eq!(iter_vec[1].1, &"b");
    assert_eq!(iter_vec[2].0, &3);
    assert_eq!(iter_vec[2].1, &"a");
}

#[test]
fn test_delete_empty() {
    let mut map: RBTreeMap<i32, i32> = RBTreeMap::new();
    let result = map.remove(&5);
    assert!(result.is_none());
}

#[test]
fn test_delete_singleton() {
    let mut map: RBTreeMap<i32, i32> = RBTreeMap::new();
    map.insert(1, 2);
    let result = map.remove(&5);
    assert!(result.is_none());
    let result = map.remove(&1);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_delete_3_left() {
    let mut map: RBTreeMap<i32, &str> = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    let result = map.remove(&1);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "a");
    assert_eq!(map.len(), 2);
}

#[test]
fn test_delete_3_right() {
    let mut map: RBTreeMap<i32, &str> = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    let result = map.remove(&3);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "c");
    assert_eq!(map.len(), 2);
}

#[test]
fn test_delete_3_root() {
    let mut map: RBTreeMap<i32, &str> = RBTreeMap::new();
    map.insert(1, "a");
    map.insert(2, "b");
    map.insert(3, "c");
    let result = map.remove(&2);
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "b");
    assert_eq!(map.len(), 2);
}