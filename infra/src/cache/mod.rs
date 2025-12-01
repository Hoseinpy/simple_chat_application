use std::env;

use redis::Client;

pub fn get_redis_client() -> redis::RedisResult<Client> {
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL most be set in .env");
    let client = Client::open(redis_url)?;
    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::get_redis_client;
    use dotenvy::dotenv;

    #[test]
    fn test_get_redis_client() {
        dotenv().ok();
        assert!(get_redis_client().is_ok())
    }
}
