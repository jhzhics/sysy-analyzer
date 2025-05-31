use tower_lsp::{LspService, Server};
use clap::{command, Parser};
use std::net::SocketAddr;
mod backend;
#[derive(Parser, Debug)]
#[command(name = "sysy-lsp-server", version = "0.1.0", about = "SysY Language Server Protocol server")]
struct Args {
    /// Port to listen on for TCP connections, 0 for standard input/output mode
    #[arg(short, long, default_value = "0")]
    port: u16,
}

#[tokio::main]
async fn main() {
    use tokio::io::{AsyncRead, AsyncWrite};
    
    let args = Args::parse();
    let (stdin, stdout): (Box<dyn AsyncRead + Unpin + Send>, Box<dyn AsyncWrite + Unpin + Send>);
    
    if args.port > 0 {
        // TCP mode
        let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        println!("Listening on: {}", addr);
        let (stream, _) = listener.accept().await.unwrap();
        let (read, write) = tokio::io::split(stream);
        stdin = Box::new(read);
        stdout = Box::new(write);
    } else {
        // Standard input/output mode
        stdin = Box::new(tokio::io::stdin());
        stdout = Box::new(tokio::io::stdout());
    }


    let (service, socket) = LspService::new(|client| backend::Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
