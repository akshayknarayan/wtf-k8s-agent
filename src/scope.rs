use std::collections::HashMap;

use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::Api;

use crate::health::{HealthBit, get_health_bit};

// TODO should a scope encompass generic resources, rather than just pods? 
// A scope of pods within a namespace, and a cache of their health bits.
pub struct Scope {
    pods: HashMap<String, HealthBit>,
    namespace: Api<Pod>,
}
impl Scope {
    pub fn new(namespace: Api<Pod>) -> Self {
        Self {
            pods: HashMap::new(),
            namespace,
        }
    }

    // Add a pod to this scope.
    pub async fn add_to_scope(mut self: Scope, pod_name: &String) -> Result<(), String>{
     self.pods.insert(pod_name.to_string(), HealthBit::Unknown);
     Ok(())
    }

    // Update the cache of pod statuses within this scope.
    // TODO this should be called on an event-watch loop and maybe either update a pod or its
    // entire scope when an event (perhaps meeting some conditions) mentions a pod.
    pub async fn update_scope(self: &Scope) -> () {
        for (pod_name, _) in self.pods.clone() {
            self.query_pod(&pod_name.to_owned(), Some(true)).await;
        }
    }

    // Query the state of a pod within this scope. If `update` is false, return the cached status.
    // Otherwise, query the state using the API.
    pub async fn query_pod(mut self: Scope, pod_name: &String, update: Option<bool>) -> Result<HealthBit, String> {
        match self.pods.contains_key(pod_name) {
            true => {
                match update.unwrap_or(false) {
                    true => {
                        match get_health_bit(&self.namespace, &pod_name).await {
                            Ok(bit) => {self.pods.insert(pod_name.to_string(), bit); Ok(())},
                            Err(e) => Err(e)
                        };
                    },
                    false => ()
                }

                // TODO just derive copy trait?
                // TODO prob don't use unwrap even though it's safe here (bc match on contains_key)
                Ok(self.pods.get(pod_name).unwrap().to_owned())
            }, 
            false => {
                Err("Pod not found within scope!".to_string())
            }
        }
    }

}
