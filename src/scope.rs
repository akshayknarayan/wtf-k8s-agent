use std::collections::HashMap;

use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::Api;

use crate::health::{HealthBit, get_health_bit};

    // TODO should a scope encompass generic resources, rather than just pods? 
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

    pub async fn add_to_scope(mut self: Scope, pod_name: &String) -> Result<(), String>{
     self.pods.insert(pod_name.to_string(), HealthBit::Unknown);
     Ok(())
    }

    // TODO error return
    pub async fn update_scope(self: Scope) -> () {
        for pod_name in self.pods.keys().into_iter() {
            self.query_pod(pod_name, Some(true));
        }
    }

    pub async fn query_pod(self: Scope, pod_name: &String, update: Option<bool>) -> Result<HealthBit, String> {
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
                // TODO prob don't use unwrap even though it's safe here
                Ok(self.pods.get(pod_name).unwrap().to_owned())
            }, 
            false => {
                Err("Pod not found within scope!".to_string())
            }
        }
    }

}
