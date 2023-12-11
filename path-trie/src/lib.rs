use std::{
    collections::HashMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[derive(Default, Debug)]
struct TrieNode {
    full_path: bool,
    path_in_parents: bool,
    children: HashMap<OsString, TrieNode>,
}

#[derive(Default, Debug)]
pub struct PathTrie {
    root: TrieNode,
}

#[derive(Debug)]
pub enum InsertError {
    RelativePath,
}

impl PathTrie {
    pub fn new() -> Self {
        PathTrie {
            root: TrieNode::default(),
        }
    }

    pub fn insert(&mut self, path: &Path) -> Result<(), InsertError> {
        if !path.is_absolute() {
            return Err(InsertError::RelativePath);
        }

        let mut current_node = &mut self.root;
        let mut path_in_parents = false;

        for c in path.components() {
            if current_node.full_path {
                path_in_parents = true;
            }
            current_node = current_node
                .children
                .entry(c.as_os_str().to_os_string())
                .or_default();
            current_node.path_in_parents = path_in_parents;
        }

        current_node.full_path = true;

        // for c in word.chars() {
        //     current_node = current_node.children.entry(c).or_default();
        // }
        // current_node.is_end_of_word = true;

        Ok(())
    }

    pub fn closest_parent(&self, path: &Path) -> Option<PathBuf> {
        let mut current_node = &self.root;
        let mut closest_parent: Option<PathBuf> = None;

        for c in path.components() {
            match current_node.children.get(c.as_os_str()) {
                Some(node) => {
                    current_node = node;
                    match closest_parent {
                        Some(ref mut p) => p.push(c),
                        None => closest_parent = Some(PathBuf::from(c.as_os_str())),
                    }
                }
                None => break,
            }
        }

        let Some(mut longest_path) = closest_parent else {
            return None;
        };

        loop {
            if current_node.full_path {
                return Some(longest_path);
            }

            if !current_node.path_in_parents {
                return None;
            }

            longest_path.pop();

            match self.node(&longest_path) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
    }

    pub fn contains(&self, path: &Path) -> bool {
        let mut current_node = &self.root;

        for c in path.components() {
            match current_node.children.get(c.as_os_str()) {
                Some(node) => current_node = node,
                None => return false,
            }
        }

        current_node.full_path
    }

    fn node(&self, path: &Path) -> Option<&TrieNode> {
        let mut current_node = &self.root;

        for c in path.components() {
            match current_node.children.get(c.as_os_str()) {
                Some(node) => current_node = node,
                None => return None,
            }
        }

        Some(current_node)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn it_works() {
        let sample_proj: PathBuf = "/home/choc/code/kondo".into();
        let sample_proj2: PathBuf = "/home/choc".into();

        // x.components()

        let mut trie = PathTrie::new();
        // trie.insert("hello");
        // trie.insert("hi");
        // trie.insert("hey");
        // trie.insert("world");
        trie.insert(&sample_proj2).unwrap();
        trie.insert(&sample_proj).unwrap();

        println!("{trie:#?}");

        assert!(trie.contains(&sample_proj));
        assert!(!trie.contains(&PathBuf::from("/home/choc/code")));

        // let x = trie.contains(&PathBuf::from("/home/choc/code/kondo"));
        // println!("x: {}", x);

        let closest_parent = trie.closest_parent(&PathBuf::from("/home/choc/code/car/foo/baz"));
        assert_eq!(closest_parent, Some(sample_proj2));

        // assert!(!trie.contains("hello"));

        // println!("hiiii? {}", trie.contains("hiiii"));
        // assert!(!trie.contains("hiiii"));

        // assert!(false);
    }
}
