//! Integration tests driving the built binary against temporary config
//! directories. These never touch the real `~/.config/opencode`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;

/// A temporary opencode config directory with a fixture config in it.
struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("could not create a temp dir");
        let fx = Fixture { dir };
        fs::write(fx.config_path(), fx.sample_config()).expect("could not write fixture config");
        fx
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn config_path(&self) -> PathBuf {
        self.dir.path().join("opencode.json")
    }

    fn profiles_dir(&self) -> PathBuf {
        self.dir.path().join("profiles")
    }

    fn sample_config(&self) -> String {
        // Note the deliberate key order: `model` before `agent`, and `mcp`
        // last. A save that reorders these is a regression, and
        // `saving_preserves_agent_key_order` catches it.
        //
        // The `provider` block is what makes a model reference resolvable.
        // Without it, every `use` fails validation for the right reason but
        // the wrong test: an "unknown provider" test would pass because the
        // config declares no providers at all, not because it caught the
        // specific bad reference.
        r#"{
  "model": "local/one",
  "agent": {
    "build": {
      "model": "local/one"
    },
    "writer": {
      "model": "remote/writer"
    }
  },
  "provider": {
    "local": {
      "models": {
        "one": {},
        "two": {}
      }
    },
    "remote": {
      "models": {
        "writer": {},
        "big": {}
      }
    }
  },
  "mcp": {}
}
"#
        .to_string()
    }

    /// Run the binary with OPENCODE_CONFIG_DIR pointed at this fixture.
    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("occonfig").expect("binary should build");
        cmd.env("OPENCODE_CONFIG_DIR", self.path());
        // Ensure the file-path override does not leak in from the environment.
        cmd.env_remove("OPENCODE_CONFIG");
        cmd
    }

    fn config(&self) -> serde_json::Value {
        let raw = fs::read_to_string(self.config_path()).unwrap();
        serde_json::from_str(&raw).unwrap()
    }

    fn backups(&self) -> Vec<PathBuf> {
        fs::read_dir(self.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with(".pre-occonfig-") && n.ends_with(".bak"))
                    .unwrap_or(false)
            })
            .collect()
    }
}

fn write_profile(fx: &Fixture, name: &str, model: &str, agents: &[(&str, &str)]) {
    fs::create_dir_all(fx.profiles_dir()).unwrap();
    let mut agent_map = BTreeMap::new();
    for (k, v) in agents {
        agent_map.insert((*k).to_string(), (*v).to_string());
    }
    let profile = serde_json::json!({
        "name": name,
        "model": model,
        "agents": agent_map,
        "saved_at": "2026-09-13T00:00:00Z",
    });
    fs::write(
        fx.profiles_dir().join(format!("{name}.json")),
        serde_json::to_string_pretty(&profile).unwrap(),
    )
    .unwrap();
}

// ---------------------------------------------------------------------------
// current
// ---------------------------------------------------------------------------

#[test]
fn current_reports_the_top_level_model_and_drift() {
    let fx = Fixture::new();
    fx.cmd()
        .arg("current")
        .assert()
        .success()
        .stdout(predicate::str::contains("top level: local/one"))
        .stdout(predicate::str::contains("1 of 2 on the top-level model"))
        .stdout(predicate::str::contains("writer: remote/writer"));
}

// ---------------------------------------------------------------------------
// save
// ---------------------------------------------------------------------------

#[test]
fn save_writes_a_profile_with_the_current_assignments() {
    let fx = Fixture::new();
    fx.cmd()
        .args(["save", "lab"])
        .assert()
        .success()
        .stdout(predicate::str::contains("saved profile 'lab'"));

    let written = fs::read_to_string(fx.profiles_dir().join("lab.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&written).unwrap();
    assert_eq!(parsed["model"], serde_json::json!("local/one"));
    assert_eq!(parsed["agents"]["build"], serde_json::json!("local/one"));
    assert_eq!(
        parsed["agents"]["writer"],
        serde_json::json!("remote/writer")
    );
}

#[test]
fn save_refuses_overwrite_without_force() {
    let fx = Fixture::new();
    fx.cmd().args(["save", "lab"]).assert().success();

    // Second save without --force must fail non-zero.
    fx.cmd()
        .args(["save", "lab"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("--force"));

    // With --force it succeeds.
    fx.cmd().args(["save", "lab", "--force"]).assert().success();
}

// ---------------------------------------------------------------------------
// use
// ---------------------------------------------------------------------------

#[test]
fn use_applies_the_profile_to_every_agent_it_names() {
    let fx = Fixture::new();
    write_profile(
        &fx,
        "switched",
        "remote/big",
        &[("build", "remote/big"), ("writer", "remote/writer")],
    );

    fx.cmd()
        .args(["use", "switched"])
        .assert()
        .success()
        .stdout(predicate::str::contains("applied profile 'switched'"));

    let cfg = fx.config();
    assert_eq!(cfg["model"], serde_json::json!("remote/big"));
    assert_eq!(
        cfg["agent"]["build"]["model"],
        serde_json::json!("remote/big")
    );
}

#[test]
fn use_rejects_profile_with_unknown_provider() {
    let fx = Fixture::new();
    // `ghost/nothing` is not declared under `provider` in the fixture config.
    write_profile(&fx, "stale", "ghost/nothing", &[("build", "ghost/nothing")]);

    let before = fs::read_to_string(fx.config_path()).unwrap();

    fx.cmd()
        .args(["use", "stale"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("provider 'ghost' is not present"))
        .stderr(predicate::str::contains("was not modified"));

    let after = fs::read_to_string(fx.config_path()).unwrap();
    assert_eq!(before, after, "a stale profile must not modify the config");
    assert!(
        fx.backups().is_empty(),
        "a stale profile must fail before the backup step"
    );
}

#[test]
fn use_leaves_non_model_blocks_untouched() {
    let fx = Fixture::new();
    write_profile(&fx, "switched", "remote/big", &[("build", "remote/big")]);

    fx.cmd().args(["use", "switched"]).assert().success();

    let cfg = fx.config();
    assert_eq!(cfg["mcp"], serde_json::json!({}));
    assert_eq!(
        cfg["agent"]["writer"]["model"],
        serde_json::json!("remote/writer")
    );
}

#[test]
fn use_dry_run_writes_nothing() {
    let fx = Fixture::new();
    write_profile(&fx, "switched", "remote/big", &[("build", "remote/big")]);
    let before = fs::read_to_string(fx.config_path()).unwrap();

    fx.cmd()
        .args(["use", "switched", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would change"));

    let after = fs::read_to_string(fx.config_path()).unwrap();
    assert_eq!(before, after, "--dry-run must not write");
    assert!(fx.backups().is_empty(), "--dry-run must not back up");
}

#[test]
fn use_reports_an_unknown_profile_name_clearly() {
    let fx = Fixture::new();
    fx.cmd()
        .args(["use", "nope"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("no profile named 'nope'"));
}

// ---------------------------------------------------------------------------
// backups
// ---------------------------------------------------------------------------

#[test]
fn mutating_command_creates_backup() {
    let fx = Fixture::new();
    write_profile(&fx, "switched", "remote/big", &[("build", "remote/big")]);

    assert!(fx.backups().is_empty(), "no backup should exist yet");

    fx.cmd().args(["use", "switched"]).assert().success();

    let backups = fx.backups();
    assert_eq!(
        backups.len(),
        1,
        "expected exactly one backup, got {backups:?}"
    );

    let name = backups[0].file_name().unwrap().to_str().unwrap();
    assert!(
        name.starts_with(".pre-occonfig-opencode-"),
        "backup name {name} does not follow the convention"
    );

    // The backup holds the pre-change contents.
    let backup = fs::read_to_string(&backups[0]).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&backup).unwrap();
    assert_eq!(
        parsed["model"],
        serde_json::json!("local/one"),
        "the backup should hold the model from before the change"
    );
}

// ---------------------------------------------------------------------------
// set-model
// ---------------------------------------------------------------------------

#[test]
fn set_model_updates_top_level_and_all_agents() {
    let fx = Fixture::new();
    fx.cmd().args(["set-model", "local/two"]).assert().success();

    let cfg = fx.config();
    assert_eq!(cfg["model"], serde_json::json!("local/two"));
    assert_eq!(
        cfg["agent"]["build"]["model"],
        serde_json::json!("local/two")
    );
    assert_eq!(
        cfg["agent"]["writer"]["model"],
        serde_json::json!("local/two")
    );
    assert_eq!(fx.backups().len(), 1);
}

#[test]
fn set_model_rejects_an_unknown_reference() {
    let fx = Fixture::new();
    let before = fs::read_to_string(fx.config_path()).unwrap();

    fx.cmd()
        .args(["set-model", "nope/nothing"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("provider 'nope' is not present"));

    assert_eq!(fs::read_to_string(fx.config_path()).unwrap(), before);
    assert!(fx.backups().is_empty());
}

#[test]
fn set_model_agents_only_leaves_the_top_level_alone() {
    let fx = Fixture::new();
    fx.cmd()
        .args(["set-model", "remote/big", "--agents-only"])
        .assert()
        .success();

    let cfg = fx.config();
    assert_eq!(cfg["model"], serde_json::json!("local/one"));
    assert_eq!(
        cfg["agent"]["build"]["model"],
        serde_json::json!("remote/big")
    );
}

#[test]
fn set_model_does_not_pin_agents_that_inherit_the_top_level_model() {
    // Regression for a real config shape: some agents carry only `options`
    // (tools, maxOutputTokens) and no `model` key, so they inherit the
    // top-level model. `set-model` must not write a `model` into them, or a
    // later `use` of a profile saved before the change cannot restore them.
    let fx = Fixture::new();
    fs::write(
        fx.config_path(),
        r#"{
  "model": "local/one",
  "agent": {
    "build": {
      "model": "local/one"
    },
    "plan": {
      "options": {
        "maxOutputTokens": 32768
      }
    }
  },
  "provider": {
    "local": {
      "models": {
        "one": {},
        "two": {}
      }
    }
  },
  "mcp": {}
}
"#,
    )
    .unwrap();

    fx.cmd()
        .args(["set-model", "local/two"])
        .assert()
        .success()
        .stderr(predicate::str::contains("inherit the top-level one"));

    let cfg = fx.config();
    assert_eq!(cfg["model"], serde_json::json!("local/two"));
    assert_eq!(
        cfg["agent"]["build"]["model"],
        serde_json::json!("local/two")
    );
    assert!(
        cfg["agent"]["plan"].get("model").is_none(),
        "an agent with no model key must not be pinned; got {:?}",
        cfg["agent"]["plan"]
    );
    assert_eq!(
        cfg["agent"]["plan"]["options"]["maxOutputTokens"],
        serde_json::json!(32768),
        "the agent's existing keys must survive"
    );
}

// ---------------------------------------------------------------------------
// key order (the preserve_order guard)
// ---------------------------------------------------------------------------

#[test]
fn saving_preserves_agent_key_order() {
    let fx = Fixture::new();

    // Deliberately non-alphabetical agent order: `build` before `writer`
    // happens to sort that way, so use names where alphabetical differs.
    let original = r#"{
  "model": "local/one",
  "agent": {
    "zebra": {
      "model": "local/one"
    },
    "alpha": {
      "model": "local/one"
    }
  },
  "provider": {
    "local": {
      "models": {
        "one": {},
        "two": {}
      }
    }
  },
  "mcp": {}
}
"#;
    fs::write(fx.config_path(), original).unwrap();

    let cfg = fx.config();
    let keys_before: Vec<&String> = cfg["agent"].as_object().unwrap().keys().collect();
    assert_eq!(
        keys_before,
        vec!["zebra", "alpha"],
        "fixture order is wrong"
    );

    write_profile(&fx, "switched", "local/two", &[("zebra", "local/two")]);
    fx.cmd().args(["use", "switched"]).assert().success();

    let after = fx.config();
    let keys_after: Vec<&String> = after["agent"].as_object().unwrap().keys().collect();
    assert_eq!(
        keys_after,
        vec!["zebra", "alpha"],
        "agent key order changed on save; the preserve_order feature is missing \
         from serde_json in Cargo.toml"
    );
}

// ---------------------------------------------------------------------------
// doctor
// ---------------------------------------------------------------------------

#[test]
fn doctor_reports_stale_profile_reference() {
    let fx = Fixture::new();
    // Add a provider so the config is not empty, then a profile that points
    // somewhere absent.
    fs::write(
        fx.config_path(),
        r#"{
  "model": "local/one",
  "provider": {
    "local": {
      "models": {
        "one": {}
      }
    }
  },
  "agent": {}
}
"#,
    )
    .unwrap();
    write_profile(&fx, "stale", "removed/gone", &[]);

    fx.cmd()
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("problem(s) found"))
        .stdout(predicate::str::contains("references 'removed/gone'"))
        .stdout(predicate::str::contains(
            "provider 'removed' is not present",
        ));
}

#[test]
fn doctor_reports_ok_when_everything_resolves() {
    let fx = Fixture::new();
    fs::write(
        fx.config_path(),
        r#"{
  "model": "local/one",
  "provider": {
    "local": {
      "models": {
        "one": {}
      }
    }
  },
  "agent": {}
}
"#,
    )
    .unwrap();
    write_profile(&fx, "good", "local/one", &[]);

    fx.cmd()
        .args(["doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ok: every model reference resolves",
        ));
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

#[test]
fn list_reports_saved_profiles() {
    let fx = Fixture::new();
    write_profile(&fx, "lab", "local/one", &[]);
    write_profile(&fx, "other", "local/two", &[]);

    fx.cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("lab"))
        .stdout(predicate::str::contains("other"));
}

#[test]
fn list_says_so_when_there_are_no_profiles() {
    let fx = Fixture::new();
    fx.cmd()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("no profiles saved yet"));
}

// ---------------------------------------------------------------------------
// version
// ---------------------------------------------------------------------------

#[test]
fn version_prints_something_non_empty() {
    let fx = Fixture::new();
    let out = fx.cmd().arg("--version").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("occonfig"),
        "version output should name the binary, got: {stdout:?}"
    );
}
