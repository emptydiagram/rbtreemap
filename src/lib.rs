use std::ptr::NonNull;
use std::fmt::{self, Debug};

#[cfg(test)]
mod tests;


#[derive(PartialEq, Eq, Debug)]
enum NodeColor {
    Red,
    Black
}

struct Node<K, V>
{
    key: K,
    value: V,
    parent:Option<NonNull<Node<K, V>>>,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
    color: NodeColor,
}

impl<K, V> Debug for Node<K, V>
where
    K: Debug,
    V: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ [{:?}] {:?} -> {:?}, l: {:?}, r: {:?}", self.color, self.key, self.value, self.left, self.right)
    }
}

impl <K, V> Node<K, V>
{
    #[inline]
    fn is_black(&self) -> bool {
        self.color == NodeColor::Black
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color == NodeColor::Red
    }

    fn unwrap_parent(&self) -> &Node<K, V> {
        match self.parent {
            None => panic!("No parent"),
            Some(p_ptr) => unsafe { p_ptr.as_ref() },
        }
    }

    fn unwrap_parent_mut(&mut self) -> &mut Node<K, V> {
        match self.parent {
            None => panic!("No parent"),
            Some(mut p_ptr) => unsafe { p_ptr.as_mut() },
        }
    }

    fn leftmost_descendant(&self) -> &Node<K, V> {
        let mut curr = self;
        while let Some(ref next) = curr.left {
            curr = next;
        }
        curr
    }

    fn rightmost_descendant(&self) -> &Node<K, V> {
        let mut curr = self;
        while let Some(ref next) = curr.right {
            curr = next;
        }
        curr
    }

    fn first_right_strict_ancestor(&self) -> Option<&Node<K, V>> {
        let mut curr = self;
        while let Some(curr_parent) = curr.parent.map(|node_ptr| unsafe { node_ptr.as_ref() }) {
            if curr_parent.left.is_some() && std::ptr::eq(&**curr_parent.left.as_ref().unwrap(), curr) {
                return Some(curr_parent);
            }
            curr = curr_parent;
        }
        None

    }
}

pub struct RBTreeMap<K, V>
where
    K: Ord
{
    root: Option<Box<Node<K, V>>>,
    size: usize,
}

pub struct Iter<'a, K: Ord, V>
{
    curr: Option<&'a Node<K, V>>,
}


impl<K, V> RBTreeMap<K, V>
where
    K: Ord + Debug,
    V: Debug
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

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut maybe_node = self.root.as_ref().map(|box_node| &*box_node);
        while let Some(curr_node) = maybe_node {
            if curr_node.key == *key {
                return Some(&curr_node.value);
            } else if *key < curr_node.key {
                maybe_node = curr_node.left.as_ref();
            } else {
                maybe_node = curr_node.right.as_ref();
            }
        }
        None
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| {
            let first = r.leftmost_descendant();
            (&first.key, &first.value)
        })
    }

    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| {
            let first = r.rightmost_descendant();
            (&first.key, &first.value)
        })
    }


    pub fn insert(&mut self, key: K, value: V) {
        let fixup_ptr: NonNull<Node<K, V>> = {
            let mut current = &mut self.root;
            let mut parent: Option<*mut Node<K, V>> = None;
            while let Some(node) = current {
                parent = Some(&mut **node);
                if key < node.key {
                    current = &mut node.left;
                } else {
                    current = &mut node.right;
                }
            }

            let parent_nn: Option<NonNull<Node<K, V>>> = parent.map(|n| unsafe { NonNull::new_unchecked(n) });
            let z = Box::new(Node {
                key, value, parent: parent_nn, left: None, right: None, color: NodeColor::Red
            });

            *current = Some(z);
            self.size += 1;
            current
                .as_mut()
                .map(|b| unsafe { NonNull::new_unchecked(b.as_mut()) })
                .unwrap()
        };
        self.insert_fixup(fixup_ptr);
    }


    fn insert_fixup(&mut self, mut z: NonNull<Node<K, V>>) {
        // invariants:
        //  - z is red,
        //  - if z.parent is root then it's black,
        //  - if there's a violation, then (root is red) XOR (there is a red-red parent-child combo)
        loop {
            // if no parent, z is root. there are no other R-B tree violations, so exit the loop
            let z_node = unsafe { z.as_mut() };
            if z_node.parent.is_none() {
                break;
            }
            let z_p: *mut Node<K, V> = z_node.unwrap_parent_mut();

            unsafe {
                if (*z_p).is_black() || (*z_p).parent.is_none() { break; }

                // if z.parent.parent is None, then z.parent is root, so z.parent is black
                // so z.parent.parent is Some
                let z_p_p: *mut Node<K, V> = (*z_p).unwrap_parent_mut();

                if (*z_p_p).left.is_some() && std::ptr::eq(z_p, (*z_p_p).left.as_ref().unwrap().as_ref()) {
                    // z.parent is left child
                    let maybe_z_p_p_r = (*z_p_p).right.as_mut();
                    if maybe_z_p_p_r.is_some() && maybe_z_p_p_r.as_ref().unwrap().is_red() {
                        let y = maybe_z_p_p_r.unwrap();
                        (*z_p).color = NodeColor::Black;
                        y.color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        z = (*z_p).parent.unwrap();
                    } else {
                        // z.parent doesn't have a red sibling
                        if (*z_p).right.is_some() && std::ptr::eq(z.as_ref(), &**(*z_p).right.as_ref().unwrap()) {
                            z = z.as_ref().parent.unwrap();
                            self.rotate_left(z);
                        }
                        (*z_p).color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        self.rotate_right(NonNull::new_unchecked(z_p_p));
                    }
                } else {
                    // z.parent is right child
                    let maybe_z_p_p_l = (*z_p_p).left.as_mut();
                    if maybe_z_p_p_l.is_some() && maybe_z_p_p_l.as_ref().unwrap().is_red() {
                        let y = maybe_z_p_p_l.unwrap();
                        (*z_p).color = NodeColor::Black;
                        y.color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        z = (*z_p).parent.unwrap();
                    } else {
                        if (*z_p).left.is_some() && std::ptr::eq(z.as_ref(), &**(*z_p).left.as_ref().unwrap()) {
                            z = z.as_ref().parent.unwrap();
                            self.rotate_right(z);
                        }
                        (*z_p).color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        self.rotate_left(NonNull::new_unchecked(z_p_p));
                    }
                }
            }
        }
        self.root.as_mut().unwrap().color = NodeColor::Black;
    }

    fn rotate_left(&mut self, mut x_ptr: NonNull<Node<K, V>>) {
        let x_ref = unsafe { x_ptr.as_mut() };
        let mut y = x_ref.right.take().expect("Only call rotate_left on a node with a right child");
        let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
        x_ref.right = y.left.take();
        if let Some(ref mut x_right) = x_ref.right {
            x_right.parent = Some(x_ptr);
        }

        y.parent = x_ref.parent;

        match x_ref.parent {
            None => {
                let x = self.root.take().unwrap();
                self.root = Some(y);
                self.root.as_mut().unwrap().left = Some(x);
            },
            Some(mut x_parent_ptr) => {
                let x_parent = unsafe { x_parent_ptr.as_mut() };
                if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x_ref) }) {
                    let x = x_parent.left.take().unwrap();
                    x_parent.left = Some(y);
                    x_parent.left.as_mut().unwrap().left = Some(x);
                } else {
                    let x = x_parent.right.take().unwrap();
                    x_parent.right = Some(y);
                    x_parent.right.as_mut().unwrap().left = Some(x);
                }
            }
        }
        x_ref.parent = Some(y_ptr);
    }

    //   x      y
    //  /   =>   \
    // y          x

    fn rotate_right(&mut self, mut x_ptr: NonNull<Node<K, V>>) {
        let x_ref = unsafe { x_ptr.as_mut() };
        let mut y = x_ref.left.take().expect("Only call rotate_right on a node with a left child");
        let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
        x_ref.left = y.right.take();
        if let Some(ref mut x_left) = x_ref.left {
            x_left.parent = Some(x_ptr);
        }

        y.parent = x_ref.parent;

        match x_ref.parent {
            None => {
                let x = self.root.take().unwrap();
                self.root = Some(y);
                self.root.as_mut().unwrap().right = Some(x);
            },
            Some(mut x_parent_ptr) => {
                // if x_parent.left is x, set left to y, else set right to y
                let x_parent = unsafe { x_parent_ptr.as_mut() };
                if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x_ref) }) {
                    let x = x_parent.left.take().unwrap();
                    x_parent.left = Some(y);
                    x_parent.left.as_mut().unwrap().right = Some(x);
                } else {
                    let x = x_parent.right.take().unwrap();
                    x_parent.right = Some(y);
                    x_parent.right.as_mut().unwrap().right = Some(x);
                }
            }
        }
        x_ref.parent = Some(y_ptr);
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        let mut prev: Option<&Box<Node<K, V>>> = None;
        let mut curr = self.root.as_ref();
        while let Some(node) = curr {
            prev = curr;
            curr = node.left.as_ref();
        }
        Iter {
            curr: prev.map(|n| &**n),
        }
    }

}



impl<'a, K: Ord + Debug, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.curr.take();
        if let Some(node) = head {
            // find the next node, if there is one, and add it to nodes
            // two ways for next node:
            //  - it's the leftmost descendant of the right child
            //  - it's the first right ancestor
            let maybe_rc_lm_desc = node.right.as_ref().map(|rc| rc.leftmost_descendant());
            if let Some(rc_lm_desc) = maybe_rc_lm_desc {
                self.curr = Some(rc_lm_desc);
            } else {
                if let Some(frs_anc) = node.first_right_strict_ancestor() {
                    self.curr = Some(frs_anc);
                }
            }

        }
        head.map(|node| (&node.key, &node.value))
    }
}
