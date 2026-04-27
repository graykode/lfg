use lfg::ecosystems::npm::{NpmFetchError, NpmPackumentClient};

pub(crate) struct StaticPackumentClient;

impl NpmPackumentClient for StaticPackumentClient {
    fn fetch_packument(&self, package_name: &str) -> Result<String, NpmFetchError> {
        if package_name != "left-pad" {
            return Err(NpmFetchError::Unavailable(package_name.to_owned()));
        }

        Ok(r#"{
          "name": "left-pad",
          "dist-tags": { "latest": "1.1.0" },
          "time": {
            "1.0.0": "1970-01-01T00:00:00.000Z",
            "1.1.0": "1970-01-02T00:00:00.000Z"
          },
          "versions": {
            "1.0.0": {
              "dist": { "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz" }
            },
            "1.1.0": {
              "dist": { "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz" }
            }
          }
        }"#
        .to_owned())
    }
}
