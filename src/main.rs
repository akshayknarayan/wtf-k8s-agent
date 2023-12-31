use std::sync::Arc;

use color_eyre::Report;
use kube::Client;
use wtf_scope::WtfScope;

mod health;
mod constants;
// TODO scope doesn't compile, don't really know how to fix the hashmap in-place iteration (over keys) & update
mod wtf_scope;

#[tokio::main]
async fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let client = Client::try_default().await?;
    let scope = wtf_scope::WtfScope::new(client);
    let objects = Arc::clone(&scope.objects);
    tokio::spawn(monitor_client(scope));

    // TODO figure out why the minikube container deadlocks after quitting this app

    loop {
        println!("enter the object's id to get its status or 'exit' to exit");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        // I think the lock gets taken by the monitor service when we write a lock and never gets
        // released?
        match line.as_str().trim() {
            "exit" => {println!("exiting!"); break},
            object_name => match WtfScope::get_object_health_bit(Arc::clone(&objects), &object_name.to_string()).await {
                Ok(status) => println!("{} has most recent status {}, updated at timestamp {:?}", object_name, status.0, status.1),
                Err(e) => println!("{}", e),
            },
        }
        println!("")
    }

    Ok(())
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
