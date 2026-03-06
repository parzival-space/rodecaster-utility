**HID Report Descriptor**

- Bytes: 46

```c
uint8_t const desc_hid_report[] = {
  0x06, 0x00, 0xFF, 0x09, 0x01, 0xA1, 0x01, 0x75, 0x08, 0x15, 0x00, 0x25, 0xFF, 0x09, 0x02, 0x85, 0x01, 
  0x95, 0x3F, 0x91, 0x02, 0x09, 0x03, 0x85, 0x02, 0x95, 0x3F, 0x81, 0x02, 0x09, 0x04, 0x85, 0x03, 0x95,
  0xFF, 0x91, 0x02, 0x09, 0x05, 0x85, 0x04, 0x95, 0xFF, 0x81, 0x02, 0xC0
};
```

**Report Summary**

<table>
<tr>
    <td colspan="3" align="center">Report ID 1</td>
</tr>
<tr>
    <td>
        <b>Input</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
    <td>
        <b>Output</b><br>
        504 bits (63 bytes fields, 64 bytes on wire)
    </td>
    <td>
        <b>Feature</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
</tr>
<tr style="height: 0.5em;"></tr>
<tr>
    <td colspan="3" align="center">Report ID 2</td>
</tr>
<tr>
    <td>
        <b>Input</b><br>
        504 bits (63 bytes fields, 64 bytes on wire)
    </td>
    <td>
        <b>Output</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
    <td>
        <b>Feature</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
</tr>
<tr style="height: 0.5em;"></tr>
<tr>
    <td colspan="3" align="center">Report ID 3</td>
</tr>
<tr>
    <td>
        <b>Input</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
    <td>
        <b>Output</b><br>
        2040 bits (255 bytes fields, 256 bytes on wire)
    </td>
    <td>
        <b>Feature</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
</tr>
<tr style="height: 0.5em;"></tr>
<tr>
    <td colspan="3" align="center">Report ID 4</td>
</tr>
<tr>
    <td>
        <b>Input</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
    <td>
        <b>Output</b><br>
        0 bits (0 bytes fields, 1 bytes on wire)
    </td>
    <td>
        <b>Feature</b><br>
        2040 bits (255 bytes fields, 256 bytes on wire)
    </td>
</tr>
<tr style="height: 0.5em;"></tr>
</table>