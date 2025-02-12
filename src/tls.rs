use std::{
    io,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use rustls::{pki_types::ServerName, ClientConfig, ClientConnection, StreamOwned};

pub struct MyTcpStream {
    inner: TcpStream,
}

impl MyTcpStream {
    pub(crate) fn set_write_timeout(&self, p0: Option<Duration>) -> io::Result<()> {
        self.inner.set_write_timeout(p0)
    }

    pub(crate) fn set_read_timeout(&self, p0: Option<Duration>) -> io::Result<()> {
        self.inner.set_read_timeout(p0)
    }

    pub fn connect(address: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        println!("MyTcpStream: Connected to {}", address);
        Ok(MyTcpStream { inner: stream })
    }
}

// ocall
impl Read for MyTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        println!("MyTcpStream: read() called");
        self.inner.read(buf)
    }
}

// ocall
impl Write for MyTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        println!("MyTcpStream: write() called ({} bytes)", buf.len());
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        println!("MyTcpStream: flush() called");
        self.inner.flush()
    }
}

fn generate_tls_client_config() -> Arc<ClientConfig> {
    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Arc::new(config)
}

pub fn tls_handshake_flow(
    server_hostname: &str,
) -> anyhow::Result<StreamOwned<ClientConnection, MyTcpStream>> {
    let config = generate_tls_client_config();
    let server_name = ServerName::try_from(server_hostname.to_string())?;
    let address = format!("{}:443", server_hostname);

    let mut tcp_stream = MyTcpStream::connect(&address)?;
    tcp_stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    tcp_stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut connection = ClientConnection::new(config, server_name).unwrap();
    println!("client_connection created");

    while connection.is_handshaking() {
        println!("(ECALL) Handshake in progress...");
        if connection.wants_write() {
            let mut buf = Vec::new();
            println!("(ECALL) Handshake in progress... write");

            connection.write_tls(&mut buf)?;
            println!("buf: {:?}", buf);
            tcp_stream.write_all(&buf)?;
        }
        if connection.wants_read() {
            println!("(ECALL) Handshake in progress... read");
            let mut buf = vec![0u8; 4096];
            let bytes_read = tcp_stream.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            buf.truncate(bytes_read);
            connection.read_tls(&mut &buf[..])?;
        }
        connection.process_new_packets()?;
    }

    println!("(ECALL) Handshake completed successfully.");

    Ok(StreamOwned::new(connection, tcp_stream))
}
