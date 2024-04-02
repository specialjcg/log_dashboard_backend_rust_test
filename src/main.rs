use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use chrono::{DateTime, Local, NaiveDateTime, ParseError, TimeZone, Utc};
use regex::Regex;
use tokio_postgres::{Client, NoTls, types::ToSql};
use tokio_postgres::types::Timestamp;

mod test;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn parse_timestamp(timestamp_str: &str) -> Result<SystemTime, ParseError> {
    // Parse the timestamp string
    let naive_date_time = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S,%f")?;

    // Convert the NaiveDateTime to SystemTime
    let seconds = naive_date_time.timestamp();
    let subsec_nanos = naive_date_time.timestamp_subsec_nanos();

    // Create a SystemTime instance from seconds and nanoseconds since the Unix epoch
    let system_time = UNIX_EPOCH + Duration::new(seconds as u64, subsec_nanos);

    Ok(system_time)
}
#[derive(Debug, PartialEq)]
struct LogEntry {
    timestamp: std::time::SystemTime,
    severity: String,
    logger: String,
    message: String,
}

impl LogEntry {
    fn parse_log_entry(entry: &str) -> Option<Self> {
        let re = Regex::new(r"^(?P<timestamp>[^ ]+ [^ ]+)(\s+)(?P<severity>[A-Z]+)(\s+)(?P<logger>[^ ]+)(\s+)-(\s+)(?P<message>.*)").unwrap();

        if let Some(captures) = re.captures(entry) {
            let timestamp = match parse_timestamp(captures.name("timestamp").unwrap().as_str()) {
                Ok(parsed_time) => parsed_time,
                Err(err) => {
                    eprintln!("Failed to parse timestamp: {}", err);
                    // Here you can choose what value to use in case of an error,
                    // such as returning a default timestamp or exiting the function.
                    // For demonstration, let's use the current timestamp.
                    SystemTime::now()
                }
            };
            let severity = captures.name("severity").unwrap().as_str().to_string();
            let logger = captures.name("logger").unwrap().as_str().to_string();
            let message = captures.name("message").unwrap().as_str().to_string();
            Some(LogEntry {
                timestamp,
                severity,
                logger,
                message,
            })
        } else {
            None
        }
    }
}
impl FromStr for LogEntry {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match LogEntry::parse_log_entry(s) {
            Some(log_entry) => Ok(log_entry),
            None => Err(()),
        }
    }
}

async fn store_logs(client: web::Data<Arc<Client>>) -> impl Responder {
    // Open the log file
    let file = File::open("/home/jcgouleau/Bureau/Dior/ishelves/ishelves.log")
        .expect("Failed to open file");
    let reader = BufReader::new(file);



    let mut log_entries = Vec::new();
    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                if let Ok(log_entry) = LogEntry::from_str(&line) {
                    log_entries.push(log_entry);
                } else {
                    if let Some(mut last_value) = log_entries.pop() {
                        // Modify the last value (e.g., add 10 to it)
                        let new_last_value = last_value.message + line.as_str();
                        last_value.message = new_last_value;
                        // Push the modified value back onto the vector
                        log_entries.push(last_value);
                    } else {
                        println!("Vector is empty");
                    }
                    println!("Failed to parse log entry: {:?}", line);
                }
            }
            Err(err) => {
                eprintln!("Error reading line: {}", err);
            }
        }
    }

    for entry in &log_entries {
        // Convert NaiveDateTime to DateTime<Utc>
        // Convert NaiveDateTime to String
        let timestamp_param = entry.timestamp;

        // Prepare the SQL statement
        let statement = client.prepare("INSERT INTO logs (timestamp, severity, logger, message) VALUES ($1, $2, $3, $4)")
            .await
            .expect("Failed to prepare statement");

        // Execute the SQL statement with each log entry
        client.execute(&statement, &[&timestamp_param as &(dyn ToSql + Sync), &entry.severity, &entry.logger, &entry.message])
            .await
            .expect("Failed to execute statement");

    }
    // Return a success response
    HttpResponse::Ok().body("Logs stored in database")
}
// Define a custom error type
async fn establish_db_connection() -> Result<Client, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=pass123 port=5432",
        NoTls,
    )
        .await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });
    Ok(client)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Establish a connection to the PostgreSQL database
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=pass123 port=5432",
        NoTls,
    )
        .await
        .expect("Failed to connect to database");

    // Spawn a new asynchronous task to process the database connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Wrap the client in an Arc to allow sharing across threads
    let db_client = Arc::new(client);

    // Start the Actix web server
    HttpServer::new(move || {
        // Clone the Arc containing the client for each worker
        let db_client = db_client.clone();

        App::new()
            // Provide the client to each request handler
            .data(db_client.clone())
            .service(web::resource("/").route(web::get().to(hello)))
            .service(web::resource("/store_logs").route(web::get().to(store_logs)))
    })
        .bind("127.0.0.1:3000")?
        .run()
        .await
}

async fn hello(db_client: web::Data<Arc<Client>>) -> String {
    // Access the database client within the request handler
    // Perform database operations asynchronously here
    "Hello, world!".to_string()
}


