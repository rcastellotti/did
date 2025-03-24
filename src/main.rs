use chrono::prelude::*;
use clap::Parser;
use dirs::data_dir;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "did")]
#[clap(about = "A simple CLI to log your daily tasks")]
struct Cli {
    task: String,
}

fn get_db_path() -> PathBuf {
    let mut path = data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("did");
    fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("did.db");
    path
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            date TEXT NOT NULL,
            task TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

fn add_task(conn: &Connection, task: &str) -> Result<()> {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO tasks (date, task) VALUES (?1, ?2)",
        params![now, task],
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let db_path = get_db_path();
    println!("using database at: {}", db_path.display());

    let conn = Connection::open(db_path)?;
    init_db(&conn)?;

    add_task(&conn, &args.task)?;
    println!("task logged successfully!");

    Ok(())
}
