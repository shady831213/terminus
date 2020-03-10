use super::{InsnMap, Instruction, Decoder};
use std::ops::{Deref, DerefMut};

struct TreeNode<'a, T> {
    left: Option<*mut TreeNode<'a, T>>,
    right: Option<*mut TreeNode<'a, T>>,
    level: u32,
    value: Option<&'a T>,
}

impl<'a, T> TreeNode<'a, T> {
    fn new(level: u32) -> TreeNode<'a, T> {
        TreeNode {
            left: None,
            right: None,
            level: level,
            value: None,
        }
    }

    fn insert(&mut self, key: u32, value: &'a T) {
        if self.level == 32 {
            self.value = Some(value)
        } else {
            let node = if key & (1 << self.level) == 0 {
                self.left.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            } else {
                self.right.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            };
            unsafe {
                (*node).as_mut().unwrap().insert(key, value)
            }
        }
    }

    fn get_node(node: *mut Self) -> *mut Self {
        let n = unsafe{
            node.as_mut().unwrap()
        };
        if n.left.is_some() && n.right.is_none() {
            Self::get_node(n.left.unwrap())
        } else if n.right.is_some() && n.left.is_none() {
            Self::get_node(n.right.unwrap())
        } else {
            node
        }
    }

    fn get(&mut self, key: u32) -> Option<&'a T> {
        let path_compress= |node:&mut Option<*mut Self>| {
            if let Some(ref n) = node {
                let new_node = Self::get_node(*n);
                if *n != new_node {
                    *node = Some(new_node)
                }
            }
        };
        if self.level == 32 {
            self.value
        } else {
            if let Some(node) = if key & (1 << self.level) == 0 {
                path_compress(&mut self.left);
                self.left
            } else {
                path_compress(&mut self.right);
                self.right
            }{
                unsafe {
                    node.as_mut().unwrap().get(key)
                }
            } else {
                None
            }
        }
    }
}

#[test]
fn test_insert() {
    let mut tree: TreeNode<u32> = TreeNode::new(0);
    tree.insert(7, &7);
    assert_eq!(*tree.get(7).unwrap(), 7);
    assert_eq!(tree.get(8), None);
}