use crate::connection_options::ConnectionOptions;
pub use postgres::Client;
use postgres::{NoTls, SimpleQueryMessage, SimpleQueryRow, Statement};
use postgres_types::Type;

#[derive(Debug)]
struct ResultColumn {
    name: String,
    max_size: usize,
    type_: Type,
}

impl ResultColumn {
    pub fn new() -> ResultColumn {
        ResultColumn {
            name: String::new(),
            max_size: 0,
            type_: Type::TEXT,
        }
    }
}

const ADDITIONAL_SPACES: usize = 2;

pub fn try_connect(connection_options: &ConnectionOptions, password: Option<String>) -> Result<Client, String> {
    let connection_string = if let Some(password) = password {
        format!("{} password={}", connection_options.to_connection_string(), password)
    } else {
        connection_options.to_connection_string()
    };
    let res = Client::connect(connection_string.as_str(), NoTls);
    if let Err(err) = res {
        return Err(format!("Error while trying to connect to server : {}", err));
    }
    Ok(res.unwrap())
}

const BUILTIN_DESCRIBE_ALL: &str = "
  SELECT schemaname as schema, tablename as name, 'table' as type, tableowner as owner
    FROM pg_tables
   WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
   UNION
  SELECT schemaname as schema, sequencename as name, 'sequence' as type, sequenceowner as owner
    FROM pg_sequences
   WHERE schemaname NOT IN ('pg_catalog', 'information_schema')
   UNION
  SELECT schemaname as schema, viewname as name, 'view' as type, viewowner as owner
    FROM pg_views
   WHERE schemaname NOT IN('pg_catalog', 'information_schema')
ORDER BY name
;
";

pub fn handle_query(mut client: &mut Client, query: &str) -> Result<Vec<String>, String> {
    let mut query = query;
    if query.starts_with('\\') {
        query = BUILTIN_DESCRIBE_ALL;
    }
    let res_prepare = _prepare_query(&mut client, query);
    if let Err(e) = res_prepare {
        return Err(e);
    }
    let res_prepare = res_prepare.unwrap();

    let res_execute = _get_result_rows(&mut client, &query);
    if let Err(e) = res_execute {
        return Err(e);
    }

    let rows = res_execute.unwrap();
    let columns = _get_columns_from_result(&res_prepare, &rows);

    let mut buffer = Vec::new();
    _display_header(&columns, &mut buffer);

    for row in &rows {
        let mut tmp = String::new();
        for (i, c) in columns.iter().enumerate() {
            let val = row.get(i).unwrap_or_else(|| {
                "None"
            });
            tmp.push_str(format!("{:^width$}|", val, width = c.max_size + ADDITIONAL_SPACES).as_str());
        }
        buffer.push(tmp);
    }

    Ok(buffer)
}

fn _prepare_query(client: &mut Client, query: &str) -> Result<Statement, String> {
    let res = client.prepare(query);
    if let Err(e) = res {
        return Err(format!("Error while preparing query : {}", e));
    }
    Ok(res.unwrap())
}

fn _get_result_rows(client: &mut Client, query: &str) -> Result<Vec<SimpleQueryRow>, String> {
    let mut rows = Vec::new();
    let res = client.simple_query(query);
    if let Err(e) = res {
        return Err(format!("Error while executing query : {}", e));
    }
    for message in res.unwrap() {
        match message {
            SimpleQueryMessage::Row(row) => {
                rows.push(row);
            }
            SimpleQueryMessage::CommandComplete(_i) => {
                // println!("{} lines modified", i);
            }
             _ => {},
        }
    }
    Ok(rows)
}

fn _get_columns_from_result(result: &Statement, rows: &[SimpleQueryRow]) -> Vec<ResultColumn> {
    let mut columns : Vec<ResultColumn> = Vec::new();
    for (i, c) in result.columns().iter().enumerate() {
        columns.push(ResultColumn::new());
        columns[i].name = c.name().to_string();
        columns[i].max_size = std::cmp::max(columns[i].max_size, c.name().len());
        columns[i].type_ = c.type_().clone();
    }
    for row in rows {
        for (i, c) in columns.iter_mut().enumerate() {
            let val = row.get(i).unwrap_or_else(|| "None");
            c.max_size = std::cmp::max(c.max_size, val.len());
        }
    }
    columns
}

fn _display_header(columns: &[ResultColumn], buffer: &mut Vec<String>) {
    let mut tmp = String::new();
    for c in columns {
        tmp.push_str(format!("{:^width$}|", c.name, width = c.max_size + ADDITIONAL_SPACES).as_str());
    }
    buffer.push(tmp);

    let mut tmp = String::new();
    for c in columns {
        tmp.push_str(format!("{:-<1$}+", "", c.max_size + ADDITIONAL_SPACES).as_str());
    }
    buffer.push(tmp);
}
