use tokio_postgres::{NoTls, Error};
use tokio::time;
use std::time::Duration;

// Start pgsql container for testing:
// docker run --name pgsql11 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=secret -e POSTGRES_DB=testdb -p 5432:5432 -d postgres:11


const TOTAL_DURATION_SEC: u64 = 30;
const CONN_DURATION_SEC: u64 = 10;
const CONNECTIONS_RATE: u64 = 1000;  // pause in millis between new connections


async fn run_connection(n: i32) -> Result<(), Error> {

    println!("Initiating connection {}", n);

    // database connection
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=secret dbname=testdb", NoTls).await?;

    // spawn connection to run on its own
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let mut counter: u32 = 0;
    loop {
        counter = counter + 1;
        if counter % 10 == 0 {
            println!("Querying connection {}: {}", n, counter);
        }

        // sql statement
        let rows = client
            .query("SELECT $1::TEXT", &[&"hello world"])
            .await?;
            
        // get string back
        let value: &str = rows[0].get(0);
        assert_eq!(value, "hello world");

        // wait some time and do it again
        time::delay_for(Duration::from_millis(100)).await;
    }
}

async fn main_loop() {

    // loop creating new connections every CONNECTIONS_RATE milliseconds
    // that will live for CONN_DURATION_SEC seconds
    let mut counter = 0;
    loop {
        counter = counter + 1;
        println!("Creating a new connection... {}", counter);
        tokio::spawn(async move {
            println!("Preparing to run... {}", counter);
            if let Err(_) = time::timeout(
                Duration::from_secs(CONN_DURATION_SEC),
                run_connection(counter)
            ).await {
                println!("Killing connection... {}", counter);
            }
        });
        // wait a second before creating the next connection
        time::delay_for(Duration::from_millis(CONNECTIONS_RATE)).await;
    }

}

#[tokio::main]
async fn main() -> Result<(), Error> {

    println!("Initiating pgsql connections test...");

    // run the main loop for a certain period of time
    if let Err(_) = time::timeout(
        Duration::from_secs(TOTAL_DURATION_SEC),
        main_loop()
    ).await {
        println!("Done");
    }

    Ok(())
}
