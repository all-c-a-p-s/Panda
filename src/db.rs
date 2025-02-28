use rusqlite::Connection;
use std::time::Instant;

pub fn inspect_db(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening database: {}", path);
    let conn = Connection::open(path)?;

    // Get total count of records
    println!("\nCounting total records (this might take a while)...");
    let start = Instant::now();

    // Get list of tables
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;

    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    println!("\nFound {} tables:", tables.len());
    for table in &tables {
        println!("\nTable: {}", table);

        // Get column info
        let mut pragma_stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;

        let columns: Vec<(i32, String, String)> = pragma_stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?, // column id
                    row.get(1)?, // column name
                    row.get(2)?, // column type
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        println!("Columns:");
        for (id, name, type_) in &columns {
            println!("  {}: {} ({})", id, name, type_);
        }

        // Get record count
        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |row| {
            row.get(0)
        })?;

        println!("\nTotal records: {}", count);

        // Sample some random records
        println!("\nSample records:");
        let query = format!("SELECT * FROM {} ORDER BY RANDOM() LIMIT 3", table);

        let mut stmt = conn.prepare(&query)?;
        let column_count = stmt.column_count();
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                values.push(match row.get::<_, rusqlite::types::Value>(i) {
                    Ok(val) => format!("{:?}", val),
                    Err(_) => "NULL".to_string(),
                });
            }
            Ok(values)
        })?;

        for row in rows {
            println!("  {:?}", row?);
        }

        // Get some basic statistics if there are numeric columns
        for (_, name, type_) in columns {
            if type_.to_lowercase().contains("int")
                || type_.to_lowercase().contains("float")
                || type_.to_lowercase().contains("real")
            {
                let stats_query = format!(
                    "SELECT MIN({}), MAX({}), AVG({}) FROM {}",
                    name, name, name, table
                );
                if let Ok(row) = conn.query_row(&stats_query, [], |row| {
                    Ok((
                        row.get::<_, f64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, f64>(2)?,
                    ))
                }) {
                    println!("\nStats for {}:", name);
                    println!("  Min: {}", row.0);
                    println!("  Max: {}", row.1);
                    println!("  Avg: {}", row.2);
                }
            }
        }
    }

    let duration = start.elapsed();
    println!("\nInspection completed in {:.2?}", duration);

    Ok(())
}
