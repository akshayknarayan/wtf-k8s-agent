use std::fmt;

use std::{borrow::BorrowMut, collections::HashMap};

use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::Api;

use crate::health::{get_health_bit, HealthBit};

// TODO should a scope encompass generic resources, rather than just pods?
// A scope of pods within a namespace, and a cache of their health bits.
pub struct WtfScope {
    pods: HashMap<String, HealthBit>,
    namespace: Api<Pod>,
}
impl fmt::Display for WtfScope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for (pod_name, value) in self.pods.clone() {
            s.push_str(&format!("pod '{}' has status {}", pod_name, value).to_string());
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}
impl WtfScope {
    pub fn new(namespace: Api<Pod>) -> Self {
        Self {
            pods: HashMap::new(),
            namespace,
        }
    }

    // Add a pod to this scope.
    pub async fn add_to_scope(&mut self, pod_name: &String) -> Result<(), String> {
        self.pods.insert(pod_name.to_string(), HealthBit::Unknown);
        Ok(())
    }

    // Update the cache of pod statuses within this scope.
    // TODO this should be called on an event-watch loop and maybe either update a pod or its
    // entire scope when an event (perhaps meeting some conditions) mentions a pod.
    pub async fn update_scope(&mut self) -> Result<(), String> {
        for (pod_name, value) in self.pods.iter_mut() {
            match get_health_bit(&self.namespace, &pod_name).await {
                Ok(bit) => {
                    *value = bit;
                }
                // TODO more rust-y but also clean way to do this ?
                Err(e) => return Err(e),
            };
        }
        Ok(())
    }

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
}
