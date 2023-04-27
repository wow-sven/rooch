// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

pub mod actor;
pub mod api;
pub mod helper;
pub mod jsonrpc_types;
pub mod pb;
pub mod proxy;
pub mod response;
pub mod service;

// use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::RpcModule;
pub use pb::*;

use crate::api::account_api::AccountAPI;
use crate::api::RoochRpcModule;
use crate::os_service_client::OsServiceClient;
use crate::{
    actor::executor::ServerActor,
    os_service_server::OsServiceServer,
    proxy::ServerProxy,
    service::{OsSvc, RoochServer, RpcServiceServer},
    HelloRequest,
};
use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use coerce::actor::{system::ActorSystem, IntoActor};
use moveos::moveos::MoveOS;
use moveos_common::config::load_config;
use moveos_statedb::StateDB;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio::signal::ctrl_c;
use tokio::signal::unix::{signal, SignalKind};
use tonic::transport::Server;
use tonic::Request;
use tracing::info;

#[async_trait]
pub trait Execute {
    type Res;
    async fn execute(&self) -> Result<Self::Res>;
}

pub struct RpcModuleBuilder {
    module: RpcModule<()>,
}

impl RpcModuleBuilder {
    pub fn new() -> Self {
        Self {
            module: RpcModule::new(()),
        }
    }

    pub fn register_module<M: RoochRpcModule>(&mut self, module: M) -> Result<()> {
        Ok(self.module.merge(module.rpc())?)
    }
}

// Start json-rpc http server
pub async fn start_jsonrpc_http_server() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = load_config()?;

    let addr: SocketAddr = format!("{}:{}", config.http_server.host, config.http_server.port).parse()?;

    // let actor_system = ActorSystem::global_system();
    // let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    // let actor = ServerActor::new(moveos)
    //     .into_actor(Some("Server"), &actor_system)
    //     .await?;
    // let manager = ServerProxy::new(actor.into());
    // let rpc_service = RoochServer::new(manager);

    // let server = ServerBuilder::default().build(&addr).await?;
    let  rpc_module_builder = RpcModuleBuilder::new();
    let  rpc_module_builder = register_rpc_methods(rpc_module_builder);
    // let rpc_api = build_rpc_api(rpc_api);
    let server = ServerBuilder::default().build(&addr).await?;
    let methods_names = rpc_module_builder.module.method_names().collect::<Vec<_>>();

    // let handle = server.start(rpc_service.into_rpc())?;
    let handle = server.start(rpc_module_builder.module)?;

    info!("JSON-RPC HTTP Server start listening {:?}", addr);
    info!("Available JSON-RPC methods : {:?}", methods_names);

    let mut sig_int = signal(SignalKind::interrupt()).unwrap();
    let mut sig_term = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sig_int.recv() => info!("receive SIGINT"),
        _ = sig_term.recv() => info!("receive SIGTERM"),
        _ = ctrl_c() => info!("receive Ctrl C"),
    }

    handle.stop().unwrap();

    info!("Shutdown Sever");

    Ok(())
}

fn register_rpc_methods(mut rpc_module_builder: RpcModuleBuilder) -> RpcModuleBuilder {
    rpc_module_builder.register_module(AccountAPI::new()).unwrap();
    rpc_module_builder
}


fn _build_rpc_api<M: Send + Sync + 'static>(mut rpc_module: RpcModule<M>) -> RpcModule<M> {
    let mut available_methods = rpc_module.method_names().collect::<Vec<_>>();
    available_methods.sort();

    rpc_module
        .register_method("rpc_methods", move |_, _| {
            Ok(json!({
                "methods": available_methods,
            }))
        })
        .expect("infallible all other methods have their own address space");

    rpc_module
}

/// For grpc
#[derive(Debug, Parser)]
pub struct OsServer {
    #[clap(subcommand)]
    pub command: Command,
}

impl OsServer {
    pub async fn execute(&self) -> Result<()> {
        self.command.execute().await
    }
}

#[derive(Debug, Parser)]
pub enum Command {
    Say(SayOptions),
    Start(Start),
    StartHttp(StartHttp),
}

#[async_trait]
impl Execute for Command {
    type Res = ();
    async fn execute(&self) -> Result<()> {
        use Command::*;
        match self {
            Say(say) => {
                let _resp = say.execute().await;
                Ok(())
            }
            Start(start) => start.execute().await,
            StartHttp(start_http) => start_http.execute().await,
        }
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct SayOptions {
    #[clap(short, long, default_value = "hello")]
    name: String,
}

#[async_trait]
impl Execute for SayOptions {
    type Res = String;

    // Test server liveness
    async fn execute(&self) -> Result<Self::Res> {
        let url = load_config()?.rpc_server.url(false);
        println!("url: {:?}", url);

        let mut client = OsServiceClient::connect(url).await?;
        let request = Request::new(HelloRequest {
            name: self.name.clone(),
        });

        let response = client.echo(request).await?.into_inner();
        println!("{:?}", response);

        Ok(response.message)
    }
}

pub fn http_client(url: impl AsRef<str>) -> Result<HttpClient> {
    let client = HttpClientBuilder::default().build(url)?;
    Ok(client)
}

// pub async fn http_request<R, Params>(url: impl AsRef<str>, method: impl AsRef<str>, params: Params) -> Result<R, Error>
// where
// 		R: DeserializeOwned,
// 		Params: ToRpcParams + Send,
// {
//     let client = http_client(&url)?;

//     let response: Result<R, Error> = client.request(method.as_ref(), params)
//         .await;
//         // .map_err(|e| e.into());

//     response
// }

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct Start {}

#[async_trait]
impl Execute for Start {
    type Res = ();
    async fn execute(&self) -> Result<Self::Res> {
        // Ok(start_server().await?)
        Ok(start_grpc_server().await?)
    }
}

#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct StartHttp {}

#[async_trait]
impl Execute for StartHttp {
    type Res = ();
    async fn execute(&self) -> Result<Self::Res> {
        Ok(start_jsonrpc_http_server().await?)
    }
}


#[derive(Debug, Parser, Serialize, Deserialize)]
pub struct PublishPackage {
    // TODO Refactor fields to build module
    #[clap(short, long, default_value = ".")]
    pub module_path: String,
}

impl PublishPackage {
    // TODO compile module and serialize to bcs bytes
    pub fn compile_module_and_serailize(&self) -> Vec<u8> {
        self.module_path.as_bytes().to_vec()
    }
}

// Start grpc server
pub async fn start_grpc_server() -> Result<()> {
    let config = load_config()?;

    let addr: SocketAddr = format!("{}:{}", config.rpc_server.host, config.rpc_server.port).parse()?;

    let actor_system = ActorSystem::global_system();
    let moveos = MoveOS::new(StateDB::new_with_memory_store())?;
    let actor = ServerActor::new(moveos)
        .into_actor(Some("Server"), &actor_system)
        .await?;
    let manager = ServerProxy::new(actor.into());
    let svc = OsSvc::new(manager);
    let svc = OsServiceServer::new(svc);

    println!("gRPC Server start listening on {addr:?}");
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}

pub async fn start_server() -> Result<()> {
    start_grpc_server().await.unwrap();
    // start_jsonrpc_http_server().await.unwrap();

    tokio::spawn(async move {
        start_jsonrpc_http_server().await.unwrap();
    });

    Ok(())
}