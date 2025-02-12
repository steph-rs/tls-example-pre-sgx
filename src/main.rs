mod tls;
use std::io::{Read, Write};

fn main() {
    println!("Starting TLS handshake simulation to Binance...");

    match tls::tls_handshake_flow("data-api.binance.vision") {
        Ok(mut tls_stream) => {
            println!("TLS handshake succeeded.");

            let request = b"GET /api/v3/ticker/price?symbol=BTCUSDT HTTP/1.1\r\n\
                             Host: data-api.binance.vision\r\n\
                             Connection: close\r\n\r\n";
            tls_stream.write_all(request).unwrap();
            tls_stream.flush().unwrap();

            let mut response = Vec::new();
            tls_stream.read_to_end(&mut response).unwrap();
            let resp = String::from_utf8_lossy(&response);
            println!("Received decrypted response: {}", resp.len());
            println!("resp: {:?}", resp);
        }
        Err(e) => eprintln!("TLS handshake failed: {}", e),
    }
}
