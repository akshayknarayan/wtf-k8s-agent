use std::fmt;

use color_eyre::Report;
use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::{api::Api, core::ObjectMeta, Client};
use tracing::info;

mod health;
// TODO scope doesn't compile, don't really know how to fix the hashmap in-place iteration (over keys) & update
// mod scope;

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let client = Client::try_default().await?;

        let pods: Api<Pod> = Api::default_namespaced(client.clone());

        // TODO fix list hang (and then panic on timeout) here when we have paused pods? 
        // is this inherent to the lib????
        // 2023-07-05T01:37:59.868290Z ERROR kube_client::client::builder: failed with error error trying to connect: deadline has elapsed
        let _pods = pods.list(&Default::default()).await?;

        for p in _pods.items {
        let res = health::get_health_bit(&pods, &p.metadata.name.clone().unwrap()).await;
        //let res = health::query_pod_logs(&pods, &p.metadata.name.unwrap()).await;
        match res {
        Ok(bit) => println!("Pod: {} Health bit: {}", p.metadata.name.unwrap(), bit),
        Err(e) => println!("{}", e),
        }
        }


        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");


        Ok(())
    })
}
