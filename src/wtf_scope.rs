use anyhow::anyhow;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::Time;
use k8s_openapi::chrono::{DateTime, Utc};
use k8s_openapi::{Metadata, Resource};
use pin_utils::pin_mut;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::fmt;

use std::str::FromStr;
use std::{borrow::BorrowMut, collections::HashMap};

use futures::stream::TryStreamExt;
use k8s_openapi::api::apps::v1::{Deployment, ReplicaSet};
use k8s_openapi::api::core::v1::{Event, Node, Pod, ReadServiceResponse};
use kube::runtime::{watcher, WatchStreamExt};
use kube::Api;
use kube::Client;
use tracing::info;

use crate::health::{query_pod_logs, HealthBit, QueryableResource};

#[derive(Clone)]
enum ResourceType {
    Pod,
    Deployment,
    ReplicaSet,
}

impl FromStr for ResourceType {
    type Err = ();

    fn from_str(input: &str) -> Result<ResourceType, Self::Err> {
        match input {
            "Pod" => Ok(ResourceType::Pod),
            "Deployment" => Ok(ResourceType::Deployment),
            "ReplicaSet" => Ok(ResourceType::ReplicaSet),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResourceType::Pod => write!(f, "Pod"),
            ResourceType::Deployment => write!(f, "Deployment"),
            ResourceType::ReplicaSet => write!(f, "ReplicaSet"),
        }
    }
}
// TODO should a scope encompass generic resources, rather than just pods?
// A scope of pods within a namespace, and a cache of their health bits.
pub struct WtfScope {
    objects: HashMap<String, ResourceStatus>,
    pod_api: Api<Pod>,
    //node_api: Api<Node>,
    deployment_api: Api<Deployment>,
    replica_set_api: Api<ReplicaSet>,
    client: Client,
}

#[derive(Clone)]
struct ResourceStatus {
    // Timestamped history of health bits, TODO this should probably be a circular buffer
    health_bit: Vec<(HealthBit, Time)>,
    object_type: ResourceType,
}

impl fmt::Display for WtfScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (object, status) in self.objects.clone() {
            s.push_str(
                &format!(
                    "object '{}' of type {} has health bit {}",
                    object,
                    status.object_type,
                    status.health_bit[status.health_bit.len() - 1].0
                )
                .to_string(),
            );
            s.push('\n');
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
            // node_api: Api::default_namespaced(client.clone()),
            replica_set_api: Api::default_namespaced(client.clone()),
            deployment_api: Api::default_namespaced(client.clone()),
            objects: HashMap::new(),
            client,
        }
    }

    async fn health_bit_from_event(ev: Event) -> HealthBit {
        match ev.type_.unwrap_or("Warning".to_string()).as_str() {
            "Normal" => HealthBit::Green,
            "Warning" => HealthBit::Yellow,
            _ => HealthBit::Yellow,
        }
    }

    async fn handle_new_log_event(&mut self, ev: Event) -> anyhow::Result<()> {
        info!(
            "Event: \"{}\" via {} {}",
            ev.message.unwrap_or("none".to_string()).trim(),
            ev.involved_object.kind.unwrap(),
            ev.involved_object.name.unwrap()
        );

        let resulting_status = WtfScope::health_bit_from_event(ev).await;

        let obj_type = ResourceType::from_str(&ev.involved_object.kind.unwrap()).unwrap();

        match ev.involved_object.name {
            Some(name) => match self.objects.entry(name.clone()) {
                Occupied(entry) => {
                    entry.into_mut().health_bit.push((
                        resulting_status,
                        ev.last_timestamp.unwrap_or(Time {
                            0: DateTime::<Utc>::MIN_UTC,
                        }),
                    ));
                    Ok(())
                }
                Vacant(entry) => {
                    match self.objects.insert(
                        name,
                        ResourceStatus {
                            health_bit: vec![(
                                resulting_status,
                                ev.last_timestamp.unwrap_or(Time {
                                    0: DateTime::<Utc>::MIN_UTC,
                                }),
                            )],
                            object_type: obj_type,
                        },
                    ) {
                        Some(_) => Ok(()),
                        None => Err(anyhow!(
                            "inserting a new status history failed on {} {}, somehow",
                            obj_type,
                            name
                        )),
                    }
                }
            },
            None => Err(anyhow!("warning: involved object has no name!")),
        }
    }

    // Update the cache of pod statuses within this scope.
    // TODO this should be called on an event-watch loop and maybe either update a pod or its
    // entire scope when an event (perhaps meeting some conditions) mentions a pod.
    pub async fn monitor(&mut self) -> anyhow::Result<()> {
        //tracing_subscriber::fmt::init();
        let client = Client::try_default().await?;

        let events: Api<Event> = Api::all(client);
        let wc = watcher::Config::default();
        let ew = watcher(events, wc).applied_objects();
        pin_mut!(ew);
        while let Some(event) = ew.try_next().await? {
            Self::handle_new_log_event(self, event);
        }
        Ok(())
    }

    pub async fn populate_objects(&mut self) -> Result<(), String> {
        // TODO cleaner
        let pods = match self.pod_api.list(&Default::default()).await {
            Ok(p) => p.into_iter(),
            Err(_) => Vec::<Pod>::new().into_iter(),
        };
        for pod in pods {
            match pod.metadata.name.clone() {
                Some(name) => self.objects.insert(
                    name,
                    ResourceStatus {
                        health_bit: vec![(
                            pod.get_health_bit().await?,
                            Time {
                                0: DateTime::<Utc>::MIN_UTC,
                            },
                        )],
                        object_type: ResourceType::Pod,
                    },
                ),
                None => return Err("pod has no name!".to_string()),
            };
        }

        let replica_sets = match self.replica_set_api.list(&Default::default()).await {
            Ok(p) => p.into_iter(),
            Err(_) => Vec::<ReplicaSet>::new().into_iter(),
        };
        for rset in replica_sets {
            match rset.metadata.name.clone() {
                Some(name) => self.objects.insert(
                    name,
                    ResourceStatus {
                        health_bit: vec![(
                            rset.get_health_bit().await?,
                            Time {
                                0: DateTime::<Utc>::MIN_UTC,
                            },
                        )],
                        object_type: ResourceType::ReplicaSet,
                    },
                ),
                None => return Err("pod has no name!".to_string()),
            };
        }

        println!("TODO deployment tracking");
        Ok(())
    }

    // Query the state of an object within this scope.
    // This result may be stale, and is likely to be so if the event watch loop is not running.
    pub async fn get_object_health_bit(
        &mut self,
        object_name: &String,
    ) -> Result<HealthBit, String> {
        match self.objects.get(object_name) {
            Some(status) => Ok(status.health_bit[status.health_bit.len() - 1].0),
            None => Err("Object  not found!".to_string()),
        }
    }
}
