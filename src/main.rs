use color_eyre::Report;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::Api, Client};

mod health;
// TODO scope doesn't compile, don't really know how to fix the hashmap in-place iteration (over keys) & update
mod wtf_scope;

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let client = Client::try_default().await?;


        let mut a = wtf_scope::WtfScope::new(client);
        match a.populate_objects().await {
            Ok(_) => println!("{}", a),
            Err(e) => println!("error {}", e)
        } ;


        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");

        Ok(())
    })
}
