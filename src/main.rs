use std::ptr::NonNull;


enum NodeColor {
    Red,
    Black
}

struct Node<K, V>
where
    K: Ord
{
    key: K,
    value: V,
    parent:Option<NonNull<Node<K, V>>>,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
    color: NodeColor,
}

struct RBTreeMap<K, V>
where
    K: Ord
{
    root: Option<Box<Node<K, V>>>,
    size: usize,
}

impl<K, V> RBTreeMap<K, V>
where
    K: Ord
{
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
        // TODO: correctly handle when key is already present
        unsafe {
            let mut node: Option<NonNull<Node<K, V>>> = self.root.as_mut().map(|b: &mut Box<Node<K, V>> | NonNull::from(b.as_mut()));
            let mut new_parent: Option<NonNull<Node<K, V>>> = None;
            while !node.is_none() {
                new_parent = node;
                let mut node_ptr = node.unwrap();
                if key < node_ptr.as_ref().key {
                    node = node_ptr.as_mut().left.as_mut().map(|b| NonNull::from(b.as_mut()));
                } else {
                    node = node_ptr.as_mut().right.as_mut().map(|b| NonNull::from(b.as_mut()));
                }
            }
            let z = Node { key, value, parent: new_parent, left: None, right: None, color: NodeColor::Red };
            if let Some(mut new_parent_ptr) = new_parent {
                if z.key < new_parent_ptr.as_ref().key {
                    new_parent_ptr.as_mut().left = Some(Box::new(z));
                } else {
                    new_parent_ptr.as_mut().right = Some(Box::new(z));
                }
            } else {
                self.root = Some(Box::new(z));
            }
        }
        self.size += 1;
        // self.insert_fixup();
    }


    fn insert_fixup(&mut self) {
        panic!("Not implemented");
    }

    fn rotate_left(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).right.take().expect("Only call rotate_left on a node with a right child");
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

    //   x      y
    //  /   =>   \
    // y          x

    fn rotate_right(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).left.take().expect("Only call rotate_right on a node with a left child");
            (*x).left = y.right.take();
            if let Some(ref mut x_left) = (*x).left {
                x_left.parent = Some(x_ptr);
            }

            y.parent = (*x).parent;

            match (*x).parent {
                None => {
                    self.root = Some(y);
                },
                Some(x_parent_ptr) => {
                    // if x_parent.left is x, set left to y, else set right to y
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

    fn test_two_elements() {
        let mut map = RBTreeMap::new();
        map.insert(12, "abc");
        map.insert(34, "def");
        assert_eq!(map.len(), 2);
    }

}