use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub radarr_url: String,
    pub radarr_api_key: String,
    // Transcoding thresholds
    pub min_file_size_gb: f64,
    pub min_bits_per_pixel: f64,
    pub min_compression_potential: f64,
}

impl Config {
    pub fn from_env() -> Self {
        let socket_dir = env::var("DB_SOCKET_DIR").ok();

        let name = env::var("DB_NAME").unwrap_or_else(|_| "rankode".to_string());

        let database_url = if let Some(dir) = socket_dir {
            let user = env::var("DB_USER").unwrap_or_else(|_| env::var("USER").unwrap_or_default());
            let password = env::var("DB_PASSWORD").unwrap_or_default();
            if password.is_empty() {
                format!("postgres://{}@/{}?host={}", user, name, dir)
            } else {
                format!("postgres://{}:{}@/{}?host={}", user, password, name, dir)
            }
        } else {
            let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = env::var("DB_PORT").unwrap_or_else(|_| "5433".to_string());
            let user = env::var("DB_USER").unwrap_or_else(|_| env::var("USER").unwrap_or_default());
            let password = env::var("DB_PASSWORD").unwrap_or_default();
            if password.is_empty() {
                format!("postgres://{}@{}:{}/{}", user, host, port, name)
            } else {
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    user, password, host, port, name
                )
            }
        };

        let radarr_url =
            env::var("RADARR_URL").expect("RADARR_URL environment variable must be set");
        let radarr_api_key =
            env::var("RADARR_API_KEY").expect("RADARR_API_KEY environment variable must be set");

        let min_file_size_gb = env::var("RANKODE_MIN_FILE_SIZE_GB")
            .unwrap_or_else(|_| "2.0".to_string())
            .parse()
            .unwrap_or(2.0);
        let min_bits_per_pixel = env::var("RANKODE_MIN_BITS_PER_PIXEL")
            .unwrap_or_else(|_| "0.04".to_string())
            .parse()
            .unwrap_or(0.04);
        let min_compression_potential = env::var("RANKODE_MIN_COMPRESSION_POTENTIAL")
            .unwrap_or_else(|_| "1.0".to_string())
            .parse()
            .unwrap_or(1.0);

        Config {
            database_url,
            radarr_url,
            radarr_api_key,
            min_file_size_gb,
            min_bits_per_pixel,
            min_compression_potential,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::from_env();
        assert!(config.database_url.contains("rankode"));
    }
}
