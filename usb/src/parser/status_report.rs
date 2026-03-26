use std::collections::{HashMap};
use nom::{IResult};
use nom::number::streaming::{le_u16, le_u8};
use crate::parser::common::{parse_field_value, parse_null_terminated_string, FieldValue};

/// A type in the RodeCaster protocol that can represent multiple structural type.
/// This type can behave similar to Interfaces or Objects.
#[derive(Debug, Clone, PartialEq)]
pub struct StructComponent {
    pub name: String,
    pub properties: HashMap<String, FieldValue>,
    pub children: HashMap<String, Vec<StructComponent>>,
}

pub fn parse_struct_component(input: &[u8]) -> IResult<&[u8], StructComponent> {
    let (input, name) = parse_null_terminated_string(input)?;
    let (input, component_kind) = le_u8(input)?;

    match component_kind {
        0x00 => { // collection, only has children
            let (input, child_count_length) = le_u8(input)?;
            let (input, child_count) = match child_count_length {
                0x00 => (input, 0x00usize), // no children
                0x01 => le_u8(input).map(|result| (result.0, result.1 as usize))?,
                0x02 => le_u16(input).map(|result| (result.0, result.1 as usize))?,
                _ => unreachable!("unknown child count length byte: {:02x?}", child_count_length)
            };

            let mut last_input = input;
            let mut children = HashMap::new();
            for _ in 0..child_count {
                let (input, child) = parse_struct_component(last_input)?;
                last_input = input;

                let child_name = child.name.clone();
                if !children.contains_key(&child_name) {
                    children.insert(child_name.clone(), Vec::new());
                }
                children.get_mut(&child_name).unwrap().push(child);
            };

            Ok((last_input, StructComponent {
                name,
                properties: HashMap::new(),
                children,
            }))
        }
        0x01 => { // entity, has properties, might have children
            let (input, properties_count) = le_u8(input)?;

            let mut last_input = input;
            let mut properties = HashMap::new();
            for _ in 0..properties_count {
                let (input, property_name) = parse_null_terminated_string(last_input)?;
                let (input, property) = parse_field_value(input)?;
                last_input = input;

                properties.insert(property_name, property);
            }

            let (input, close_tag) = le_u8(last_input)?;
            match close_tag {
                0x00 => Ok(( // end of component
                    input, StructComponent {
                        name,
                        properties,
                        children: HashMap::new(),
                    }
                )),
                0x01 => { // begin child list
                    let (input, child_count) = le_u8(input)?;

                    let mut last_input = input;
                    let mut children = HashMap::new();
                    for _ in 0..child_count {
                        let (input, child) = parse_struct_component(last_input)?;
                        last_input = input;

                        let child_name = child.name.clone();
                        if !children.contains_key(&child_name) {
                            children.insert(child_name, Vec::new());
                        }
                        children.get_mut(&child.name.clone()).unwrap().push(child);
                    };

                    Ok((last_input, StructComponent {
                        name,
                        properties,
                        children,
                    }))
                },
                _ => unreachable!("unknown close tag: {:02x?}", close_tag)
            }
        },
        _ => unreachable!("unknown component_kind: {:02x?}", component_kind)
    }
}