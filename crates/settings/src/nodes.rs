//! Indexed node container providing O(1) lookup by ID.
//!
//! This module provides the [`Nodes<T>`] container which stores leaf nodes
//! with assigned IDs while preserving tree structure for menu building.

use crate::host::Host;
use crate::types::{Action, Entry, Group};
use std::fmt;

/// Unique identifier for a leaf node within a [`Nodes`] container.
///
/// Created during container construction. Use with [`Nodes::get()`]
/// to retrieve the associated value.
///
/// # Display Format
///
/// Formats as `"node_{index}"` for use as menu item IDs.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a new `NodeId` from an index.
    ///
    /// This is used internally during construction and for parsing menu IDs.
    #[must_use]
    pub fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Returns the underlying index.
    ///
    /// Use this when parsing menu IDs back to `NodeId`.
    #[must_use]
    pub fn index(&self) -> usize {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node_{}", self.0)
    }
}

/// A node in the indexed tree.
///
/// Use pattern matching to distinguish leaves from groups when
/// building menus or iterating the tree structure.
#[derive(Debug, Clone)]
pub enum Node<T> {
    /// A leaf node with its value and assigned ID.
    Leaf {
        /// Unique identifier for this leaf.
        id: NodeId,
        /// The leaf value.
        value: T,
    },
    /// A named group containing nested nodes.
    Group {
        /// Display name for the submenu.
        name: String,
        /// Child nodes.
        children: Vec<Node<T>>,
    },
}

impl<T> Node<T> {
    /// Returns the ID if this is a `Leaf`, `None` for `Group`.
    #[must_use]
    pub fn id(&self) -> Option<NodeId> {
        match self {
            Self::Leaf { id, .. } => Some(*id),
            Self::Group { .. } => None,
        }
    }

    /// Returns `true` if this is a `Leaf` variant.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Returns `true` if this is a `Group` variant.
    #[must_use]
    pub fn is_group(&self) -> bool {
        matches!(self, Self::Group { .. })
    }
}

/// Generic container providing O(1) lookup by [`NodeId`].
///
/// Built from config data during settings load. Provides both flat
/// iteration (for lookup) and tree iteration (for menu building).
///
/// # Type Parameter
///
/// `T: Clone` - The leaf value type. Clone is required because values
/// are stored in both the tree structure and the flat lookup table.
#[derive(Debug, Clone)]
pub struct Nodes<T> {
    /// Tree structure for menu building.
    tree: Vec<Node<T>>,
    /// Flat list of leaves for O(1) lookup by `NodeId`.
    leaves: Vec<T>,
}

impl Nodes<Action> {
    /// Build from config entries, assigning IDs during depth-first traversal.
    #[must_use]
    pub fn from_entries(entries: Vec<Entry>) -> Self {
        let mut leaves = Vec::new();
        let tree = entries
            .into_iter()
            .map(|e| Self::convert_entry(e, &mut leaves))
            .collect();
        Self { tree, leaves }
    }

    fn convert_entry(entry: Entry, leaves: &mut Vec<Action>) -> Node<Action> {
        match entry {
            Entry::Action(action) => {
                let id = NodeId::from_index(leaves.len());
                leaves.push(action.clone());
                Node::Leaf { id, value: action }
            }
            Entry::Group(Group { name, entries }) => {
                let children = entries
                    .into_iter()
                    .map(|e| Self::convert_entry(e, leaves))
                    .collect();
                Node::Group { name, children }
            }
        }
    }
}

impl Nodes<Host> {
    /// Build from hostnames (flat, no groups).
    #[must_use]
    pub fn from_hostnames(hostnames: Vec<String>) -> Self {
        let mut leaves = Vec::new();
        let tree = hostnames
            .into_iter()
            .enumerate()
            .map(|(i, hostname)| {
                let host = Host { hostname };
                leaves.push(host.clone());
                Node::Leaf {
                    id: NodeId::from_index(i),
                    value: host,
                }
            })
            .collect();
        Self { tree, leaves }
    }
}

impl<T> Nodes<T> {
    /// O(1) lookup by ID.
    ///
    /// Returns `Some(&T)` if the ID is valid, `None` otherwise.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.leaves.get(id.0)
    }

    /// Iterate all leaf values with their IDs (flat, depth-first order).
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> + '_ {
        self.leaves
            .iter()
            .enumerate()
            .map(|(i, v)| (NodeId::from_index(i), v))
    }

    /// Access tree structure for menu building.
    #[must_use]
    pub fn nodes(&self) -> &[Node<T>] {
        &self.tree
    }

    /// Returns the number of leaf nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    /// Returns `true` if there are no leaf nodes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === User Story 1: Menu Item Click Lookup (P1) ===

    #[test]
    fn test_action_lookup_by_id() {
        let entries = vec![
            Entry::Action(Action {
                name: "Deploy".into(),
                cmd: "deploy.sh".into(),
            }),
            Entry::Group(Group {
                name: "Servers".into(),
                entries: vec![Entry::Action(Action {
                    name: "Prod".into(),
                    cmd: "ssh prod".into(),
                })],
            }),
        ];

        let nodes = Nodes::from_entries(entries);

        // O(1) lookup by ID
        let id0 = NodeId::from_index(0);
        let action0 = nodes.get(id0).expect("should find action at index 0");
        assert_eq!(action0.name, "Deploy");
        assert_eq!(action0.cmd, "deploy.sh");

        let id1 = NodeId::from_index(1);
        let action1 = nodes.get(id1).expect("should find action at index 1");
        assert_eq!(action1.name, "Prod");
        assert_eq!(action1.cmd, "ssh prod");
    }

    #[test]
    fn test_host_lookup_by_id() {
        let hostnames = vec!["staging".into(), "prod".into(), "dev".into()];
        let nodes = Nodes::from_hostnames(hostnames);

        let id0 = NodeId::from_index(0);
        assert_eq!(nodes.get(id0).unwrap().hostname, "staging");

        let id1 = NodeId::from_index(1);
        assert_eq!(nodes.get(id1).unwrap().hostname, "prod");

        let id2 = NodeId::from_index(2);
        assert_eq!(nodes.get(id2).unwrap().hostname, "dev");
    }

    #[test]
    fn test_invalid_id_returns_none() {
        let entries = vec![Entry::Action(Action {
            name: "Test".into(),
            cmd: "test".into(),
        })];
        let nodes = Nodes::from_entries(entries);

        // ID beyond the valid range should return None
        let invalid_id = NodeId::from_index(999);
        assert!(nodes.get(invalid_id).is_none());
    }

    // === User Story 2: Menu Building with IDs (P2) ===

    #[test]
    fn test_iter_yields_ids_with_values() {
        let entries = vec![
            Entry::Action(Action {
                name: "First".into(),
                cmd: "first".into(),
            }),
            Entry::Action(Action {
                name: "Second".into(),
                cmd: "second".into(),
            }),
        ];

        let nodes = Nodes::from_entries(entries);
        let items: Vec<_> = nodes.iter().collect();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0.index(), 0);
        assert_eq!(items[0].1.name, "First");
        assert_eq!(items[1].0.index(), 1);
        assert_eq!(items[1].1.name, "Second");
    }

    #[test]
    fn test_iter_ids_are_stable() {
        let entries = vec![
            Entry::Action(Action {
                name: "A".into(),
                cmd: "a".into(),
            }),
            Entry::Action(Action {
                name: "B".into(),
                cmd: "b".into(),
            }),
        ];

        let nodes = Nodes::from_entries(entries);

        // First iteration
        let first: Vec<_> = nodes.iter().map(|(id, _)| id.index()).collect();

        // Second iteration
        let second: Vec<_> = nodes.iter().map(|(id, _)| id.index()).collect();

        // IDs should be identical
        assert_eq!(first, second);
    }

    #[test]
    fn test_independent_id_spaces() {
        let actions = vec![Entry::Action(Action {
            name: "Action".into(),
            cmd: "cmd".into(),
        })];
        let hosts = vec!["host1".into()];

        let action_nodes = Nodes::from_entries(actions);
        let host_nodes = Nodes::from_hostnames(hosts);

        // Both start at index 0
        let action_ids: Vec<_> = action_nodes.iter().map(|(id, _)| id.index()).collect();
        let host_ids: Vec<_> = host_nodes.iter().map(|(id, _)| id.index()).collect();

        assert_eq!(action_ids, vec![0]);
        assert_eq!(host_ids, vec![0]);
    }

    // === User Story 3: Tree Structure Preservation (P3) ===

    #[test]
    fn test_nodes_preserves_tree_structure() {
        let entries = vec![
            Entry::Action(Action {
                name: "Root".into(),
                cmd: "root".into(),
            }),
            Entry::Group(Group {
                name: "SubMenu".into(),
                entries: vec![Entry::Action(Action {
                    name: "Child".into(),
                    cmd: "child".into(),
                })],
            }),
        ];

        let nodes = Nodes::from_entries(entries);
        let tree = nodes.nodes();

        assert_eq!(tree.len(), 2);

        // First is a leaf
        assert!(tree[0].is_leaf());
        if let Node::Leaf { value, .. } = &tree[0] {
            assert_eq!(value.name, "Root");
        }

        // Second is a group
        assert!(tree[1].is_group());
        if let Node::Group { name, children } = &tree[1] {
            assert_eq!(name, "SubMenu");
            assert_eq!(children.len(), 1);
            assert!(children[0].is_leaf());
        }
    }

    #[test]
    fn test_deeply_nested_groups() {
        let entries = vec![Entry::Group(Group {
            name: "Level1".into(),
            entries: vec![Entry::Group(Group {
                name: "Level2".into(),
                entries: vec![Entry::Group(Group {
                    name: "Level3".into(),
                    entries: vec![Entry::Action(Action {
                        name: "Deep".into(),
                        cmd: "deep".into(),
                    })],
                })],
            })],
        })];

        let nodes = Nodes::from_entries(entries);
        let tree = nodes.nodes();

        // Navigate to the deeply nested action
        let level1 = &tree[0];
        assert!(level1.is_group());

        if let Node::Group { children, .. } = level1 {
            let level2 = &children[0];
            assert!(level2.is_group());

            if let Node::Group { children, .. } = level2 {
                let level3 = &children[0];
                assert!(level3.is_group());

                if let Node::Group { children, .. } = level3 {
                    let deep = &children[0];
                    assert!(deep.is_leaf());
                    if let Node::Leaf { value, .. } = deep {
                        assert_eq!(value.name, "Deep");
                    }
                }
            }
        }

        // Still only 1 leaf in flat iteration
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn test_flat_hosts_no_groups() {
        let hosts = vec!["h1".into(), "h2".into(), "h3".into()];
        let nodes = Nodes::from_hostnames(hosts);

        // All entries at root level are leaves
        for node in nodes.nodes() {
            assert!(node.is_leaf());
            assert!(!node.is_group());
        }

        assert_eq!(nodes.len(), 3);
    }

    // === Edge Cases ===

    #[test]
    fn test_empty_container() {
        let actions: Nodes<Action> = Nodes::from_entries(vec![]);
        let hosts: Nodes<Host> = Nodes::from_hostnames(vec![]);

        assert!(actions.is_empty());
        assert!(hosts.is_empty());
        assert_eq!(actions.len(), 0);
        assert_eq!(hosts.len(), 0);
        assert!(actions.get(NodeId::from_index(0)).is_none());
    }

    #[test]
    fn test_duplicate_names_unique_ids() {
        let entries = vec![
            Entry::Action(Action {
                name: "Same".into(),
                cmd: "cmd1".into(),
            }),
            Entry::Action(Action {
                name: "Same".into(),
                cmd: "cmd2".into(),
            }),
        ];

        let nodes = Nodes::from_entries(entries);

        // Same names but different IDs
        let items: Vec<_> = nodes.iter().collect();
        assert_eq!(items[0].0.index(), 0);
        assert_eq!(items[1].0.index(), 1);
        assert_eq!(items[0].1.name, items[1].1.name); // Same name
        assert_ne!(items[0].1.cmd, items[1].1.cmd); // Different cmd
    }

    #[test]
    fn test_empty_group() {
        let entries = vec![Entry::Group(Group {
            name: "EmptyGroup".into(),
            entries: vec![],
        })];

        let nodes = Nodes::from_entries(entries);

        // Empty group still exists in tree
        assert_eq!(nodes.nodes().len(), 1);
        assert!(nodes.nodes()[0].is_group());

        // But no leaves
        assert!(nodes.is_empty());
        assert_eq!(nodes.len(), 0);
    }

    // === NodeId Display ===

    #[test]
    fn test_node_id_display() {
        let id = NodeId::from_index(42);
        assert_eq!(id.to_string(), "node_42");
        assert_eq!(id.index(), 42);
    }

    #[test]
    fn test_node_id_equality() {
        let id1 = NodeId::from_index(5);
        let id2 = NodeId::from_index(5);
        let id3 = NodeId::from_index(6);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}
