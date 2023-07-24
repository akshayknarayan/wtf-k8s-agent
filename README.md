## WTF k8s Agent
Welcome to the (very WIP) WTF agent for kubernetes!

Currently, initial health bit gathering and primitive health bit updates based on logs are supported. 
To get started, take a look at `main.rs`. Usage is supported as defined below:

- Construct a WtfScope object with `wtf_scope::WtfScope::new(client)`, where `client` is a `kube::Client`.

- call `monitor` in a tokio thread to keep status pod updated

- query logs with `get_object_health_bit`
