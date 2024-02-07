mod pb;
use pb::hello_service_server::HelloServiceServer;
use tonic::transport::Server;
mod server;
use server::HelloServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = HelloServer::default();
    println!("GreeterServer listening on {}", addr);
    Server::builder()
        .add_service(HelloServiceServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
