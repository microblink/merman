use serde::Deserialize;

use indexmap::IndexMap;
use std::cmp::Ord;
use std::collections::{BTreeMap, VecDeque};

enum Direction {
    LeftRight,
    TopDown,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphFormat {
    layout_direction: String,
    nodes:            IndexMap<String, NodeFormat>,
    connections:      Vec<ConnectionFormat>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeFormat {
    name: String,
    op:   Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectionFormat {
    from: String,
    to:   String,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id:   String,
    pub name: String,
    pub op:   Option<String>,
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub from_index: usize,
    pub to_index:   usize,
}

#[derive(Clone, Debug)]
pub struct Graph {
    nodes:            Vec<Node>,
    from_connections: Vec<Vec<Connection>>,
    to_connections:   Vec<Vec<Connection>>,
}

#[derive(Debug)]
pub enum GraphError {
    ParseError { message: String },
    InternalError { message: String },
}

pub struct SortOrder {
    pub order_indices:  Vec<usize>,
    pub depths:         Vec<usize>,
    pub index_at_depth: Vec<usize>,
    pub nodes_in_level: Vec<usize>,
}

impl Graph {
    pub fn from_str(content: &str) -> Result<Graph, GraphError> {
        let parsed: GraphFormat = serde_json::from_str(content).map_err(|e| GraphError::ParseError { message: e.to_string() })?;

        let mut result = Graph {
            nodes:            Vec::new(),
            from_connections: Vec::with_capacity(parsed.nodes.len()),
            to_connections:   Vec::with_capacity(parsed.nodes.len()),
        };

        for _ in 0..parsed.nodes.len() {
            result.from_connections.push(Vec::new());
            result.to_connections.push(Vec::new());
        }

        let mut index_map: BTreeMap<String, usize> = BTreeMap::new();

        for (index, (k, v)) in parsed.nodes.iter().enumerate() {
            result.nodes.push(Node {
                id:   k.clone(),
                name: v.name.clone(),
                op:   v.op.clone(),
            });
            index_map.insert(k.into(), index);
        }

        for connection in parsed.connections.iter() {
            let from_index = *index_map.get(&connection.from).ok_or_else(|| GraphError::InternalError {
                message: format!("Invalid from reference {}", connection.from),
            })?;
            let to_index = *index_map.get(&connection.to).ok_or_else(|| GraphError::InternalError {
                message: format!("Invalid to reference {}", connection.to),
            })?;
            result.from_connections[from_index].push(Connection { from_index, to_index });
            result.to_connections[to_index].push(Connection { from_index, to_index });
        }

        Ok(result)
    }

    pub fn node_size(&self) -> usize {
        self.nodes.len()
    }

    pub fn node(&self, node_index: usize) -> &Node {
        &self.nodes[node_index]
    }

    pub fn to_connections(&self) -> &[Vec<Connection>] {
        &self.to_connections
    }

    pub fn reverse_topological_sort(&self) -> Result<SortOrder, GraphError> {
        // Push nodes which do not have outgoing connections
        let mut nodes_to_visit: VecDeque<(usize, usize)> = self
            .from_connections
            .iter()
            .enumerate()
            .filter(|(_, from_connections)| from_connections.is_empty())
            .map(|(i, _)| (i, 0usize))
            .collect();

        if nodes_to_visit.is_empty() {
            return Err(GraphError::InternalError {
                message: "EmptyGraph".into(),
            });
        }

        let mut nodes_distance: Vec<usize> = vec![usize::MAX; self.node_size()];

        while let Some((current_node_index, current_node_distance)) = nodes_to_visit.pop_front() {
            // Cycle detection
            if current_node_distance >= self.node_size() {
                return Err(GraphError::InternalError {
                    message: format!("Cycle detected at node {}", &self.node(current_node_index).name),
                });
            }

            // Node is visited
            if nodes_distance[current_node_index] == current_node_distance {
                continue;
            }

            nodes_distance[current_node_index] = current_node_distance;

            // Advance distance by one
            let new_node_distance = current_node_distance + 1;

            // Add originating nodes to queue
            for connection in self.to_connections[current_node_index].iter() {
                nodes_to_visit.push_back((connection.from_index, new_node_distance));
            }
        }

        // Get order of nodes
        let order_indices: Vec<usize> = {
            let mut indices: Vec<_> = (0..self.node_size()).collect();
            indices.sort_by(|left_index, right_index| nodes_distance[*left_index].cmp(&nodes_distance[*right_index]));
            indices
        };

        let depths: Vec<_> = order_indices.iter().map(|&i| nodes_distance[i]).collect();

        let num_depth_levels = *depths.last().unwrap() + 1;

        let mut nodes_in_depth_level = vec![0usize; num_depth_levels];

        let mut index_at_depth = vec![0usize; order_indices.len()];

        let mut last_depth_index = 0usize;
        let mut index_within_depth = 0usize;

        for depth_index in 0..depths.len() {
            if depths[depth_index] != last_depth_index {
                last_depth_index = depths[depth_index];
                index_within_depth = 0usize;
            }
            index_at_depth[depth_index] = index_within_depth;
            nodes_in_depth_level[last_depth_index] = index_within_depth + 1;

            index_within_depth += 1;
        }

        Ok(SortOrder {
            order_indices,
            depths,
            index_at_depth,
            nodes_in_level: nodes_in_depth_level,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_add_test() {
        const CONTENT: &str = include_str!("../sample_files/simple_add_with_const.json");

        let graph = Graph::from_str(CONTENT).expect("Could not parse file!");

        assert_eq!(graph.node_size(), 4usize);

        assert_eq!(graph.node(0).name, "Input 0");
    }

    #[test]
    fn simple_add_topological_sort_0() {
        const CONTENT: &str = include_str!("../sample_files/simple_add_with_const.json");

        let graph = Graph::from_str(CONTENT).expect("Could not parse file!");

        let sort_order = graph.reverse_topological_sort().unwrap();

        assert_eq!(graph.node_size(), 4usize);

        assert_eq!(
            sort_order.order_indices.iter().map(|&i| &graph.node(i).name).collect::<Vec<_>>(),
            vec!["Output", "Add", "Input 0", "Bias"]
        );

        assert_eq!(graph.node(sort_order.order_indices[0]).name, "Output");
        assert_eq!(graph.node(sort_order.order_indices[1]).name, "Add");
        assert_eq!(graph.node(sort_order.order_indices[2]).name, "Input 0");
        assert_eq!(graph.node(sort_order.order_indices[3]).name, "Bias");

        assert_eq!(sort_order.depths, vec![0, 1, 2, 2]);
    }

    #[test]
    fn simple_add_topological_sort_1() {
        const CONTENT: &str = include_str!("../sample_files/diamond.json");

        let graph = Graph::from_str(CONTENT).expect("Could not parse file!");

        let sort_order = graph.reverse_topological_sort().unwrap();

        assert_eq!(graph.node_size(), 5usize);

        assert_eq!(graph.node(sort_order.order_indices[0]).name, "Output");
        assert_eq!(graph.node(sort_order.order_indices[1]).name, "Mul");
        assert_eq!(graph.node(sort_order.order_indices[2]).name, "Add");
        assert_eq!(graph.node(sort_order.order_indices[3]).name, "Input 0");
        assert_eq!(graph.node(sort_order.order_indices[4]).name, "Bias");

        assert_eq!(sort_order.depths, vec![0, 1, 2, 3, 3]);
    }
}
