use k8s_openapi::{Resource, Metadata};
use pin_utils::pin_mut;
use std::fmt;

use std::{borrow::BorrowMut, collections::HashMap};

use futures::stream::TryStreamExt;
use k8s_openapi::api::core::v1::{Event, Pod, ReadServiceResponse, Node};
use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet};
use kube::runtime::{watcher, WatchStreamExt};
use kube::Api;
use kube::Client;
use tracing::info;

use crate::health::{get_health_bit, query_pod_logs, HealthBit};

// TODO should a scope encompass generic resources, rather than just pods?
// A scope of pods within a namespace, and a cache of their health bits.
pub struct WtfScope {
    objects: HashMap<String, ResourceStatus>,
    pod_api: Api<Pod>,
    node_api: Api<Node>,
    deployment_api: Api<Deployment>,
    replica_pod_api: Api<ReplicaSet>,
    namespace: Option<String>
} 

struct ResourceStatus {
    health_bit: HealthBit,
}

impl fmt::Display for WtfScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (deployment, pods) in self.deployments {
            for (pod_name, value) in pods.pods.clone() {
                s.push_str(
                    &format!(
                        "deployment {} pod '{}' has status {}",
                        deployment, pod_name, value
                    )
                    .to_string(),
                );
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}
// havign a hard time figuring out a nice way to do this. Where are centralized lists like we can
// get in kubectl describe? everything except pods can only be queried with a name and a namespace
// need to populate objects before we can query them

impl WtfScope {
    pub fn new(client: Client) -> Self {
        Self {
            pod_api: Api::default_namespaced(client.clone()),
            node_api: Node::default_namespaced(client.clone()),
            replica_pod_api: ReplicaSet::default_namespaced(client.clone()),
            deployment_api: Deployment::default_namespaced(client.clone()),
            namespace,
        }
    }
    fn handle_new_log_event(ev: Event) -> anyhow::Result<()> {
        info!(
            "Event: \"{}\" via {} {}",
            ev.message.unwrap_or("none".to_string()).trim(),
            ev.involved_object.kind.unwrap(),
            ev.involved_object.name.unwrap()
        );
        Ok(())
    }

    pub async fn monitor() -> anyhow::Result<()> {
        //tracing_subscriber::fmt::init();
        let client = Client::try_default().await?;

        let events: Api<Event> = Api::all(client);
        let wc = watcher::Config::default();
        let ew = watcher(events, wc).applied_objects();
        pin_mut!(ew);
        while let Some(event) = ew.try_next().await? {
            Self::handle_new_log_event(event)?;
        }
        Ok(())
    }

    pub async fn update_scope(&mut self) {
    }

    // Update the cache of pod statuses within this scope.
    // TODO this should be called on an event-watch loop and maybe either update a pod or its
    // entire scope when an event (perhaps meeting some conditions) mentions a pod.
    pub async fn populate_deployment(&mut self, deployment_name: &String) -> Result<(), String> {

        let pods = match self.pod_api.list(&Default::default()).await {
            Ok(p) => p.into_iter(),
            Err(_) => Vec::<Pod>::new().into_iter(),
        };
        for pod in pods {
            match pod.metadata.name {
                Some(name) => self.objects.insert(name, WtfScope::ResourceStatus {health_bit: get_health_bit(&pod).await?}),
                None => return Err("pod has no name!".to_string())
            };
        }
        Ok(())
    }


    /*
    // Query the state of a pod within this scope. If `update` is false, return the cached status.
    // Otherwise, query the state using the API.
    pub async fn query_pod(
        &mut self,
        pod_name: &String,
        update: Option<bool>,
    ) -> Result<HealthBit, String> {
        match self.pods.get(pod_name).borrow_mut() {
            Some(pod) => {
                match get_health_bit(&self.namespace, &pod_name).await {
                    Ok(bit) => {
                        match update.unwrap_or(false) {
                            true => {
                                *pod = &bit.to_owned();
                            }
                            false => (),
                        };
                        Ok(bit)
                    }
                    Err(e) => Err(e),
                }

                // TODO just derive copy trait?
                // TODO prob don't use unwrap even though it's safe here (bc match on contains_key)
            }
            None => Err("Pod not found within scope!".to_string()),
        }
    }
    */
}
