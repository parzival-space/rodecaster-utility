use std::collections::HashMap;
use std::io::{Write};
use std::mem::discriminant;
use nom::IResult;
use nom::number::streaming::{le_u8, le_u16};
use serde::{Deserialize, Serialize};
use crate::error::UsbError;
use crate::protocol::Parseable;
use crate::protocol::types::value::Value;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Composite {
    name: String,
    composite_type: CompositeType,
    has_children: bool,
    properties: HashMap<String, Value>,
    children: Vec<Composite>,
}

impl Composite {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    
    pub fn get_composite_type(&self) -> &CompositeType {
        &self.composite_type
    }
    
    pub fn properties(&mut self) -> &mut HashMap<String, Value> {
        &mut self.properties
    }
    
    pub fn children(&mut self) -> &mut Vec<Composite> {
        &mut self.children
    }
    
    pub fn apply_patch(&mut self, indices: Vec<usize>, name: String, value: Value) -> Result<(), UsbError> {
        if indices.is_empty() {
            if let Some(existing_property) = self.properties.get(&name) {
                if discriminant(existing_property) != discriminant(&value) {
                    return Err(UsbError::PropertyError(format!(
                        "Cannot update property '{}': type mismatch (existing: {:?}, new: {:?})",
                        name, existing_property, value
                    )));
                }
                self.properties.insert(name, value);
                Ok(())
            } else {
                Err(UsbError::PropertyError(format!(
                    "Cannot update property '{}': property does not exist in composite '{}'",
                    name, self.name
                )))
            }
        } else {
            self.children
                .get_mut(indices[0])
                .ok_or_else(|| UsbError::PropertyError(format!(
                    "Cannot apply patch to composite '{}': child index {} out of bounds",
                    self.name, indices[0]
                )))?
                .apply_patch(indices[1..].to_vec(), name, value)
        }
    }
}

/// The Composite object can represent a Collection of Composite objects, in which case it can not
/// hold any properties, of an Object, which can hold properties and might have children.
impl Parseable for Composite {
    fn can_parse(data: &[u8]) -> bool {
        // data cannot be empty and must contain at least 3 bytes:
        // at least 1 byte for an empty name
        // at least 1 byte for 'has properties'
        // at least 1 byte for either 'children count' or 'properties count'
        data.len() >= 3
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, name) = String::parse(data)?;
        let (data, composite_type) = CompositeType::parse(data)?;

        match composite_type {
            CompositeType::Collection => {
                let (data, child_count_bytes) = le_u8(data)?;
                let (mut data, child_count) = match child_count_bytes {
                    0u8 => (data, 0usize), // empty collection
                    1u8 => le_u8(data).map(|res| (res.0, res.1 as usize))?, // u8 for size
                    2u8 => le_u16(data).map(|res| (res.0, res.1 as usize))?, // u16 for size
                    _ => Err(nom::Err::Error(
                        nom::error::Error::new(data, nom::error::ErrorKind::LengthValue)
                    ))?, // invalid child count bytes
                };

                // parse children
                let mut children = Vec::with_capacity(child_count);
                for _ in 0..child_count {
                    let (data_boxed, child) = Self::parse(data)?;
                    data = data_boxed;
                    children.push(child);
                }

                Ok((data, Self {
                    name,
                    composite_type,
                    has_children: true,
                    properties: HashMap::with_capacity(0), // has no properties
                    children,
                }))
            }
            CompositeType::Object => {
                let (mut data, properties_count) = le_u8(data)?;
                let mut properties = HashMap::with_capacity(properties_count.into());
                for _ in 0..properties_count {
                    let (data_boxed, property_name) = String::parse(data)?;
                    let (data_boxed, property_value) = Value::parse(data_boxed)?;
                    data = data_boxed;
                    properties.insert(property_name, property_value);
                }

                let (mut data, has_children) = le_u8(data).map(|res| (res.0, res.1 != 0))?;
                let mut children = Vec::with_capacity(0);
                if has_children {
                    let (mut data_boxed, child_count) = le_u8(data)?;
                    children = Vec::with_capacity(child_count.into());

                    for _ in 0..child_count {
                        let (data_looped, child) = Self::parse(data_boxed)?;
                        data_boxed = data_looped;
                        children.push(child);
                    }

                    data = data_boxed;
                }


                Ok((data, Self {
                    name,
                    composite_type,
                    has_children,
                    properties,
                    children,
                }))
            }
        }

    }

    fn write_to<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        todo!(
            "Serializing of the Composite type is currently not implemented, as there is no use\
            case for sending it down the wire back to the device. For updating properties use the\
            appropriate Patch messages instead."
        )
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub enum CompositeType {
    #[default]
    Object,
    Collection,
}

impl Parseable for CompositeType {
    fn can_parse(data: &[u8]) -> bool {
        // needs to have one byte in stream
        data.len() >= 1
    }

    fn parse(data: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized
    {
        let (data, type_id) = le_u8(data)?;
        match type_id {
            0x00u8 => Ok((data, Self::Collection)),
            0x01u8 => Ok((data, Self::Object)),
            _ => Err(nom::Err::Error(nom::error::Error::new(data, nom::error::ErrorKind::Verify))),
        }
    }

    fn write_to<W: Write>(&self, _writer: &mut W) -> std::io::Result<()> {
        todo!(
            "See Composite::serialize for more details."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAPTURED_COMPOSITE_WITH_CHILDREN: [u8; 0x1F2] = [
        0x52, 0x41, 0x44, 0x49, 0x4F, 0x00, 0x01, 0x03, 0x72, 0x61, 0x64, 0x69, 0x6F, 0x50, 0x61, 0x69,
        0x72, 0x00, 0x01, 0x05, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x72, 0x61, 0x64, 0x69, 0x6F, 0x50, 0x61,
        0x69, 0x72, 0x65, 0x64, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x72, 0x61, 0x64, 0x69,
        0x6F, 0x55, 0x6E, 0x70, 0x61, 0x69, 0x72, 0x00, 0x01, 0x02, 0x05, 0x00, 0x01, 0x02, 0x52, 0x41,
        0x44, 0x49, 0x4F, 0x54, 0x58, 0x00, 0x01, 0x0C, 0x74, 0x78, 0x43, 0x6F, 0x6E, 0x6E, 0x65, 0x63,
        0x74, 0x69, 0x6F, 0x6E, 0x49, 0x64, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78,
        0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x53, 0x4E, 0x00, 0x01, 0x02, 0x05, 0x00, 0x74, 0x78, 0x44,
        0x65, 0x76, 0x69, 0x63, 0x65, 0x54, 0x79, 0x70, 0x65, 0x00, 0x01, 0x05, 0x01, 0xFF, 0xFF, 0xFF,
        0xFF, 0x74, 0x78, 0x43, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x65, 0x64, 0x00, 0x01, 0x01, 0x03,
        0x74, 0x78, 0x52, 0x73, 0x73, 0x69, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78,
        0x53, 0x69, 0x67, 0x6E, 0x61, 0x6C, 0x51, 0x75, 0x61, 0x6C, 0x69, 0x74, 0x79, 0x00, 0x01, 0x05,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78, 0x42, 0x61, 0x74, 0x74, 0x65, 0x72, 0x79, 0x4C, 0x65,
        0x76, 0x65, 0x6C, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78, 0x43, 0x68, 0x61,
        0x72, 0x67, 0x69, 0x6E, 0x67, 0x53, 0x74, 0x61, 0x74, 0x65, 0x00, 0x01, 0x01, 0x03, 0x74, 0x78,
        0x52, 0x65, 0x63, 0x6F, 0x72, 0x64, 0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x52, 0x65, 0x6D, 0x6F,
        0x74, 0x65, 0x4D, 0x75, 0x74, 0x65, 0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x47, 0x61, 0x69, 0x6E,
        0x41, 0x73, 0x73, 0x69, 0x73, 0x74, 0x00, 0x01, 0x05, 0x01, 0x02, 0x00, 0x00, 0x00, 0x74, 0x78,
        0x50, 0x61, 0x64, 0x00, 0x01, 0x01, 0x03, 0x00, 0x52, 0x41, 0x44, 0x49, 0x4F, 0x54, 0x58, 0x00,
        0x01, 0x0C, 0x74, 0x78, 0x43, 0x6F, 0x6E, 0x6E, 0x65, 0x63, 0x74, 0x69, 0x6F, 0x6E, 0x49, 0x64,
        0x00, 0x01, 0x05, 0x01, 0x01, 0x00, 0x00, 0x00, 0x74, 0x78, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65,
        0x53, 0x4E, 0x00, 0x01, 0x02, 0x05, 0x00, 0x74, 0x78, 0x44, 0x65, 0x76, 0x69, 0x63, 0x65, 0x54,
        0x79, 0x70, 0x65, 0x00, 0x01, 0x05, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x74, 0x78, 0x43, 0x6F, 0x6E,
        0x6E, 0x65, 0x63, 0x74, 0x65, 0x64, 0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x52, 0x73, 0x73, 0x69,
        0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78, 0x53, 0x69, 0x67, 0x6E, 0x61, 0x6C,
        0x51, 0x75, 0x61, 0x6C, 0x69, 0x74, 0x79, 0x00, 0x01, 0x05, 0x01, 0x00, 0x00, 0x00, 0x00, 0x74,
        0x78, 0x42, 0x61, 0x74, 0x74, 0x65, 0x72, 0x79, 0x4C, 0x65, 0x76, 0x65, 0x6C, 0x00, 0x01, 0x05,
        0x01, 0x00, 0x00, 0x00, 0x00, 0x74, 0x78, 0x43, 0x68, 0x61, 0x72, 0x67, 0x69, 0x6E, 0x67, 0x53,
        0x74, 0x61, 0x74, 0x65, 0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x52, 0x65, 0x63, 0x6F, 0x72, 0x64,
        0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x52, 0x65, 0x6D, 0x6F, 0x74, 0x65, 0x4D, 0x75, 0x74, 0x65,
        0x00, 0x01, 0x01, 0x03, 0x74, 0x78, 0x47, 0x61, 0x69, 0x6E, 0x41, 0x73, 0x73, 0x69, 0x73, 0x74,
        0x00, 0x01, 0x05, 0x01, 0x02, 0x00, 0x00, 0x00, 0x74, 0x78, 0x50, 0x61, 0x64, 0x00, 0x01, 0x01,
        0x03, 0x00,
    ];

    #[test]
    fn test_parse_composite() {
        let expected_name = "RADIO";
        let expected_property_names = ["radioPair", "radioPaired", "radioUnpair"];
        let expected_child_names = ["RADIOTX", "RADIORX"];

        let result = Composite::parse(&CAPTURED_COMPOSITE_WITH_CHILDREN);
        assert!(result.is_ok());

        let (rest, composite) = result.unwrap();
        assert_eq!(rest.len(), 0); // all bytes should be consumed
        assert_eq!(composite.name, expected_name);
        assert_eq!(composite.composite_type, CompositeType::Object);
        assert_eq!(composite.has_children, true);

        assert_eq!(composite.properties.len(), expected_property_names.len());
        for property_name in &expected_property_names {
            assert!(composite.properties.get(&property_name.to_string()).is_some());
        }

        assert_eq!(composite.children.len(), expected_child_names.len());
    }
}