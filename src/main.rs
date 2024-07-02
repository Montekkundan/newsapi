use postgres::{Client, NoTls};
use postgres::Error as PostgresError;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::env;

#[macro_use]
extern crate serde_derive;

// Model: Article struct with id, title, content, source
#[derive(Serialize, Deserialize)]
struct Article {
    id: Option<i32>,
    title: String,
    content: String,
    source: String,
}

// Constants
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

// Main function
fn main() {
    // Set database
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    if let Err(e) = set_database(&db_url) {
        println!("Error: {}", e);
        return;
    }

    // Start server and print port
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Server started at port 8080");

    // Handle the client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &db_url);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

// Handle client function
fn handle_client(mut stream: TcpStream, db_url: &str) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /articles") => handle_post_request(r, db_url),
                r if r.starts_with("GET /articles/") => handle_get_request(r, db_url),
                r if r.starts_with("GET /articles") => handle_get_all_request(r, db_url),
                r if r.starts_with("PUT /articles/") => handle_put_request(r, db_url),
                r if r.starts_with("DELETE /articles/") => handle_delete_request(r, db_url),
                _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

// Controllers

// Handle POST request function
fn handle_post_request(request: &str, db_url: &str) -> (String, String) {
    match (get_article_request_body(request), Client::connect(db_url, NoTls)) {
        (Ok(article), Ok(mut client)) => {
            client
                .execute(
                    "INSERT INTO articles (title, content, source) VALUES ($1, $2, $3)",
                    &[&article.title, &article.content, &article.source]
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "Article created".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// Handle GET request function
fn handle_get_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) =>
            match client.query_one("SELECT * FROM articles WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let article = Article {
                        id: row.get(0),
                        title: row.get(1),
                        content: row.get(2),
                        source: row.get(3),
                    };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&article).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "Article not found".to_string()),
            }

        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// Handle GET all request function
fn handle_get_all_request(_request: &str, db_url: &str) -> (String, String) {
    match Client::connect(db_url, NoTls) {
        Ok(mut client) => {
            let mut articles = Vec::new();

            for row in client.query("SELECT * FROM articles", &[]).unwrap() {
                articles.push(Article {
                    id: row.get(0),
                    title: row.get(1),
                    content: row.get(2),
                    source: row.get(3),
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&articles).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// Handle PUT request function
fn handle_put_request(request: &str, db_url: &str) -> (String, String) {
    match (
        get_id(request).parse::<i32>(),
        get_article_request_body(request),
        Client::connect(db_url, NoTls),
    ) {
        (Ok(id), Ok(article), Ok(mut client)) => {
            client
                .execute(
                    "UPDATE articles SET title = $1, content = $2, source = $3 WHERE id = $4",
                    &[&article.title, &article.content, &article.source, &id]
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "Article updated".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// Handle DELETE request function
fn handle_delete_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM articles WHERE id = $1", &[&id]).unwrap();

            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Article not found".to_string());
            }

            (OK_RESPONSE.to_string(), "Article deleted".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

// Set database function
fn set_database(db_url: &str) -> Result<(), PostgresError> {
    // Connect to database
    let mut client = Client::connect(db_url, NoTls)?;

    // Create table
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id SERIAL PRIMARY KEY,
            title VARCHAR NOT NULL,
            content TEXT NOT NULL,
            source VARCHAR NOT NULL
        )"
    )?;
    Ok(())
}

// Get ID function
fn get_id(request: &str) -> &str {
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// Deserialize article from request body
fn get_article_request_body(request: &str) -> Result<Article, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
