use tokio_postgres::{NoTls, Error};
use tokio::time;
use std::time::Duration;
use futures::future::join_all;
use std::vec::Vec;

// Start pgsql container for testing:
// docker run --name pgsql11 -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=secret -e POSTGRES_DB=testdb -p 5432:5432 -d postgres:11


const DURATION_SEC: u64 = 15;
const N_CONNECTIONS: i32 = 10;


async fn run_connection(n: i32) -> Result<(), Error> {

    eprintln!("Initiating connection {}", n);

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

    let mut f = Vec::new();

    for n in 0..N_CONNECTIONS {
        f.push(
            time::timeout(
                Duration::from_secs(DURATION_SEC), 
                run_connection(n)
            )
        );
    }

    join_all(f).await;

    println!("Done");

    Ok(())
}
