use std::sync::Arc;

use color_eyre::Report;
use kube::Client;
use wtf_scope::WtfScope;

mod health;
// TODO scope doesn't compile, don't really know how to fix the hashmap in-place iteration (over keys) & update
mod wtf_scope;

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    // TODO figure out how this works
    // until then, just gonna spawn (green, I think) thread for monitoring
    /*
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let client = Client::try_default().await?;

        let mut a = wtf_scope::WtfScope::new(client);
        match a.populate_objects().await {
            Ok(_) => println!("{}", a),
            Err(e) => println!("error {}", e),
        };




        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");

        Ok(())
    })
    */

    let client = Client::try_default().await?;
    let scope = wtf_scope::WtfScope::new(client);
    let objects = Arc::clone(&scope.objects);
    let t = tokio::spawn(monitor_client(scope));
    println!("bo");

    loop {
        println!("enter the object's id to get its status or 'exit' to exit");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        match line.as_str() {
            "exit" => break,
            _ => match WtfScope::get_object_health_bit(Arc::clone(&objects), &line).await {
                Ok(status) => println!("{} has most recent status {}", line, status),
                Err(e) => println!("{}", e),
            },
        }
        println!("input recieved");
    }
    match t.await {
        Ok(_) => {
            println!("amogu");
            Ok(())
        }
        Err(e) => Err(Report::new(e)),
    }
}
async fn monitor_client(mut scope: WtfScope) {
    match scope.populate_objects().await {
        Ok(_) => println!("{}", scope),
        Err(e) => println!("error {}", e),
    };
    match scope.monitor().await {
        Ok(_) => println!("{}", scope),
        Err(e) => println!("error {}", e),
    };
}
