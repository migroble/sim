#![deny(clippy::pedantic)]
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::ops::RangeFrom;
use fnv::FnvHashMap;
use k2_tree::K2Tree;

pub struct Graph {
    nodes: Vec<usize>,
    edges: K2Tree,
    id_gen: RangeFrom<usize>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: K2Tree::new(),
            id_gen: 0..,
        }
    }

    pub fn nodes(&self) -> &[usize] {
        &self.nodes
    }

    pub fn add_node(&mut self) -> usize {
        let id = self.id_gen.next().unwrap();
        self.nodes.push(id);
        id
    }

    pub fn remove_node(&mut self, id: usize) {
        if self.nodes.iter().find(|p| **p == id).is_some() {
            for (i, &n) in self.edges.get_row(id).unwrap().iter().enumerate() {
                if n {
                    self.set_edge(id, i, false);
                }
            }

            self.nodes.remove(id);
        }
    }

    fn set_edge(&mut self, id1: usize, id2: usize, value: bool) {
        let (id1, id2) = if id1 > id2 { (id1, id2) } else { (id2, id1) };

        while id1 >= self.edges.matrix_width() || id2 >= self.edges.matrix_width() {
            self.edges.grow();
        }

        self.edges.set(id1, id2, value).unwrap();
    }

    pub fn add_edge(&mut self, id1: usize, id2: usize) {
        self.set_edge(id1, id2, true);
    }

    pub fn remove_edge(&mut self, id1: usize, id2: usize) {
        self.set_edge(id1, id2, false);
    }

    pub fn components(&self) -> FnvHashMap<usize, usize> {
        let mut idx = (0..self.nodes.len()).collect::<Vec<_>>();
        let mut c = idx.clone();

        for l in self.edges.leaves() {
            if l.value {
                if c[idx[l.x]] < c[idx[l.y]] {
                    c[idx[l.y]] = c[idx[l.x]]
                } else {
                    idx[l.x] = idx[l.y];
                }
            }
        }

        idx.into_iter().map(|i| c[i]).enumerate().collect()
    }
}
