use petgraph::graph::NodeIndex;
use petgraph::{Direction, Graph};
use std::fs;
use std::fs::File;
use std::io::Write;

pub struct ProtoGraphNode {
    is_root: bool,
    is_leaf: bool,
    weight: String,
}

pub fn generate(output_dir: &str) -> Result<Graph<ProtoGraphNode, ()>, anyhow::Error> {
    let mut proto_graph = Graph::<ProtoGraphNode, ()>::new();
    let root_node = proto_graph.add_node(ProtoGraphNode {
        is_root: true,
        is_leaf: false,
        weight: String::from("root"),
    });

    let mut file_names: Vec<String> = vec![];
    for compiled_proto in fs::read_dir(output_dir)? {
        let os_name = compiled_proto?.file_name();
        let name = os_name.to_str().unwrap();
        file_names.push(String::from(name));
    }

    file_names.sort();

    let mut curr_node: NodeIndex;
    let mut prev_node: NodeIndex;

    for name in file_names.iter() {
        let tokens = name.split('.');

        prev_node = root_node;
        for token in tokens {
            let mut node = ProtoGraphNode {
                is_root: false,
                is_leaf: false,
                weight: token.to_string(),
            };

            if token == "rs" {
                node.weight = name.to_string();
                node.is_leaf = true;
            }

            let existing_node = proto_graph
                .neighbors_directed(prev_node, Direction::Outgoing)
                .find(|&x| proto_graph[x].weight == token);

            match existing_node {
                None => {
                    curr_node = proto_graph.add_node(node);
                    proto_graph.add_edge(prev_node, curr_node, ());
                }
                Some(node) => {
                    curr_node = node;
                }
            }

            prev_node = curr_node;
        }
    }

    Ok(proto_graph)
}

pub fn display(
    graph: &Graph<ProtoGraphNode, ()>,
    file: &mut File,
    node: NodeIndex,
) -> Result<(), anyhow::Error> {
    let children = graph.neighbors_directed(node, Direction::Outgoing);

    if graph[node].is_root {
        for child in children {
            display(graph, file, child)?;
        }

        return Ok(());
    }

    if !graph[node].is_leaf {
        if !graph[node].weight.contains("serde") {
            file.write_all(format!("pub mod {} {{\n", graph[node].weight).as_bytes())?;
        }
    }

    if graph[node].is_leaf {
        file.write_all(format!("include!(\"{}\");", graph[node].weight).as_bytes())?;
    } else {
        for child in children {
            display(graph, file, child)?;
        }
    }

    if !graph[node].is_leaf && !graph[node].weight.contains("serde") {
        file.write_all(b"}\n")?;
    }

    Ok(())
}
