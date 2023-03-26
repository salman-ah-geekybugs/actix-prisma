use crate::prisma::PrismaClient;
use actix_web::error::ErrorInternalServerError;
use actix_web::{error, get, post, HttpResponse};
use actix_web::{web, Error};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{self, Argon2, PasswordHasher};
use serde::{Deserialize, Serialize};
use validator::Validate;

// const salt: &str = "argonHash";

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: i32,
    email: String,
    password: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct CreateUser {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserDto {
    id: i32,
    email: String,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
struct PaginationDto {
    page: i32,
    limit: i32,
}

#[get("/users")]
async fn find_all_paginated(
    client: web::Data<PrismaClient>,
    query_params: web::Query<PaginationDto>,
) -> Result<HttpResponse, Error> {
    let skipped_rec = query_params.limit * query_params.page;

    let user_entities = client
        .user()
        .find_many(vec![])
        .take(query_params.limit.into())
        .skip(skipped_rec.into())
        .exec()
        .await
        .expect("Failed to fetch users from database");

    let mapped_data: Vec<UserDto> = user_entities
        .iter()
        .map(|each_entity| UserDto {
            id: each_entity.id,
            email: each_entity.email.to_owned(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(mapped_data))
}

#[post("/user")]
async fn create_user(
    client: web::Data<PrismaClient>,
    body: web::Json<CreateUser>,
) -> Result<HttpResponse, Error> {
    let salt: SaltString = SaltString::generate(&mut OsRng);
    let hasher: Argon2 = Argon2::default();
    let hashed_password = hasher
        .hash_password(body.password.as_bytes(), &salt)
        .expect("Could not generate salt");
    println!("hashed password {}", hashed_password);
    let created_user = client
        .user()
        .create(body.email.to_owned(), hashed_password.to_string(), vec![])
        .exec()
        .await;
    match created_user {
        Ok(res) => Ok(HttpResponse::Created().json(UserDto {
            email: body.email.to_owned(),
            id: res.id,
        })),
        Err(e) => Err(ErrorInternalServerError(format!(
            "Error occured while creating user {}",
            e
        ))),
    }
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_paginated);
    cfg.service(create_user);
}
