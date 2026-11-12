use std::collections::HashMap;
use nom::{error_position, IResult};
use nom::error::ErrorKind;
use nom::number::streaming::*;
use crate::protocol::{Deserialize, Serialize};
use crate::protocol::types::value::Value;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum NodeType {
    #[default]
    List,
    Object,
}

impl Serialize for NodeType {
    fn serialize(&self) -> std::io::Result<Vec<u8>>
    where
        Self: Sized
    {
        panic!(
            "Serializing of the Composite type is currently not implemented, as there is no use\
            case for sending it down the wire back to the device. For updating properties use the\
            appropriate Patch messages instead."
        )
    }
}

impl Deserialize for NodeType {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        match le_u8(data)? {
            (data, 0u8) => Ok((data, NodeType::List)),
            (data, 1u8) => Ok((data, NodeType::Object)),
            _ => Err(nom::Err::Error(error_position!(data, ErrorKind::Fail)))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Node {
    pub name: String,
    pub node_type: NodeType,
    pub children: Vec<Node>,
    pub properties: HashMap<String, Value>,
}

impl Serialize for Node {
    fn can_serialize(&self) -> bool
    where
        Self: Sized,
    {
        false
    }

    fn serialize(&self) -> std::io::Result<Vec<u8>> {
        panic!(
            "Serializing of the Composite type is currently not implemented, as there is no use\
            case for sending it down the wire back to the device. For updating properties use the\
            appropriate Patch messages instead."
        )
    }
}

impl Deserialize for Node {
    fn deserialize(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, node_name) = String::deserialize(data)?;
        let (data, node_type) = NodeType::deserialize(data)?;

        match node_type {
            NodeType::List => {
                // parse child nodes
                let (data, child_count_bytes) = le_u8(data)?;
                let (mut data, child_count) = match child_count_bytes {
                    0u8 => Ok((data, 0usize)),
                    1u8 => le_u8(data).map(|(data, value)| (data, value as usize)),
                    2u8 => le_u16(data).map(|(data, value)| (data, value as usize)),
                    _ => Err(nom::Err::Error(error_position!(data, ErrorKind::LengthValue)))
                }?;

                let mut child_nodes = Vec::with_capacity(child_count);
                for _ in 0..child_count {
                    let (data_inner, child_node) = Node::deserialize(data)?;
                    child_nodes.push(child_node);
                    data = data_inner;
                }

                Ok((data, Self {
                    name: node_name,
                    node_type,
                    children: child_nodes,
                    properties: HashMap::with_capacity(0),
                }))
            }
            NodeType::Object => {
                // parse properties
                let (mut data, properties_count) = le_u8(data)?;

                let mut properties = HashMap::with_capacity(properties_count.into());
                for _ in 0..properties_count {
                    let (data_inner, property_name) = String::deserialize(data)?;
                    let (data_inner, property_value) = Value::deserialize(data_inner)?;
                    properties.insert(property_name, property_value);

                    data = data_inner;
                }

                // parse child nodes
                let (data, has_children) = le_u8(data).map(|(data, value)| (data, value != 0))?; // todo: offload this into a dedicated bool serialize implementation
                let (data, children) = match has_children {
                    false => (data, Vec::with_capacity(0)),
                    true => {
                        let (mut data_inner, child_count) = le_u8(data)?;
                        let mut children = Vec::with_capacity(child_count.into());

                        for _ in 0..child_count {
                            let (data_inner_loop, child_node) = Node::deserialize(data_inner)?;
                            children.push(child_node);
                            data_inner = data_inner_loop;
                        }

                        (data_inner, children)
                    }
                };

                Ok((data, Self {
                    name: node_name,
                    node_type,
                    children,
                    properties,
                }))
            }
        }
    }
}