use actix_web::middleware::Logger;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
mod post;
mod prisma;
use prisma::PrismaClient;
use prisma_client_rust::NewClientError;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Server is running successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client: Result<PrismaClient, NewClientError> = PrismaClient::_builder().build().await;
    let prisma_service = web::Data::new(client.unwrap());
    // let client = web::Data::new(prisma::new_client().await.unwrap());
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("a% %{User-Agent}i %s"))
            .app_data(prisma_service.clone())
            .configure(post::post::configure)
            .service(hello)
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
