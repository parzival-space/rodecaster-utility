import std.string;
import std.sys;


/// TYPES
using NullString = std::string::NullString;

enum ValueType: u8 {
    U32 = 0x01,
    BOOL_TRUE = 0x02,
    BOOL_FALSE = 0x03,
    F64 = 0x04,
    STRING = 0x05,
    DOUBLE = 0x06,
    ARRAY = 0x08,
};

struct Value {
    u8 length_bytes [[hidden]];
    if (length_bytes == 1) {
        u8 length;
    } else if (length_bytes == 2) {
        u16 length;
    } else {
        std::unimplemente();
    }

    ValueType type;
    match (type) {
        (ValueType::U32): u32 value;
        (ValueType::BOOL_TRUE): bool value = true;
        (ValueType::BOOL_FALSE): bool value = false;
        (ValueType::F64): double value;
        (ValueType::STRING): NullString value;
        (ValueType::DOUBLE): double value;
        (ValueType::ARRAY): {
            u8 remaing_length_bytes [[hidden]];
            if (remaing_length_bytes == 0) {
                u8 remaing_length = 0;
            } else if (remaing_length_bytes == 1) {
                u8 remaing_length;
            } else if (remaing_length_bytes == 2) {
                u16 remaing_length;
            } else {
                std::unimplemented();
            }

            u8 value[remaing_length];
        }
    }
};

struct KeyValuePair {
    NullString name;
    Value value;
};

enum CompositeType : u8 {
    Collection = 0x00,
    Object = 0x01,
};

struct Composite {
    NullString name;
    CompositeType type;

    match (type) {
        (CompositeType::Collection): {
            u8 child_count_bytes [[hidden]];
            if (child_count_bytes == 0) {
                u8 child_count = 0;
            } else if (child_count_bytes == 1) {
                u8 child_count;
            } else if (child_count_bytes == 2) {
                u16 child_count;
            } else {
                std::unimplemented();
            }

            Composite children[child_count];
        }
        (CompositeType::Object): {
            u8 properties_count;
            KeyValuePair properties[properties_count];

            u8 has_children [[hidden]]; // note: might be the child_count_bytes counter
            if (has_children == 1) {
                u8 child_count;
                Composite children[child_count];
            }
        }
    }
};

struct Index {
    u8 value_bytes [[hidden]];
    if (value_bytes == 0) {
        u8 value = 0;
    } else if (value_bytes == 1) {
        u8 value;
    } else if (value_bytes == 2) {
        u16 value;
    } else {
        std::unimplemented();
    }
};

/// PACKETS
struct PropertyPatchPacket {
    u8 packet_kind_id; // shoud always be 0x01 for this packet
    std::assert(packet_kind_id == 0x01, "This is not a PropertyPatchPacket");

    u8 unknown1 [[comment("Might be the byte count for the indices count, for now we always expect 0x01 here.")]];
    std::assert(unknown1 == 0x01, "unknown1 did not match the expected 0x01");

    u8 indices_count;
    Index indices[indices_count];

    NullString name;
    Value value;

};

struct DeviceStatusPacket {
    u8 packet_kind_id; // shoud always be 0x02 for this packet
    std::assert(packet_kind_id == 0x02, "This is not a DeviceStatusPacket");

    Composite status;
};

/// MAIN
DeviceStatusPacket packet @ 0x00;