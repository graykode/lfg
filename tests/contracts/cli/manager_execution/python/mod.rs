mod pip;
mod uv;

const OLD_PYTHON_PROJECT: &str = r#"{
  "info": { "name": "old-python-package", "version": "1.1.0" },
  "releases": {
    "1.0.0": [
      {
        "packagetype": "sdist",
        "url": "https://files.pythonhosted.org/packages/old-python-package-1.0.0.tar.gz",
        "upload_time_iso_8601": "1970-01-01T00:00:00.000000Z"
      }
    ],
    "1.1.0": [
      {
        "packagetype": "sdist",
        "url": "https://files.pythonhosted.org/packages/old-python-package-1.1.0.tar.gz",
        "upload_time_iso_8601": "1970-01-02T00:00:00.000000Z"
      }
    ]
  }
}"#;
