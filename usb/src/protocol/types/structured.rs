use std::collections::HashMap;
use std::mem::discriminant;
use anyhow::anyhow;
use nom::error::{Error, ErrorKind};
use nom::IResult;
use nom::number::streaming::{le_u16, le_u8};
use crate::protocol::types::{StreamableType, Value, parse_c_string};

#[derive(Debug, Clone, PartialEq)]
pub struct Structured {
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub children: Vec<Structured>,
}

impl StreamableType for Structured {
    fn parse_from_stream(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (input, name) = parse_c_string(input)?;
        let (input, component_kind) = le_u8(input)?;

        match component_kind {
            0x00 => { // collection, only has children
                // collections can have more than 255 entries
                let (input, child_count_length) = le_u8(input)?;
                let (input, child_count) = match child_count_length {
                    0x00 => (input, 0x00usize), // empty
                    0x01 => le_u8(input).map(|result| (result.0, result.1 as usize))?,
                    0x02 => le_u16(input).map(|result| (result.0, result.1 as usize))?,
                    _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Verify)))?,
                };

                let (input, children) = Self::parse_children(input, child_count)?;
                Ok((input, Structured {
                    name,
                    properties: HashMap::new(),
                    children
                }))
            },
            0x01 => { // object, has children and might have properties
                let (input, properties) = Self::parse_properties(input)?;

                let (input, close_tag) = le_u8(input)?; // check if end of object
                match close_tag {
                    0x00 => Ok(( // end of object
                        input, Structured {
                            name,
                            properties,
                            children: vec![]
                        }
                    )),
                    0x01 => { // has children
                        // objects can have a max of 255 children
                        let (input, child_count) = le_u8(input)?;
                        let (input, children) = Self::parse_children(input, child_count as usize)?;
                        Ok((
                            input,  Structured {
                                name,
                                properties,
                                children
                            }
                        ))
                    },
                    _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Verify)))
                }
            },
            _ => Err(nom::Err::Error(Error::new(input, ErrorKind::Verify)))
        }
    }

    fn write_to_stream(&self, _: &mut Vec<u8>) -> anyhow::Result<()> {
        todo!("There is currently no use case for the write function of the Structured type.")
    }
}

impl Structured {
    /// Helper function to parse the children section of the structured component
    fn parse_children(input: &[u8], child_count: usize) -> IResult<&[u8], Vec<Structured>> {
        let mut children = Vec::new();
        let mut last_input = input;
        for _ in 0..child_count {
            let (input, child) = Structured::parse_from_stream(&last_input)?;
            children.push(child);
            last_input = input;
        };

        Ok((last_input, children))
    }

    /// Helper function to parse the properties section of the structured component
    fn parse_properties(input: &[u8]) -> IResult<&[u8], HashMap<String, Value>> {
        let (input, properties_count) = le_u8(input)?;

        let mut properties = HashMap::new();
        let mut last_input = input;
        for _ in 0..properties_count {
            let (input, property_name) = parse_c_string(last_input)?;
            let (input, property_value) = Value::parse_from_stream(input)?;
            last_input = input;

            properties.insert(property_name, property_value);
        };

        Ok((last_input, properties))
    }

    /// update a value at a given index
    pub fn set_property(&mut self, indices: Vec<usize>, property_name: String, value: Value) -> anyhow::Result<()> {
        if indices.is_empty() {
            if let Some(existing_property) = self.properties.get(&property_name){
                // ensure the new value has the same type as the old value
                if discriminant(existing_property) != discriminant(&value) {
                    return Err(anyhow!("Cannot update property {}: type mismatch (existing: {:?}, new: {:?})", property_name, existing_property, value));
                }
                self.properties.insert(property_name, value);
                Ok(())
            } else {
                Err(anyhow!("Cannot update property {}: property does not exist", property_name))?
            }
        } else {
            self.children.get_mut(indices[0])
                .ok_or_else(|| anyhow!("No such index {}", indices[0]))?
                .set_property(indices[1..].to_vec(), property_name, value)
        }
    }
}