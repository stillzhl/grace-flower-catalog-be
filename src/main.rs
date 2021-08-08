#[macro_use]
extern crate diesel;

use std::collections::HashMap;

use actix_web::{get, post, delete, middleware, web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use uuid::Uuid;

mod actions;
mod models;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[get("/flowers")]
async fn index_flowers(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    // use web::block to offload blocking Diesel code without blocking server thread
    let flowers = web::block(move || actions::get_flowers(&conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    Ok(HttpResponse::Ok().json(flowers))
}

/// Finds flower by UID
#[get("/flower/{flower_uid}")]
async fn get_flower(
    pool: web::Data<DbPool>,
    flower_uid: web::Path<Uuid>
) -> Result<HttpResponse, Error> {
    let flower_uid = flower_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let flower = web::block(move || actions::find_flower_by_uid(flower_uid, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    if let Some(flower) = flower {
        Ok(HttpResponse::Ok().json(flower))
    } else {
        let res = HttpResponse::NotFound()
            .body(format!("No flower found with uid: {}", flower_uid));
        Ok(res)
    }
}

/// Deletes flower by UID
#[delete("/flower/{flower_uid}")]
async fn delete_flower(
    pool: web::Data<DbPool>,
    flower_uid: web::Path<Uuid>
) -> Result<HttpResponse, Error> {
    let flower_uid = flower_uid.into_inner();
    let conn = pool.get().expect("couldn't get db connection from pool");

    let uid = web::block(move || actions::delete_flower_by_uid(flower_uid, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;
    let mut res: HashMap<String, String> = HashMap::new();
    res.insert("flw_uid".to_string(), uid);
    res.insert("status".to_string(), "deleted".to_string());
    Ok(HttpResponse::Ok().json(res))
}

/// Inserts new flower with attributes defined in form.
#[post("/flower")]
async fn add_flower(
    pool: web::Data<DbPool>,
    new_flower: web::Json<models::NewFlower>,
) -> Result<HttpResponse, Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");
    let flower_name = new_flower.flw_name.to_owned();
    let flower = web::block(move || actions::find_flower_by_name(&flower_name, &conn))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    if let Some(flower) = flower {
        Ok(HttpResponse::SeeOther().json(flower))
    } else {
        let conn = pool.get().expect("couldn't get db connection from pool");
        let flower = web::block(
            move || actions::insert_new_flower(new_flower, &conn)
            )
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                HttpResponse::InternalServerError().finish()
            })?;
        Ok(HttpResponse::Ok().json(flower))
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv::dotenv().ok();

    // set up database connection pool
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let bind = "127.0.0.1:8080";
    println!("Starting server at: {}", &bind);

    HttpServer::new(move || {
        App::new()
            // set up DB pool to be used with web::Data<Pool> extractor
            .data(pool.clone())
            .wrap(middleware::Logger::default())
            .service(index_flowers)
            .service(get_flower)
            .service(add_flower)
            .service(delete_flower)
    })
    .bind(&bind)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use std::{thread, time};

    #[actix_rt::test]
    async fn flower_routes() {
        std::env::set_var("RUST_LOG", "actix_web=debug");
        env_logger::init();
        dotenv::dotenv().ok();

        let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let manager = ConnectionManager::<SqliteConnection>::new(connspec);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let mut app = test::init_service(
            App::new()
                    .data(pool.clone())
                    .wrap(middleware::Logger::default())
                    .service(index_flowers)
                    .service(get_flower)
                    .service(add_flower)
                    .service(delete_flower)
        ).await;

        let req = test::TestRequest::post()
            .uri("/flower")
            .set_json(&models::NewFlower {
                flw_source: String::from("http://www.gardening.cornell.edu/homegardening/scenecea9.html"),
                flw_name: String::from("Astilbe, Chinese"),
                flw_img: String::from("http://www.gardening.cornell.edu/homegardening/images/garden/photos_garden/Saxifragaceae/Astilbe/chinensis/whole.jpg"),
                flw_family: String::from("Herbaceous Perennial Flower"),
                flw_season: String::from("Summer"),
                flw_desc: String::from("Deceptively delicate in appearance this moisture- and sem"),
                flw_site_chars: String::from("Prefers shady, moist sites, but needs good drainage over winter."),
                flw_plant_traits: String::from("Most varieties grow about 1.5 to 3 feet tall. 'Davidii' grows up to 6 feet tall."),
                flw_special_cons: String::from("bears ornamental fruit - Spent flowers are attractive and can be left until spring."),
                flw_growing_infos: String::from("Propagate by division"),
                flw_varieties: String::from("var. davidii grows 3 to 6 feet tall with deep green leaves heavily"),
            })
            .to_request();
        let resp: models::Flower = test::read_response_json(&mut app, req).await;
        assert_eq!(resp.flw_name, "Astilbe, Chinese");

        // Get a user
        let req = test::TestRequest::get()
            .uri(&format!("/flower/{}", resp.flw_id))
            .to_request();
        let resp: models::Flower = test::read_response_json(&mut app, req).await;
        assert_eq!(resp.flw_name, "Astilbe, Chinese");

        let ten_secs = time::Duration::from_secs(10);
        thread::sleep(ten_secs);
        // Delete new user from table
        // use crate::schema::flowers::dsl::*;
        // diesel::delete(flowers.filter(flw_id.eq(resp.flw_id)))
        //     .execute(&pool.get().expect("get db connection failed"))
        //     .expect("delete test user failed");
        let delete_req = test::TestRequest::delete()
            .uri(&format!("/flower/{}", resp.flw_id))
            .to_request();
        let delete_resp: HashMap<String, String> = test::read_response_json(&mut app, delete_req).await;
        assert_eq!(&resp.flw_id, delete_resp.get("flw_uid").unwrap());
        assert_eq!("deleted", delete_resp.get("status").unwrap());
    }
}
