use crate::clients::CommonParams;
use crate::clients::{Client, ValidatorDemand};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::ValidatorSet;
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::fs::create_dir;

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PrysmValidatorClient {
    #[serde(flatten)]
    pub common: CommonParams,
    pub validators: Option<usize>,
}

#[typetag::deserialize(name = "prysm_vc")]
impl Client for PrysmValidatorClient {
    fn add_to_node(
        &self,
        node: &NodeInfo,
        ctx: &mut SimulationContext,
        vs: &ValidatorSet,
    ) -> Result<Process, Error> {
        if self.common.executable.is_empty() {
            return Err(Error::MissingExecutable(String::from("prysm_vc")));
        }
        let dir = node.dir().join("prysm");
        let dir_str = dir.to_str().ok_or(Error::NonUTF8Path)?;
        if !dir.exists() {
            create_dir(&dir)?;
        }
        let wallet_dir = dir.join("wallet");
        let wallet_dir_str = wallet_dir.to_str().ok_or(Error::NonUTF8Path)?;

        let password_file = dir.join("passphrase");
        let password_file_str = password_file.to_str().ok_or(Error::NonUTF8Path)?;

        fs::rename(vs.base_path().join("prysm"), &wallet_dir)?;
        fs::write(&password_file, "12345678")?;

        let meta_dir = ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?;

        let args = format!(
            "--chain-config-file \"{meta_dir}/config.yaml\" \
                --accept-terms-of-use \
                --datadir \"{dir_str}\" \
                --wallet-dir \"{wallet_dir_str}\" \
                --wallet-password-file \"{password_file_str}\" \
                {}",
            self.common
                .arguments("--suggested-fee-recipient 0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134"),
        );

        Ok(Process {
            path: self.common.executable.clone().into(),
            args,
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn validator_demand(&self) -> ValidatorDemand {
        match self.validators {
            None => ValidatorDemand::Any,
            Some(num) => ValidatorDemand::Count(num),
        }
    }
}
