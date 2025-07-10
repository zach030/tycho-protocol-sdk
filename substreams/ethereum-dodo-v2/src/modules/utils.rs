use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Params {
    pub dpp_factory: String,
    pub dsp_factory: String,
    pub dvm_factory: String,
    pub gsp_factory: String,
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        serde_qs::from_str(input).map_err(|e| anyhow!("Failed to parse query params: {}", e))
    }

    pub fn decode_addresses(&self) -> Result<([u8; 20], [u8; 20], [u8; 20], [u8; 20])> {
        let dpp_factory =
            hex::decode(&self.dpp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let dsp_factory =
            hex::decode(&self.dsp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let dvm_factory =
            hex::decode(&self.dvm_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        let gsp_factory =
            hex::decode(&self.gsp_factory).map_err(|e| anyhow!("Invalid factory hex: {}", e))?;
        if dpp_factory.len() != 20 || dsp_factory.len() != 20 || dvm_factory.len() != 20 || gsp_factory.len() != 20 {
            return Err(anyhow!("Addresses must be 20 bytes"));
        }
        Ok((
            dpp_factory.try_into().unwrap(),
            dsp_factory.try_into().unwrap(),
            dvm_factory.try_into().unwrap(),
            gsp_factory.try_into().unwrap(),
        ))
    }
}
