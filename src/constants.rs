use crate::health::HealthBit;
use futures::executor::block_on;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static! {

    pub static ref log_mapping: Arc<RwLock<HashMap<String, HealthBit>>> = {
        let map = Arc::new(RwLock::new(HashMap::new()));
        block_on(map.write()).insert(r"Stopping container.*".to_string(), HealthBit::Red); // I think setting to red here (when
        block_on(map.write()).insert(r"Liveness probe failed.*".to_string(), HealthBit::Red); // I think setting to red here (when
        block_on(map.write()).insert(r"Deleted pod: (\S*)".to_string(), HealthBit::Red); // I think setting to red here (when
                                                         // deleted) is better than deleting bc else we might query in the future and be like
                                                         // where'd it go
        block_on(map.write()).insert(r"Pod sandbox changed, it will be killed and re-created.".to_string(), HealthBit::Yellow);
        // TODO we might need to construct some schema that allows certian 'pending' log messages
        // to be 'resolved' ie log("Pulling image") -> log("successfully pulled image")

        map
    };
}
