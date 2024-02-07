use crate::pb::hello_service_server::HelloService;
use crate::pb::{HelloReply, HelloRequest};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct HelloServer {}

#[tonic::async_trait]
impl HelloService for HelloServer {
    async fn hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };
        Ok(Response::new(reply))
    }
}
