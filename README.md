## read_until_slice &nbsp;[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![crates.io Version](https://img.shields.io/crates/v/read_until_slice.svg)](https://crates.io/crates/read_until_slice) [![Documentation](https://docs.rs/read_until_slice/badge.svg)](https://docs.rs/read_until_slice)

The tokio io-util feature provides the method:
```rust
pub async fn read_until(&mut self, delimiter: u8, buf: Vec<u8>) -> Result<usize>
```
on `impl AsyncBufRead + Unpin`.

This reads from an async buffered reader until either EOF or the delimiter is reached.

While useful, it is limited to a single byte delimiter.

This crate extends this by taking a slice as a delimiter instead of a single byte.

```rust
pub async fn read_until(&mut self, delimiter: u8, buf: Vec<u8>) -> Result<usize>
```
on the same `impl AsyncBufRead + Unpin`.

Example

```rust
// Open socket
let stream = TcpStream::connect(addr)
    .await
    .expect("could not connect to remote address");
// Split stream into reader and writer halves
let (reader, mut writer) = split(stream);
// Buffer read stream
let mut reader = BufReader::new(reader);
...
// Read until new line delimiter into buffer
let mut buffer = vec![];
let delimiter = b"\r\n";
let n = reader.read_until(delimiter, &mut buffer)
    .await
    .expect("could not read from socket");
assert_eq!(n, buffer.len());
if buffer.ends_with(delimiter) {
    println!("end of line delimiter reached");
} else {
    println!("end of stream reached");
}
```
