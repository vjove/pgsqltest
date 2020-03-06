use tokio_postgres::{NoTls, Error};
use tokio::time;
use std::time::Duration;

// Start pgsql container for testing:
// docker run --name pgsql11 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=secret -e POSTGRES_DB=testdb -p 5432:5432 -d postgres:11


async fn run_connection() -> Result<(), Error> {

    // database connection
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=secret dbname=testdb", NoTls).await?;

    // spawn connection to run on its own
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    loop {
        // sql statement
        let rows = client
            .query("SELECT $1::TEXT", &[&"hello world"])
            .await?;
            
        // get string back
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");

        // wait some time and do it again
        time::delay_for(Duration::from_millis(10)).await;
    }
}


#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Initiating pgsql connections test...");

    if let Err(_) = time::timeout(
        Duration::from_secs(15), 
        run_connection()
    ).await {
        println!("Finishing...");
    }

    println!("Done");

    Ok(())
}
