use std::fmt;

use color_eyre::Report;
use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::{
    api::Api,
    core::{ObjectList, ObjectMeta},
    discovery::Scope,
    Client,
};
use tracing::info;

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

        let pods: Api<Pod> = Api::default_namespaced(client.clone());

        let _pods = match pods.list(&Default::default()).await {
            Ok(p) => p.into_iter(),
            Err(_) => Vec::<Pod>::new().into_iter(),
        };

        let mut scope = wtf_scope::WtfScope::new(pods);

        for p in _pods {
            match &p.metadata.name {
                Some(name) => match scope.add_to_scope(name).await {
                    Ok(_) => (),
                    Err(e) => println!("error adding {} to scope: {}", name, e),
                },
                None => (),
            };
        }

        match scope.update_scope().await {
            Ok(_) => (),
            Err(e) => println!("error updating scope: {}", e),
        };
        println!("{}", scope);

        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");

        Ok(())
    })
}
