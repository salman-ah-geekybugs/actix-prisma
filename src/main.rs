use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use env_logger::Env;
mod prisma;
use prisma::PrismaClient;
use prisma_client_rust::{ NewClientError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Post {
    id: String,
    name: String,
    author: String,
    views: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CreatePost {
    name: String,
    author: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Server is running successfully")
}

#[get("/posts")]
async fn get_all_posts(client: web::Data<PrismaClient>) -> Result<impl Responder> {
    let all_post_entities = client
        .post()
        .find_many(vec![])
        .exec()
        .await
        .expect("Error occured while fetching posts from repository");
    let mapped_data: Vec<Post> = all_post_entities
        .into_iter()
        .map(|each_entity| Post {
            id: each_entity.id,
            name: each_entity.name,
            author: each_entity.author,
            views: each_entity.views,
        })
        .collect();
    Ok(web::Json(mapped_data))
}

#[post("/posts/create")]
async fn create_post(
    client: web::Data<PrismaClient>,
    dto: web::Json<CreatePost>,
) -> Result<impl Responder> {
    let result = client
        .post()
        .create(
            dto.name.to_owned(),
            dto.author.to_owned(),
            0,
            vec![],
        )
        .exec()
        .await
        .expect("Could not create a post due server error");
    Ok(web::Json(result))
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
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
            .service(hello)
            .service(echo)
            .service(get_all_posts)
            .service(create_post)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}
