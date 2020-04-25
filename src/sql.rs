use crate::connection_options::ConnectionOptions;
use postgres::{Client, NoTls, SimpleQueryMessage, SimpleQueryRow, Statement};
use postgres_types::Type;
use std::io::{self, Write};

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

pub fn handle_connection(connection_options: &ConnectionOptions) -> Result<(), String> {
    let mut client = _try_connect(&connection_options)?;

    let mut again = true;
    while again {
        let res = _loop(&mut client);
        if let Err(e) = res {
            eprintln!("{}", e);
        }
        else {
            again = res.unwrap();
        }
    }

    Ok(())
}

fn _try_connect(connection_options: &ConnectionOptions) -> Result<Client, String> {
    let res = Client::connect(connection_options.to_connection_string().as_str(), NoTls);
    if let Err(err) = res {
        return Err(format!("Error while trying to connect to server : {}", err));
    }
    Ok(res.unwrap())
}

fn _loop(mut client: &mut Client) -> Result<bool, String> {
    _display_prompt()?;
    let query_buffer = _get_query()?;
    let query = query_buffer.as_str();
    if query.is_empty() {
        println!("Exit");
        return Ok(false);
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
    _display_header(&columns);

    for row in &rows {
        for i in 0..row.len() {
            let val = row.get(i).unwrap_or_else(|| {
                "None"
            });

            print!("{:^width$}|", val, width = columns[i].max_size + ADDITIONAL_SPACES);
        }
        print!("\n");
    }

    Ok(true)
}

fn _display_prompt() -> Result<(), String> {
    print!("$> ");
    if let Err(err) = io::stdout().flush() {
        return Err(format!("Error printing prompt : {}", err));
    }
    Ok(())
}

fn _get_query() -> Result<String, String> {
    let mut buffer = String::new();
    if let Err(e) = io::stdin().read_line(&mut buffer) {
        return Err(format!("Error reading query : {}", e));
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
            SimpleQueryMessage::CommandComplete(i) => {
                println!("{} lines modified", i);
            }
             _ => {},
        }
    }
    Ok(rows)
}

fn _get_columns_from_result(result: &Statement, rows: &Vec<SimpleQueryRow>) -> Vec<ResultColumn> {
    let mut columns : Vec<ResultColumn> = Vec::new();
    for (i, c) in result.columns().iter().enumerate() {
        columns.push(ResultColumn::new());
        columns[i].name = c.name().to_string();
        columns[i].max_size = std::cmp::max(columns[i].max_size, c.name().len());
        columns[i].type_ = c.type_().clone();
    }
    for row in rows {
        for i in 0..row.len() {
            let val = row.get(i).unwrap_or_else(|| "None");
            columns[i].max_size = std::cmp::max(columns[i].max_size, val.len());
        }
    }
    columns
}

fn _display_header(columns: &Vec<ResultColumn>) {
    for c in columns {
        print!("{:^width$}|", c.name, width = c.max_size + ADDITIONAL_SPACES);
    }
    print!("\n");

    for c in columns {
        print!("{:-<1$}+", "", c.max_size + ADDITIONAL_SPACES);
    }
    print!("\n");
}