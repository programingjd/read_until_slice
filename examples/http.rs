use read_until_slice::AsyncBufReadUntilSliceExt;
use std::str::from_utf8;
use tokio::io::{copy, sink, split, AsyncWriteExt, BufReader};
use tokio::join;
use tokio::net::{lookup_host, TcpStream};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // DNS lookup for http://httpbingo.org
    let addr = lookup_host("httpbingo.org:80")
        .await
        .ok()
        .and_then(|mut it| it.next())
        .unwrap();
    // Open socket
    let stream = TcpStream::connect(addr).await.unwrap();
    // Split stream into reader and writer halves
    let (reader, mut writer) = split(stream);
    // Manual HTTP/1.1 request for /status/200
    let mut reader = BufReader::new(reader);
    let request = b"\
        GET /status/200 HTTP/1.1\r\n\
        Host: httpbingo.org\r\n\
        Connection: close\r\n\
        User-Agent: crate/example/read_until_slice\r\n\
    \r\n";
    println!("{}", from_utf8(request).unwrap());
    // Write request and read response
    join!(
        // Write request
        async move {
            writer.write_all(request).await.unwrap();
        },
        // Read response
        async {
            // Read status line
            let mut buffer = vec![];
            let _ = reader.read_until_slice(b"\r\n", &mut buffer).await.unwrap();
            // If buffer doesn't end with \r\n, it means we exhausted the reader
            // before reaching our delimiter
            assert!(buffer.ends_with(b"\r\n"));
            let status_line = &buffer[..buffer.len() - 2];
            println!("{}", status_line.escape_ascii());

            let mut split = status_line.split(|&b| b == b' ');
            let http_version = split.next().unwrap();
            assert_eq!(http_version, b"HTTP/1.1");
            let status_code = split.next().unwrap();
            assert_eq!(status_code, b"200");
            let status_text = split.next().unwrap();
            assert_eq!(status_text, b"OK");
            // Read headers
            loop {
                buffer.clear();
                let _ = reader.read_until_slice(b"\r\n", &mut buffer).await.unwrap();
                // If buffer doesn't end with \r\n, it means we exhausted the reader
                // before reaching our delimiter
                assert!(buffer.ends_with(b"\r\n"));
                let header_line = &buffer[..buffer.len() - 2];
                println!("{}", header_line.escape_ascii());
                if header_line.is_empty() {
                    break;
                };
            }
            // Discard body if there's one
            copy(&mut reader, &mut sink()).await.unwrap();
        }
    );
}
