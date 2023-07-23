use async_trait::async_trait;
use k8s_openapi::api::{apps::v1::{ReplicaSet, Deployment}, core::v1::Pod};
use std::fmt;

#[derive(Clone, Copy)]
pub enum HealthBit {
    Red,
    Green,
    Yellow,
    Unknown,
}

enum SimplePodStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

#[async_trait]
pub trait QueryableResource {
    async fn get_health_bit(&self) -> Result<HealthBit, String>;
}

impl fmt::Display for HealthBit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HealthBit::Red => write!(f, "Red"),
            HealthBit::Green => write!(f, "Green"),
            HealthBit::Yellow => write!(f, "Yellow"),
            HealthBit::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for SimplePodStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SimplePodStatus::Pending => write!(f, "Pending"),
            SimplePodStatus::Succeeded => write!(f, "Succeeded"),
            SimplePodStatus::Running => write!(f, "Running"),
            SimplePodStatus::Failed => write!(f, "Failed"),
            SimplePodStatus::Unknown => write!(f, "Unknown"),
        }
    }
}


// Get and map a pod's status to a health bit.
// TODO this function should porbably map podstatus-> healthbit and
// we'll have overloads (mapping other enums to healthbits) for other health metrics.
#[async_trait]
impl QueryableResource for Pod {
    async fn get_health_bit(&self) -> Result<HealthBit, String> {
        match get_pod_status(self).await? {
            SimplePodStatus::Succeeded => Ok(HealthBit::Green),
            SimplePodStatus::Running => Ok(HealthBit::Green),
            SimplePodStatus::Failed => Ok(HealthBit::Red),
            SimplePodStatus::Pending => Ok(HealthBit::Yellow),
            SimplePodStatus::Unknown => Ok(HealthBit::Red),
        }
    }
}

#[async_trait]
impl QueryableResource for ReplicaSet {
    async fn get_health_bit(&self) -> Result<HealthBit, String> {
        match self.status.clone() {
            Some(status) => {
                if status.ready_replicas.or(Some(0)) == Some(status.replicas) {
                    Ok(HealthBit::Green)
                } else {
                    Ok(HealthBit::Red)
                }
            }
            None => Err("no status!".to_string()),
        }
    }
}

#[async_trait]
impl QueryableResource for Deployment {
    async fn get_health_bit(&self) -> Result<HealthBit, String> {
        match self.status.clone() {
            Some(status) => {
                if status.unavailable_replicas.or(Some(0)) > Some(0) {
                    Ok(HealthBit::Red)
                } else {
                    Ok(HealthBit::Green)
                }
            }
            None => Err("no status!".to_string()),
        }
    }
}

// Query kubes API to get the string status of a pod, and map it to a SimplePodStatus enum.
async fn get_pod_status(pod: &Pod) -> Result<SimplePodStatus, String> {
    // TODO unwrap is bad
    match pod.status.clone().unwrap().phase.unwrap().as_ref() {
        "Succeeded" => Ok(SimplePodStatus::Succeeded),
        "Failed" => Ok(SimplePodStatus::Failed),
        "Running" => Ok(SimplePodStatus::Running),
        "Pending" => Ok(SimplePodStatus::Pending),
        "Unknown" => Ok(SimplePodStatus::Unknown),
        e => Err(format!("unknown status: {}", e)),
    }
}
