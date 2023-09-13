use tonic::{metadata::MetadataValue, transport::Server, Request, Response, Status};
use dotenv::dotenv;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

fn check_auth(req: Request<()>) -> Result<Request<()>, Status> {
    let token_string = std::env::var("VERIFY_TOKEN").expect("You should set variables 'VERIFY_TOKEN'");
    let token: MetadataValue<_> = format!("Bearer {token_string}", token_string=token_string).parse().unwrap();

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let server = MyGreeter::default();
    dotenv().ok();

    Server::builder()
        .add_service(GreeterServer::with_interceptor(server, check_auth))
        .serve(addr)
        .await?;

    Ok(())
}
