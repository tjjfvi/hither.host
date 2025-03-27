use std::{
    io::{Write, stdout},
    net::SocketAddr,
    sync::Arc,
};

use bytes::{Buf, Bytes};
use futures::TryFutureExt;

use rustls::ServerConfig;
use rustls_pemfile::{certs, private_key};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream, lookup_host},
};
use tokio_rustls::TlsAcceptor;

use clap::{Args, Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "hitherhost")]
enum Command {
    /// Host an HTTPS proxy for a local server.
    Proxy(ProxyArgs),
    /// Fetch certificate files for `hither.host`.
    Fetch(FetchArgs),
}

#[derive(Debug, Args)]
struct ProxyArgs {
    /// The HTTP server to proxy requests to.
    server_addr: String,
    /// The port to host the HTTPS proxy on.
    port: u16,
}

#[derive(Debug, Args)]
struct FetchArgs {
    file: CertFile,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CertFile {
    Privkey,
    Fullchain,
    Cert,
    Chain,
}

#[tokio::main]
async fn main() {
    match Command::parse() {
        Command::Proxy(args) => proxy(args).await.unwrap(),
        Command::Fetch(args) => fetch(args).await.unwrap(),
    }
}

async fn proxy(args: ProxyArgs) -> anyhow::Result<()> {
    let certs =
        certs(&mut fetch_file(CertFile::Fullchain).await?.reader()).collect::<Result<_, _>>()?;
    let key = private_key(&mut fetch_file(CertFile::Privkey).await?.reader())?.unwrap();

    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec(), b"http/1.0".to_vec()];
    let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listen_addr: SocketAddr = format!("[::]:{}", args.port).parse()?;
    let server_addr: SocketAddr = lookup_host(args.server_addr).await?.next().unwrap();

    println!("https://hither.host:{} -> {}", args.port, server_addr);

    let listener = TcpListener::bind(listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(
            async move {
                let mut inbound = tls_acceptor.accept(inbound).await?;
                let mut outbound = TcpStream::connect(server_addr).await?;
                copy_bidirectional(&mut inbound, &mut outbound).await
            }
            .map_err(|err| eprintln!("{err}")),
        );
    }

    Ok(())
}

async fn fetch(args: FetchArgs) -> anyhow::Result<()> {
    let bytes = fetch_file(args.file).await?;
    stdout().lock().write_all(&bytes)?;
    Ok(())
}

async fn fetch_file(file: CertFile) -> anyhow::Result<Bytes> {
    Ok(
        reqwest::get(format!("https://cert.for.hither.host/{}.pem", match file {
            CertFile::Privkey => "privkey",
            CertFile::Fullchain => "fullchain",
            CertFile::Cert => "cert",
            CertFile::Chain => "chain",
        }))
        .await?
        .bytes()
        .await?,
    )
}
