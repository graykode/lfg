use lfg::ecosystems::pypi::{PypiFetchError, PypiProjectClient};

pub(crate) struct StaticProjectClient;

impl PypiProjectClient for StaticProjectClient {
    fn fetch_project(&self, package_name: &str) -> Result<String, PypiFetchError> {
        if package_name != "requests" {
            return Err(PypiFetchError::Unavailable(package_name.to_owned()));
        }

        Ok(r#"{
          "info": { "name": "requests", "version": "2.32.3" },
          "releases": {
            "2.32.2": [
              {
                "packagetype": "sdist",
                "url": "https://files.pythonhosted.org/packages/requests-2.32.2.tar.gz",
                "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
              }
            ],
            "2.32.3": [
              {
                "packagetype": "sdist",
                "url": "https://files.pythonhosted.org/packages/requests-2.32.3.tar.gz",
                "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
              }
            ]
          }
        }"#
        .to_owned())
    }
}
