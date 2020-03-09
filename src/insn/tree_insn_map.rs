use super::{InsnMap, Instruction, Decoder};
use std::ops::Deref;

struct TreeNode<'a, T> {
    left: Option<Box<TreeNode<'a, T>>>,
    right: Option<Box<TreeNode<'a, T>>>,
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
                self.left.get_or_insert(Box::new(Self::new(self.level + 1)))
            } else {
                self.right.get_or_insert(Box::new(Self::new(self.level + 1)))
            };
            node.insert(key, value)
        }
    }

    fn get(&self, key: u32) -> Result<&'a T, String> {
        if self.level == 32 {
            Ok(self.value.unwrap())
        } else {
            if let Some(node) = if key & (1 << self.level) == 0 {
                &self.left
            } else {
                &self.right
            } {
                node.get(key)
            } else {
                Err("Invalid key!".to_string())
            }
        }
    }
}

#[test]
fn test_insert() {
    let mut tree: TreeNode<u32> = TreeNode::new(0);
    tree.insert(7, &7);
    println!("get {}", tree.get(7).unwrap());
    println!("{}", tree.get(8).err().unwrap());
}