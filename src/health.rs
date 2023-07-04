use k8s_openapi::api::core::v1::{Event, Pod, PodStatus};
use kube::api::{Api, LogParams};
use std::fmt;

#[derive(Clone)]
pub enum HealthBit { 
    Red,
    Green,
    Yellow,
    Unknown
}

enum SimplePodStatus {
    Pending, 
    Running,
    Succeeded,
    Failed,
    Unknown
}
impl fmt::Display for HealthBit {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                HealthBit::Red =>  write!(f, "Red"),
                HealthBit::Green =>  write!(f, "Green"),
                HealthBit::Yellow =>  write!(f, "Yellow"),
                HealthBit::Unknown =>  write!(f, "Unknown"),
        }
    }
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
pub async fn get_health_bit(pods: &Api<Pod>, pod_name: &String) -> Result<HealthBit, String> {
    match get_pod_status(pods, pod_name).await? {
        SimplePodStatus::Succeeded => Ok(HealthBit::Green),
        SimplePodStatus::Running => Ok(HealthBit::Green),
        SimplePodStatus::Failed => Ok(HealthBit::Red),
        SimplePodStatus::Pending => Ok(HealthBit::Yellow),
        SimplePodStatus::Unknown => Ok(HealthBit::Red),
    }
}
pub async fn query_pod_logs(pods: &Api<Pod>, pod_name: &String) -> Result<(), String> {
    println!("{:?}", pods.logs(pod_name, &LogParams::default()).await.unwrap());
    Ok(())

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
