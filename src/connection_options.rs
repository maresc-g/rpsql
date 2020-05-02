use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConnectionOptions {
    pub dbname: String,
    pub host: String,
    pub port: String,
    pub user: String,
}

impl ConnectionOptions {
    pub fn to_connection_string(&self) -> String {
        format!("host={} port={} user={} dbname={}", self.host, self.port, self.user, self.dbname)
    }
}
