use lfg::ecosystems::rubygems::{RubyGemsFetchError, RubyGemsVersionsClient};

pub(crate) struct StaticVersionsClient;

impl RubyGemsVersionsClient for StaticVersionsClient {
    fn fetch_versions(&self, gem_name: &str) -> Result<String, RubyGemsFetchError> {
        if gem_name != "rack" {
            return Err(RubyGemsFetchError::Unavailable(gem_name.to_owned()));
        }

        Ok(r#"[
          {
            "number": "3.0.0",
            "created_at": "1970-01-02T00:00:00.000Z"
          },
          {
            "number": "2.2.0",
            "created_at": "1970-01-01T00:00:00.000Z"
          }
        ]"#
        .to_owned())
    }
}
