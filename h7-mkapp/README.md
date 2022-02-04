# h7-mkapp

Convert a .bin file to .h7.

* Converts the start address to big endian
* Appends the MPEG_2 CRC (big endian).

```
| Addres (BE) (4 bytes) | ... data ... (N bytes) | CRC (BE) (4 bytes) |
```
