//! Module for Edge store

use crate::graph::store::edge::Edge;
use crate::util::Identity;
use std::borrow::Borrow;
use std::collections::btree_map::{Entry, Iter};
use std::collections::BTreeMap;
use std::fmt;

/// Store structure for edge.
#[derive(Eq, PartialEq, Clone)]
pub(in crate::graph) struct EdgeStore<NodeId: Identity, EdgeId: Identity> {
    inner: BTreeMap<EdgeId, Edge<NodeId, EdgeId>>,
}

impl<NodeId: Identity, EdgeId: Identity> fmt::Debug for EdgeStore<NodeId, EdgeId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.inner))
    }
}

impl<NodeId: Identity, EdgeId: Identity> fmt::Display for EdgeStore<NodeId, EdgeId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut is_first = true;
        f.write_str("{")?;
        for (edge_id, edge) in self.inner.iter() {
            if is_first {
                f.write_fmt(format_args!("{:?}:{}", edge_id, edge))?;
            } else {
                f.write_fmt(format_args!(", {:?}:{}", edge_id, edge))?;
            }
            is_first = false;
        }
        f.write_str("}")
    }
}

impl<NodeId: Identity, EdgeId: Identity> EdgeStore<NodeId, EdgeId> {
    // ---
    // constructor
    // ---

    /// create empty store
    pub(in crate::graph) fn create() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    // ---
    // getter
    // ---

    /// get edge at edge_id
    pub(in crate::graph) fn get_edge<B: ?Sized>(&self, edge_id: &B) -> Option<&Edge<NodeId, EdgeId>>
    where
        EdgeId: Borrow<B>,
        B: Identity,
    {
        self.inner.get(edge_id)
    }

    /// get edge as mutable at edge_id
    pub(in crate::graph) fn get_edge_as_mut<B: ?Sized>(
        &mut self,
        edge_id: &B,
    ) -> Option<&mut Edge<NodeId, EdgeId>>
    where
        EdgeId: Borrow<B>,
        B: Identity,
    {
        self.inner.get_mut(edge_id)
    }

    /// get incidence node ids searched by edge_ids.
    pub(in crate::graph) fn get_incidence_node_ids_by_ids(
        &self,
        edge_ids: &[&EdgeId],
    ) -> Vec<&NodeId> {
        let mut result = Vec::new();
        for edge_id in edge_ids.iter() {
            match self.inner.get(edge_id) {
                None => {
                    continue;
                }
                Some(edge) => {
                    result.extend(edge.get_incidence_node_ids_as_ref());
                }
            }
        }

        result
    }

    /// inner store iter
    pub(in crate::graph) fn inner_store_iter<'a>(
        &'a self,
    ) -> Iter<'a, EdgeId, Edge<NodeId, EdgeId>> {
        self.inner.iter()
    }

    // ---
    // setter
    // ---

    /// insert edge
    pub(in crate::graph) fn insert_edge(
        &mut self,
        edge_id: EdgeId,
        edge: Edge<NodeId, EdgeId>,
    ) -> Option<Edge<NodeId, EdgeId>> {
        self.inner.insert(edge_id, edge)
    }

    /// get as entry
    pub(in crate::graph) fn entry(
        &mut self,
        edge_id: EdgeId,
    ) -> Entry<EdgeId, Edge<NodeId, EdgeId>> {
        self.inner.entry(edge_id)
    }

    // ---
    // checker
    // ---

    /// check exist edge_id
    pub(in crate::graph) fn has_edge_id<B: ?Sized>(&self, edge_id: &B) -> bool
    where
        EdgeId: Borrow<B>,
        B: Identity,
    {
        self.inner.contains_key(edge_id)
    }

    /// check exist same edge
    pub(in crate::graph) fn exist_same_edge(&mut self, edge: &Edge<NodeId, EdgeId>) -> bool {
        self.inner
            .iter()
            .filter(|(_, stored_edge)| (*stored_edge).is_equal_to_without_weight(edge))
            .next()
            .is_some()
    }

    // ---
    // delete
    // ---

    /// clear all edges
    pub(in crate::graph) fn clear(&mut self) {
        self.inner.clear();
    }

    /// remove and get edge at edge_id
    pub(in crate::graph) fn remove<B: ?Sized>(
        &mut self,
        edge_id: &B,
    ) -> Option<Edge<NodeId, EdgeId>>
    where
        EdgeId: Borrow<B>,
        B: Identity,
    {
        self.inner.remove(edge_id)
    }

    /// remove and get edge with edge_id
    pub(in crate::graph) fn remove_with_get_id<B: ?Sized>(
        &mut self,
        edge_id: &B,
    ) -> Option<(EdgeId, Edge<NodeId, EdgeId>)>
    where
        EdgeId: Borrow<B>,
        B: Identity,
    {
        self.inner.remove_entry(edge_id)
    }
}
