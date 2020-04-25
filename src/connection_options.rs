use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionOptions {
    pub dbname: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
}

impl ConnectionOptions {
    pub fn new() -> ConnectionOptions {
        ConnectionOptions {
            dbname : String::new(),
            host : String::new(),
            port : String::new(),
            user : String::new(),
            password: String::new(),
        }
    }

    pub fn to_connection_string(&self) -> String {
        format!("host={} port={} user={} dbname={} password={}", self.host, self.port, self.user, self.dbname, self.password)
    }
}
