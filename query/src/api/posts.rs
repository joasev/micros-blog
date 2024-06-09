use actix_web::{get, web, HttpResponse, Responder};
use futures::stream::TryStreamExt;
use crate::{models::blog::Post, AppState};


#[get("/posts")]
async fn get_posts(data: web::Data<AppState>) -> impl Responder {
    let posts_collection = data.db_query.collection::<Post>("posts");

    let cursor = match posts_collection.find(None, None).await {
        Ok(cursor) => cursor,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to find posts: {}", e)),
    };

    let posts: Vec<Post> = match cursor.try_collect().await {
        Ok(posts) => posts,
        Err(e) => return HttpResponse::InternalServerError().body(format!("Failed to collect posts: {}", e)),
    };

    HttpResponse::Ok().json(posts)

}