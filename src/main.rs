use std::ptr::NonNull;


enum NodeColor {
    Red,
    Black
}

struct Node<K, V> where K: PartialEq {
    key: K,
    value: V,
    parent:Option<NonNull<Node<K, V>>>,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
    color: NodeColor,
}

struct RBTreeMap<K, V> where K: PartialEq {
    root: Option<Box<Node<K, V>>>,
    size: usize,
}

impl<K, V> RBTreeMap<K, V> where K: PartialEq {
    pub fn new() -> Self {
        RBTreeMap { root: None, size: 0 }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.size = 0;
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.is_empty() {
            let node = Node { key, value, parent: None, left: None, right: None, color: NodeColor::Black };
            self.root = Some(Box::new(node));
        } else {
            panic!("Not yet implemented");
        }
        self.size += 1;
    }

    fn _insert(&mut self, key: K, value: V, parent: NonNull<Node<K, V>>) {
        let node = Node {
            key,
            value,
            parent: Some(parent),
            left: None,
            right: None,
            color: unsafe {
                match parent.as_ref().color {
                    NodeColor::Red => NodeColor::Black,
                    NodeColor::Black => NodeColor::Red,
                }
            }
        };
    }

    fn rotate_left(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).right.take().expect("Only call rotate_left on a node with a right child");

            // move left child
            (*x).right = y.left.take();
            if let Some(ref mut x_right) = (*x).right {
                x_right.parent = Some(x_ptr);
            }

            y.parent = (*x).parent;

            match (*x).parent {
                None => {
                    self.root = Some(y);
                },
                Some(x_parent_ptr) => {
                    let x_parent = &mut(*x_parent_ptr.as_ptr());
                    if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x) }) {
                        x_parent.left = Some(y);
                    } else {
                        x_parent.right = Some(y);
                    }
                }
            }
        }
    }

    fn rotate_right(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).left.take().expect("Only call rotate_right on a node with a left child");
        }
    }

}



fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton_len() {
        let mut map = RBTreeMap::new();
        assert_eq!(map.len(), 0);
        map.insert(12, "abc");
        assert_eq!(map.len(), 1);
    }

}