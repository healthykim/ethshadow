use crate::config::ethshadow::DEFAULT_GENESIS_GEN_IMAGE;
use crate::config::FullConfig;
use crate::network_graph::{generate_network_graph, GeneratedNetworkGraph};
use crate::node::NodeManager;
use crate::validators::ValidatorManager;
use log::{debug, info};
use serde_yaml::Value;
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::fs::{create_dir, remove_dir_all, File};
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

mod clients;
pub mod config;
mod error;
pub mod genesis;
mod gml;
pub mod network_graph;
pub mod node;
mod utils;
pub mod validators;

// reexports
pub use crate::error::Error;

type CowStr = Cow<'static, str>;

pub struct ShadowInvocation {
    command: Command,
    args: Vec<OsString>,
}

impl ShadowInvocation {
    fn new<S: AsRef<OsStr>>(executable: S, args: Vec<OsString>) -> ShadowInvocation {
        ShadowInvocation {
            command: Command::new(executable),
            args,
        }
    }

    pub fn with_user_args<I, S>(&mut self, user_args: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(user_args);
    }

    pub fn command(mut self) -> Command {
        self.command.args(self.args);
        self.command
    }
}

pub fn generate<T: TryInto<FullConfig, Error = Error>>(
    config: T,
    output_path: &Path,
    use_existing_dir: bool,
) -> Result<ShadowInvocation, Error> {
    debug!("Reading config file");
    // get the config and extend it with our supported builtins
    let FullConfig {
        mut ethshadow_config,
        mut shadow_config,
    } = config.try_into()?;
    ethshadow_config.add_default_builtins();
    shadow_config.apply_defaults(ethshadow_config.minimum_latency())?;

    debug!("Creating output directory");
    if let Err(e) = create_dir(output_path) {
        if e.kind() == ErrorKind::AlreadyExists {
            if !use_existing_dir {
                // TODO: remove this config (use_existing_dir is never true)
                remove_dir_all(output_path).unwrap();
                create_dir(output_path).unwrap();
            }
        } else {
            return Err(e.into());
        }
    };
    let mut output_path = output_path.canonicalize()?;

    debug!("Desugaring node config");
    let nodes = ethshadow_config.desugar_nodes()?;

    debug!("Computing validators");
    let validators = ValidatorManager::new(&ethshadow_config, &nodes, &output_path)?;

    info!("Generating genesis information");
    genesis::write_config(
        &ethshadow_config.genesis,
        validators.total_count(),
        output_path.clone(),
    )?;
    genesis::generate(
        ethshadow_config
            .genesis
            .generator_image
            .as_deref()
            .unwrap_or(DEFAULT_GENESIS_GEN_IMAGE),
        &output_path,
    )?;

    debug!("Generating network graph");
    let GeneratedNetworkGraph {
        gml,
        mut network_graph,
    } = generate_network_graph(&ethshadow_config)?;
    shadow_config.set_network(gml, false)?;

    // postprocessing given shadow config values: overwrite string network ids
    for host in shadow_config.hosts_mut()? {
        let mapping = host?;
        if mapping.get("network_node_id").is_some() {
            return Err(Error::InvalidShadowHost);
        }
        let Value::String(location) = mapping.remove("location").ok_or(Error::InvalidShadowHost)?
        else {
            return Err(Error::ExpectedOtherType("location".to_string()));
        };
        let Value::String(reliability) = mapping
            .remove("reliability")
            .ok_or(Error::InvalidShadowHost)?
        else {
            return Err(Error::ExpectedOtherType("reliability".to_string()));
        };
        let node = network_graph.assign_network_node(&location, &reliability)?;
        mapping.insert("network_node_id".into(), node.id().into());
    }

    info!("Generating nodes");
    let mut node_manager = NodeManager::new(
        output_path.clone(),
        &nodes,
        &mut shadow_config,
        network_graph,
        validators,
    );
    node_manager.generate_nodes()?;

    info!("Writing finished configuration");
    output_path.push("shadow.yaml");
    serde_yaml::to_writer(File::create_new(&output_path)?, &shadow_config.0)?;
    let config_path = output_path.as_os_str().to_owned();
    output_path.pop();

    output_path.push("shadow");
    Ok(ShadowInvocation::new(
        ethshadow_config.shadow_path.as_deref().unwrap_or("shadow"),
        vec!["-d".into(), output_path.into_os_string(), config_path],
    ))
}
