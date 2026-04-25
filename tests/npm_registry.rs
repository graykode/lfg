use lfg::npm_registry::{
    resolve_packument_releases, NpmRelease, NpmResolveError, ResolvedNpmReleases,
};

const PACKUMENT: &str = r#"{
  "name": "left-pad",
  "dist-tags": {
    "latest": "1.1.0"
  },
  "time": {
    "created": "2026-04-20T00:00:00.000Z",
    "modified": "2026-04-24T00:00:00.000Z",
    "1.0.0": "2026-04-21T00:00:00.000Z",
    "1.2.0": "2026-04-22T00:00:00.000Z",
    "1.1.0": "2026-04-23T00:00:00.000Z"
  },
  "versions": {
    "1.0.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.0.0.tgz"
      }
    },
    "1.1.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz"
      }
    },
    "1.2.0": {
      "dist": {
        "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.2.0.tgz"
      }
    }
  }
}"#;

#[test]
fn resolves_latest_target_and_previous_published_release() {
    assert_eq!(
        resolve_packument_releases(PACKUMENT, "left-pad"),
        Ok(ResolvedNpmReleases {
            package_name: "left-pad".to_owned(),
            target: NpmRelease {
                version: "1.1.0".to_owned(),
                published_at: "2026-04-23T00:00:00.000Z".to_owned(),
                tarball_url: "https://registry.npmjs.org/left-pad/-/left-pad-1.1.0.tgz".to_owned(),
            },
            previous: NpmRelease {
                version: "1.2.0".to_owned(),
                published_at: "2026-04-22T00:00:00.000Z".to_owned(),
                tarball_url: "https://registry.npmjs.org/left-pad/-/left-pad-1.2.0.tgz".to_owned(),
            },
        })
    );
}

#[test]
fn explicit_version_overrides_latest_dist_tag() {
    let resolved = resolve_packument_releases(PACKUMENT, "left-pad@1.2.0")
        .expect("explicit version should resolve");

    assert_eq!(resolved.target.version, "1.2.0");
    assert_eq!(resolved.previous.version, "1.0.0");
}

#[test]
fn reports_missing_previous_release() {
    let packument = r#"{
      "name": "one-version",
      "dist-tags": { "latest": "1.0.0" },
      "time": { "1.0.0": "2026-04-21T00:00:00.000Z" },
      "versions": {
        "1.0.0": {
          "dist": {
            "tarball": "https://registry.npmjs.org/one-version/-/one-version-1.0.0.tgz"
          }
        }
      }
    }"#;

    assert_eq!(
        resolve_packument_releases(packument, "one-version"),
        Err(NpmResolveError::MissingPreviousRelease)
    );
}

#[test]
fn reports_missing_target_version() {
    assert_eq!(
        resolve_packument_releases(PACKUMENT, "left-pad@9.9.9"),
        Err(NpmResolveError::MissingTargetVersion("9.9.9".to_owned()))
    );
}
