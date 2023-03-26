use actix_web::{error, HttpResponse, get};
use actix_web::{web, Error, Responder, Result};
use serde::{Deserialize, Serialize};

use crate::prisma::post;
use crate::prisma::PrismaClient;

#[derive(Serialize, Deserialize, Debug)]
pub struct Post {
    id: String,
    name: String,
    author: String,
    views: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreatePost {
    name: String,
    author: String,
}

#[derive(Deserialize)]
pub struct DeletePost {
    id: String,
}

#[derive(Deserialize,Debug)]
struct FindPost {
    id: String,
}

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

async fn create_post(
    client: web::Data<PrismaClient>,
    dto: web::Json<CreatePost>,
) -> Result<impl Responder> {
    let result = client
        .post()
        .create(dto.name.to_owned(), dto.author.to_owned(), 0, vec![])
        .exec()
        .await
        .expect("Could not create a post due server error");
    Ok(web::Json(result))
}

async fn remove_post(
    client: web::Data<PrismaClient>,
    path_var: web::Path<DeletePost>,
) -> Result<impl Responder> {
    let remove_result = client
        .post()
        .delete(post::id::equals(path_var.id.to_owned()))
        .exec()
        .await
        .expect("Could not delete the selected entity");
    Ok(web::Json(Post {
        id: remove_result.id,
        name: remove_result.name,
        author: remove_result.author,
        views: remove_result.views,
    }))
}

#[get("/posts/id/{id}")]
async fn find_by_id(
    client: web::Data<PrismaClient>,
    path_var: web::Path<FindPost>,
) -> Result<HttpResponse, Error> {

    println!("path variable {:?}",path_var);
    let result = client
        .post()
        .find_unique(post::id::equals(path_var.id.to_owned()))
        .exec()
        .await
        .expect("Could not delete the selected entity");
    match result {
        None => Err(error::ErrorNotFound("Entity not found")),
        Some(dt) => {
            let body = Post {
                id: dt.id,
                name: dt.name,
                author: dt.author,
                views: dt.views,
            };

            Ok(HttpResponse::Ok().json(body))
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/posts")
            .route(web::get().to(get_all_posts))
            .route(web::post().to(create_post))
            .route(web::delete().to(remove_post)),
    );
    cfg.service(find_by_id);
}
