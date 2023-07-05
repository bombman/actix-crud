use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use sqlx::PgPool;
use sqlx::Row;
#[derive(Debug, Deserialize, Serialize, FromRow)]
struct UserProfile {
    id: i32,
    name: String,
    role_id: i32,
}

#[derive(Debug, FromRow)]
struct UserRole {
    id: i32,
    name: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/users")]
async fn get_users(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query(
        r#"
        SELECT u.id, u.name, r.id AS role_id, r.name AS role_name
        FROM userprofile AS u
        JOIN userrole AS r ON u.role_id = r.id
        "#,
    )
    .map(|row: sqlx::postgres::PgRow| UserProfile {
        id: row.get("id"),
        name: row.get("name"),
        role_id: row.get("role_id"),
    })
    .fetch_all(&**db_pool)
    .await;

    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

//Create
#[post("/users")]
async fn create_user(user: web::Json<UserProfile>, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query(
        r#"
        INSERT INTO userprofile (name, role_id)
        VALUES ($1, $2)
        RETURNING id, name, role_id
        "#,
    )
    .bind(&user.name)
    .bind(&user.role_id)
    .map(|row: sqlx::postgres::PgRow| UserProfile {
        id: row.get("id"),
        name: row.get("name"),
        role_id: row.get("role_id"),
    })
    .fetch_one(&**db_pool)
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

//Update
#[put("/users/{id}")]
async fn update_user(
    id: web::Path<i32>,
    user: web::Json<UserProfile>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let user_id = id.into_inner(); // แปลงเป็น i32
    let result = sqlx::query(
        r#"
        UPDATE userprofile
        SET name = $1, role_id = $2
        WHERE id = $3
        RETURNING id, name, role_id
        "#,
    )
    .bind(&user.name)
    .bind(&user.role_id)
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| UserProfile {
        id: row.get("id"),
        name: row.get("name"),
        role_id: row.get("role_id"),
    })
    .fetch_one(&**db_pool)
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

//Delete
#[delete("/users/{id}")]
async fn delete_user(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    let user_id = id.into_inner(); // แปลงเป็น i32
    let result = sqlx::query(
        r#"
        DELETE FROM userprofile
        WHERE id = $1
        RETURNING id, name, role_id
        "#,
    )
    .bind(user_id)
    .map(|row: sqlx::postgres::PgRow| UserProfile {
        id: row.get("id"),
        name: row.get("name"),
        role_id: row.get("role_id"),
    })
    .fetch_one(&**db_pool)
    .await;

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL not set"))
        .await
        .expect("Failed to connect to the database");

    // Create tables if they don't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS userprofile (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            role_id INT NOT NULL
        )
        "#,
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create table 'userprofile'");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS userrole (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL
        )
        "#,
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create table 'userrole'");

    // Insert sample data into userrole table
    sqlx::query(
        r#"
        INSERT INTO userrole (name)
        VALUES ('Admin'), ('User')
        "#,
    )
    .execute(&db_pool)
    .await
    .expect("Failed to insert sample data into userrole table");

    // Insert sample data into userprofile table
    sqlx::query(
        r#"
        INSERT INTO userprofile (name, role_id)
        VALUES ('John Doe', 1), ('Jane Smith', 2)
        "#,
    )
    .execute(&db_pool)
    .await
    .expect("Failed to insert sample data into userprofile table");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .service(get_users)
            .service(create_user)
            .service(update_user)
            .service(delete_user)
            .service(hello)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
