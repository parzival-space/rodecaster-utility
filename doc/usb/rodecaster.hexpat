import std.string;

/// BASE FRAME
using PatchComponent;
using StructuredComponent;
struct RodecasterPacket {
    u8 packet_type;

    if (packet_type == 1) {
        // packet id 1: the packet contains only patch information for changes made to the device
        PatchComponent root;
    } else if (packet_type == 2) {
        // packet id 2: the packet contains the entire device state as a structured object
        StructuredComponent root;
    }
};

/// STRUCTURED DATA
using CollectionComponent;
using KeyValuePair;
struct StructuredComponent {
    std::string::NullString name;

    u8 component_kind;
    if (component_kind == 0) {
        // collection, only has children
        CollectionComponent items;
    } else if (component_kind == 1) {
        // collection with properties, might has children
        u8 properties_count;
        KeyValuePair properties[properties_count];

        u8 has_children;
        if (has_children == 1) {
            // collections with properties can have a max of 255 children
            u8 child_count;
            StructuredComponent children[child_count];
        }
    }
};

struct CollectionComponent {
    u8 child_count_bytes;
    if (child_count_bytes == 0) {
        // empty
    } else if (child_count_bytes == 1) {
        // u8 length
        u8 child_count;
        StructuredComponent children[child_count] [[inline]];
    } else if (child_count_bytes == 2) {
        // u16 length
        u16 child_count;
        StructuredComponent children[child_count] [[inline]];
    }
};

using Value;
struct PatchComponent {
    // first byte seems to be always 0x01. it's very likely a length indicator for the next
    // indices_count byte. until we have found an actual packet using something different here
    // we will just assume that the first byte is always 0x01.
    u8 unknown1;

    u8 indices_count;
    u8 indices[indices_count];

    std::string::NullString name;
    Value value;
};

/// PRIMITIVES
using Value;
struct KeyValuePair {
    std::string::NullString name;

    Value value;
};

struct Value {
    u8 data_length_bytes; // amount of bytes used to indicated the data length
    if (data_length_bytes == 1) {
        u8 data_length;
    } else if (data_length_bytes == 2) {
        u16 data_length;
    }

    u8 data_type;
    if (data_type == 1) { // u32
        u32 data;
    } else if (data_type == 2) { // bool: true
        bool data = true [[comment("TRUE")]];
    } else if (data_type == 3) { // bool: false
        bool data = false [[comment("FALSE")]];;
    } else if (data_type == 4) { // f64
        double data;
    } else if (data_type == 5) { // string
        std::string::NullString data;
    } else if (data_type == 6) { // f64 (possibly double)
        double data;
    } else if (data_type == 8) { // combined (multiple values)
        u8 data[data_length - 1]; // basically a loop of Properties, but I didn't feel like implementing this in ImHex
    }
};


RodecasterPacket packet_root @ 0x00;