use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, PostParams, ResourceExt, WatchEvent},
    client::Client, CustomResource, Resource,
    runtime::{
        controller::{Context, Controller, ReconcilerAction},
        events::{Event, EventType, Recorder, Reporter},
    },
};
use std::{sync::Arc};
use futures::{StreamExt, TryStreamExt};
use k8s_openapi::api::core::v1::Pod;
use serde_json::json;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use tracing::{debug, error, event, field, info, instrument, trace, warn, Level, Span};
use tokio::{
    sync::RwLock,
    time::{Duration, Instant},
};

/// Our Foo custom resource spec
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(kind = "Labeler", group = "nulllabeler.thenullchannel.dev", version = "v1", namespaced)]
#[kube(status = "LabelerStatus")]
pub struct LabelerSpec {
	label: String,
}

// LabelerStatus defines the observed state of Labeler
#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct LabelerStatus {
}

#[instrument(skip(ctx), fields(trace_id))]
async fn reconcile(labeler: Labeler, ctx: Context<Data>) -> Result<ReconcilerAction, Error> {
    Ok(ReconcilerAction {
        requeue_after: Some(Duration::from_secs(3600 / 2)),
    })
}
#[derive(Clone)]
struct Data {
    /// kubernetes client
    client: Client,
}

fn error_policy(error: &Error, _ctx: Context<Data>) -> ReconcilerAction {
    warn!("reconcile failed: {:?}", error);
    ReconcilerAction {
        requeue_after: Some(Duration::from_secs(360)),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },
    /// Error in user input or Echo resource definition, typically missing fields.
    #[error("Invalid Echo CRD: {0}")]
    UserInputError(String),
}


#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let client: Client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");
    
    let namespace = std::env::var("NAMESPACE").unwrap_or_else(|_| "default".into());
    // let pods: Api<Pod> = Api::namespaced(client, &namespace);
    
    //let labeler_api: Api<Labeler> = Api::namespaced(client,&namespace);
    let labeler_api = Api::<Labeler>::all(client.clone());

    let _r = labeler_api
    .list(&ListParams::default().limit(1))
    .await
    .expect("is the crd installed? please run: cargo run --bin crdgen | kubectl apply -f -");

    let context = Context::new(Data {
        client: client.clone(),
    });

    let the_labeler = Controller::new(labeler_api, ListParams::default())
            .run(reconcile, error_policy, context)
            .for_each(|reconciliation_result| async move {
                match reconciliation_result {
                    Ok(echo_resource) => {
                        println!("Reconciliation successful. Resource: {:?}", echo_resource);
                    }
                    Err(reconciliation_err) => {
                        eprintln!("Reconciliation error: {:?}", reconciliation_err)
                    }
                }
            })
            .await;
    /*
    let p: Pod = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": { "name": "blog" },
        "spec": {
            "containers": [{
              "name": "blog",
              "image": "clux/blog:0.1.0"
            }],
        }
    }))?;



    let pp = PostParams::default();
    match pods.create(&pp, &p).await {
        Ok(o) => {
            let name = o.name();
            assert_eq!(p.name(), name);
        }
        Err(kube::Error::Api(ae)) => assert_eq!(ae.code, 409), // if you skipped delete, for instance
        Err(e) => return Err(e.into()),                        // any other case is probably bad
    }
    */
    
    println!("Hello, world!");
    return Ok(());
}
