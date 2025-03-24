use chrono::prelude::*;
use clap::{Parser, Subcommand};
use dirs::data_dir;
use rusqlite::{params, Connection, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "did")]
#[clap(about = "A simple CLI to log your daily tasks")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[clap(name = "add", alias = "a")]
    Add { task: String },

    #[clap(name = "yesterday", alias = "y")]
    Yesterday,

    #[clap(name = "today", alias = "t")]
    Today,

    #[clap(name = "date")]
    Date { date: String },
}

fn get_db_path() -> PathBuf {
    if let Ok(env_path) = env::var("DID_DB_PATH") {
        PathBuf::from(env_path)
    } else {
        let mut path = data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("did");
        fs::create_dir_all(&path).expect("Failed to create data directory");
        path.push("did.db");
        path
    }
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

fn get_tasks_for_date(conn: &Connection, date: &str) -> Result<()> {
    let mut stmt = conn.prepare("SELECT date, task FROM tasks WHERE date LIKE ?1")?;
    let task_iter = stmt.query_map([format!("{}%", date)], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    println!("Tasks for {}", date);
    for task in task_iter {
        let (date, task_desc): (String, String) = task?;
        println!("{}: {}", date, task_desc);
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let db_path = get_db_path();
    println!("Using database at: {}", db_path.display());
    let conn = Connection::open(db_path)?;
    init_db(&conn)?;

    match args.command {
        Command::Add { task } => {
            add_task(&conn, &task)?;
            println!("Task '{}' logged successfully!", task);
        }
        Command::Yesterday => {
            let yesterday = Local::now()
                .checked_sub_signed(chrono::Duration::days(1))
                .expect("Failed to calculate yesterday")
                .format("%Y-%m-%d")
                .to_string();
            get_tasks_for_date(&conn, &yesterday)?;
        }
        Command::Today => {
            let today = Local::now().format("%Y-%m-%d").to_string();
            get_tasks_for_date(&conn, &today)?;
        }
        Command::Date { date } => {
            get_tasks_for_date(&conn, &date)?;
        }
    }
    Ok(())
}
