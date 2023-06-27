use color_eyre::Report;
use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::{api::Api, core::ObjectMeta, Client};
use tracing::info;

fn main() -> Result<(), Report> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let client = Client::try_default().await?;

        let pods: Api<Pod> = Api::all(client.clone());
        let events: Api<Event> = Api::all(client);

        let pods = pods.list(&Default::default()).await?;

        for p in pods.items {
            match p {
                Pod {
                    metadata:
                        ObjectMeta {
                            name, namespace, ..
                        },
                    spec: Some(_sp),
                    status: Some(st),
                } => {
                    let PodStatus {
                        conditions,
                        container_statuses,
                        phase,
                        ..
                    } = st;
                    info!(
                        ?namespace,
                        ?name,
                        ?conditions,
                        ?container_statuses,
                        ?phase,
                        "pod status"
                    );
                }
                Pod {
                    metadata:
                        ObjectMeta {
                            name, namespace, ..
                        },
                    ..
                } => {
                    info!(?namespace, ?name, "no pod status");
                }
            }
        }

        let evs = events.list(&Default::default()).await?;
        info!(?evs, "events");

        Ok(())
    })
}
