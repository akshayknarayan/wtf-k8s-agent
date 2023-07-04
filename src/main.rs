use std::fmt;

use color_eyre::Report;
use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::{api::Api, core::ObjectMeta, Client};
use tracing::info;

mod health;
mod scope;

// idea have a scope tracker that updates health bits for a scope on events related to pods within
// that scope

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let client = Client::try_default().await?;

        let pods: Api<Pod> = Api::default_namespaced(client.clone());

        let _pods = pods.list(&Default::default()).await?;

        for p in _pods.items {
        //let res = health::get_health_bits(&pods, &p.metadata.name.unwrap()).await;
        let res = health::query_pod_logs(&pods, &p.metadata.name.unwrap()).await;
        match res {
        Ok(_) => (), 
        Err(e) => println!("{}", e),
        }
        }


        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");


        Ok(())
    })
}
