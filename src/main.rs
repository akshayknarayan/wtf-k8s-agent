use std::fmt;

use color_eyre::Report;
use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::{api::Api, core::ObjectMeta, Client};
use tracing::info;

enum SimplePodStatus {
    Pending, 
    Running,
    Succeeded,
    Failed,
    Unknown
}
impl fmt::Display for SimplePodStatus  {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                SimplePodStatus::Pending =>  write!(f, "Pending"),
                SimplePodStatus::Succeeded =>  write!(f, "Succeeded"),
                SimplePodStatus::Running =>  write!(f, "Running"),
                SimplePodStatus::Failed =>  write!(f, "Failed"),
                SimplePodStatus::Unknown =>  write!(f, "Unknown"),
            }
        }
}

async fn get_pod_status(pods: &Api<Pod>, pod_name: &String) -> Result<SimplePodStatus, String> {
    let pod_status_str = match pods.get_status(pod_name).await {
        Ok(it) => it,
        Err(e) => return Err(format!("{} {}", "pod not found".to_string(), e)),
    }.status.unwrap().phase.unwrap();
    match pod_status_str.as_ref() {
        "Succeeded" => Ok(SimplePodStatus::Succeeded),
        "Failed" => Ok(SimplePodStatus::Failed),
        "Running" => Ok(SimplePodStatus::Running),
        "Pending" => Ok(SimplePodStatus::Pending),
        "Unknown" => Ok(SimplePodStatus::Unknown),
        e => Err(format!("unknown status: {}",e)),
    }
}
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
        let res = get_pod_status(&pods, &p.metadata.name.unwrap()).await;
        match res {
        Ok(v) => println!("{}",v), 
        Err(e) => println!("{}", e),
        }
        }


        //let evs = events.list(&Default::default()).await?;
        //info!(?evs, "events");


        Ok(())
    })
}
