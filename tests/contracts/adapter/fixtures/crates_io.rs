use lfg::ecosystems::crates_io::{CratesIoCrateClient, CratesIoFetchError};

pub(crate) struct StaticCrateClient;

impl CratesIoCrateClient for StaticCrateClient {
    fn fetch_crate(&self, crate_name: &str) -> Result<String, CratesIoFetchError> {
        if crate_name != "serde" {
            return Err(CratesIoFetchError::Unavailable(crate_name.to_owned()));
        }

        Ok(r#"{
          "crate": { "id": "serde", "max_version": "1.0.1" },
          "versions": [
            {
              "num": "1.0.1",
              "created_at": "1970-01-02T00:00:00+00:00",
              "dl_path": "/api/v1/crates/serde/1.0.1/download"
            },
            {
              "num": "1.0.0",
              "created_at": "1970-01-01T00:00:00+00:00",
              "dl_path": "/api/v1/crates/serde/1.0.0/download"
            }
          ]
        }"#
        .to_owned())
    }
}
