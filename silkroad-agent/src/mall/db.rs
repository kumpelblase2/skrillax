use chrono::Utc;
use sqlx::PgPool;
use std::borrow::Borrow;
use std::ops::Add;

pub(crate) async fn insert_user_mall_key<T: Borrow<PgPool>>(
    pool: T,
    user_id: u32,
    server_id: u16,
    key: String,
    character_id: u32,
) {
    let expiry = Utc::now().add(chrono::Duration::minutes(15));

    let _ = sqlx::query!(
        "DELETE FROM user_item_mall WHERE user_id = $1 and server_id = $2",
        user_id as i32,
        server_id as i16
    )
    .execute(pool.borrow())
    .await;

    let _ = sqlx::query!(
        "INSERT INTO user_item_mall(user_id, character_id, server_id, key, expiry) VALUES($1, $2, $3, $4, $5)",
        user_id as i32,
        character_id as i32,
        server_id as i16,
        key,
        expiry
    )
    .execute(pool.borrow())
    .await;
}

pub(crate) async fn delete_expired_mall_keys<T: Borrow<PgPool>>(pool: T) {
    let _ = sqlx::query!("DELETE FROM user_item_mall WHERE expiry <= NOW()")
        .execute(pool.borrow())
        .await;
}
