use diesel::prelude::*;
use uuid::Uuid;
use actix_web::web;

use crate::models;

/// Run query using Diesel to get all flowers.
pub fn get_flowers(
    conn: &SqliteConnection
) -> Result<Vec<models::Flower>, diesel::result::Error> {
    use crate::schema::flowers::dsl::*;

    let flower_list = flowers
        .limit(10)
        .load::<models::Flower>(conn)?;
    Ok(flower_list)
}

/// Run query using Diesel to find flower by uid and return it.
pub fn find_flower_by_uid(
    uid: Uuid,
    conn: &SqliteConnection
) -> Result<Option<models::Flower>, diesel::result::Error> {
    use crate::schema::flowers::dsl::*;

    let flower = flowers
        .filter(flw_id.eq(uid.to_string()))
        .first::<models::Flower>(conn)
        .optional()?;

    Ok(flower)
}

pub fn find_flower_by_name(
    nm: &str,
    conn: &SqliteConnection
) -> Result<Option<models::Flower>, diesel::result::Error> {
    use crate::schema::flowers::dsl::*;

    let flower = flowers
        .filter(flw_name.eq(nm))
        .first::<models::Flower>(conn)
        .optional()?;

    Ok(flower)
}

pub fn delete_flower_by_uid(
    uid: Uuid,
    conn: &SqliteConnection
) -> Result<String, diesel::result::Error> {
    use crate::schema::flowers::dsl::*;

    diesel::delete(flowers.filter(flw_id.eq(uid.to_string()))).execute(conn)?;
    Ok(uid.to_string())
}

/// Run query using Diesel to insert a new database row and return the result.
pub fn insert_new_flower(
    new_flower: web::Json<models::NewFlower>,
    conn: &SqliteConnection
) -> Result<models::Flower, diesel::result::Error> {
    // It is common when using Diesel with Actix web to import schema-related
    // modules inside a function's scope (rather than the normal module's scope)
    // to prevent import collisions and namespace pollution.
    use crate::schema::flowers::dsl::*;
    let new_flower = models::Flower {
        flw_id: Uuid::new_v4().to_string(),
        flw_source: new_flower.flw_source.clone(),
        flw_name: new_flower.flw_name.clone(),
        flw_img: new_flower.flw_img.clone(),
        flw_family: new_flower.flw_family.clone(),
        flw_season: new_flower.flw_season.clone(),
        flw_desc: new_flower.flw_desc.clone(),
        flw_site_chars: new_flower.flw_site_chars.clone(),
        flw_plant_traits: new_flower.flw_plant_traits.clone(),
        flw_special_cons: new_flower.flw_special_cons.clone(),
        flw_growing_infos: new_flower.flw_growing_infos.clone(),
        flw_varieties: new_flower.flw_varieties.clone(),

    };

    diesel::insert_into(flowers).values(&new_flower).execute(conn)?;

    Ok(new_flower)
}
