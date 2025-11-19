use std::env;

use redis::Client;

pub fn get_redis_client() -> redis::RedisResult<Client> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL most be set in .env");
    let client = Client::open(redis_url)?;
    Ok(client)
}
