// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.

use std::process::Command;
use test_util as util;
use util::env_vars_for_npm_tests;
use util::env_vars_for_npm_tests_no_sync_download;
use util::TestContext;

itest!(_095_check_with_bare_import {
  args: "check cache/095_cache_with_bare_import.ts",
  output: "cache/095_cache_with_bare_import.ts.out",
  exit_code: 1,
});

itest!(check_extensionless {
  args: "check --reload http://localhost:4545/subdir/no_js_ext",
  output: "cache/cache_extensionless.out",
  http_server: true,
});

itest!(check_random_extension {
  args: "check --reload http://localhost:4545/subdir/no_js_ext@1.0.0",
  output: "cache/cache_random_extension.out",
  http_server: true,
});

itest!(check_all {
  args: "check --quiet --all check/all/check_all.ts",
  output: "check/all/check_all.out",
  http_server: true,
  exit_code: 1,
});

itest!(check_all_local {
  args: "check --quiet check/all/check_all.ts",
  output_str: Some(""),
  http_server: true,
});

itest!(module_detection_force {
  args: "check --quiet check/module_detection_force/main.ts",
  output_str: Some(""),
});

// Regression test for https://github.com/denoland/deno/issues/14937.
itest!(declaration_header_file_with_no_exports {
  args: "check --quiet check/declaration_header_file_with_no_exports.ts",
  output_str: Some(""),
});

itest!(check_npm_install_diagnostics {
  args: "check --quiet check/npm_install_diagnostics/main.ts",
  output: "check/npm_install_diagnostics/main.out",
  envs: vec![("NO_COLOR".to_string(), "1".to_string())],
  exit_code: 1,
});

itest!(check_export_equals_declaration_file {
  args: "check --quiet check/export_equals_declaration_file/main.ts",
  exit_code: 0,
});

itest!(check_static_response_json {
  args: "check --quiet check/response_json.ts",
  exit_code: 0,
});

itest!(check_node_builtin_modules_ts {
  args: "check --quiet check/node_builtin_modules/mod.ts",
  output: "check/node_builtin_modules/mod.ts.out",
  exit_code: 1,
});

itest!(check_node_builtin_modules_js {
  args: "check --quiet check/node_builtin_modules/mod.js",
  output: "check/node_builtin_modules/mod.js.out",
  exit_code: 1,
});

itest!(check_no_error_truncation {
    args: "check --quiet check/no_error_truncation/main.ts --config check/no_error_truncation/deno.json",
    output: "check/no_error_truncation/main.out",
    envs: vec![("NO_COLOR".to_string(), "1".to_string())],
    exit_code: 1,
  });

#[test]
fn cache_switching_config_then_no_config() {
  let context = TestContext::default();

  assert!(does_type_checking(&context, true));
  assert!(does_type_checking(&context, false));

  // should now not do type checking even when it changes
  // configs because it previously did
  assert!(!does_type_checking(&context, true));
  assert!(!does_type_checking(&context, false));

  fn does_type_checking(context: &TestContext, with_config: bool) -> bool {
    let mut args = vec![
      "check".to_string(),
      "check/cache_config_on_off/main.ts".to_string(),
    ];
    if with_config {
      let mut slice = vec![
        "--config".to_string(),
        "check/cache_config_on_off/deno.json".to_string(),
      ];
      args.append(&mut slice);
    }

    let output = context.new_command().args_vec(args).split_output().run();

    output.assert_exit_code(0);

    let stderr = output.stderr();
    stderr.contains("Check")
  }
}

#[test]
fn reload_flag() {
  // should do type checking whenever someone specifies --reload
  let context = TestContext::default();

  assert!(does_type_checking(&context, false));
  assert!(!does_type_checking(&context, false));
  assert!(does_type_checking(&context, true));
  assert!(does_type_checking(&context, true));
  assert!(!does_type_checking(&context, false));

  fn does_type_checking(context: &TestContext, reload: bool) -> bool {
    let mut args = vec![
      "check".to_string(),
      "check/cache_config_on_off/main.ts".to_string(),
    ];
    if reload {
      let mut slice = vec!["--reload".to_string()];
      args.append(&mut slice);
    }
    let output = context.new_command().args_vec(args).split_output().run();
    output.assert_exit_code(0);

    let stderr = output.stderr();
    stderr.contains("Check")
  }
}

#[test]
fn typecheck_declarations_ns() {
  let context = TestContext::default();
  let args = vec![
    "test".to_string(),
    "--doc".to_string(),
    util::root_path()
      .join("cli/tsc/dts/lib.deno.ns.d.ts")
      .to_string_lossy()
      .into_owned(),
  ];
  let output = context.new_command().args_vec(args).split_output().run();

  println!("stdout: {}", output.stdout());
  println!("stderr: {}", output.stderr());
  output.assert_exit_code(0);
}

#[test]
fn typecheck_declarations_unstable() {
  let context = TestContext::default();
  let args = vec![
    "test".to_string(),
    "--doc".to_string(),
    "--unstable".to_string(),
    util::root_path()
      .join("cli/tsc/dts/lib.deno.unstable.d.ts")
      .to_string_lossy()
      .into_owned(),
  ];
  let output = context.new_command().args_vec(args).split_output().run();

  println!("stdout: {}", output.stdout());
  println!("stderr: {}", output.stderr());
  output.assert_exit_code(0);
}

#[test]
fn typecheck_core() {
  let context = TestContext::default();
  let deno_dir = context.deno_dir();
  let test_file = deno_dir.path().join("test_deno_core_types.ts");
  std::fs::write(
    &test_file,
    format!(
      "import \"{}\";",
      deno_core::resolve_path(
        util::root_path()
          .join("core/lib.deno_core.d.ts")
          .to_str()
          .unwrap(),
        &std::env::current_dir().unwrap()
      )
      .unwrap()
    ),
  )
  .unwrap();

  let args = vec!["run".to_string(), test_file.to_string_lossy().into_owned()];
  let output = context.new_command().args_vec(args).split_output().run();

  println!("stdout: {}", output.stdout());
  println!("stderr: {}", output.stderr());
  output.assert_exit_code(0);
}

#[test]
fn ts_no_recheck_on_redirect() {
  // TODO: port to test builder
  let deno_dir = util::new_deno_dir();
  let e = util::deno_exe_path();

  let redirect_ts = util::testdata_path().join("run/017_import_redirect.ts");
  assert!(redirect_ts.is_file());
  let mut cmd = Command::new(e.clone());
  cmd.env("DENO_DIR", deno_dir.path());
  let mut initial = cmd
    .current_dir(util::testdata_path())
    .arg("run")
    .arg("--check")
    .arg(redirect_ts.clone())
    .spawn()
    .expect("failed to span script");
  let status_initial =
    initial.wait().expect("failed to wait for child process");
  assert!(status_initial.success());

  let mut cmd = Command::new(e);
  cmd.env("DENO_DIR", deno_dir.path());
  let output = cmd
    .current_dir(util::testdata_path())
    .arg("run")
    .arg("--check")
    .arg(redirect_ts)
    .output()
    .expect("failed to spawn script");

  assert!(std::str::from_utf8(&output.stderr).unwrap().is_empty());
}

itest!(check_dts {
  args: "check --quiet check/dts/check_dts.d.ts",
  output: "check/dts/check_dts.out",
  exit_code: 1,
});

itest!(check_types_dts {
  args: "check main.ts",
  cwd: Some("check/types_dts/"),
  output: "check/types_dts/main.out",
  exit_code: 0,
});

itest!(package_json_basic {
  args: "check main.ts",
  output: "package_json/basic/main.check.out",
  envs: env_vars_for_npm_tests(),
  http_server: true,
  cwd: Some("package_json/basic"),
  copy_temp_dir: Some("package_json/basic"),
  exit_code: 0,
});

itest!(package_json_fail_check {
  args: "check --quiet fail_check.ts",
  output: "package_json/basic/fail_check.check.out",
  envs: env_vars_for_npm_tests_no_sync_download(),
  http_server: true,
  cwd: Some("package_json/basic"),
  copy_temp_dir: Some("package_json/basic"),
  exit_code: 1,
});

itest!(package_json_with_deno_json {
  args: "check --quiet main.ts",
  output: "package_json/deno_json/main.check.out",
  cwd: Some("package_json/deno_json/"),
  copy_temp_dir: Some("package_json/deno_json/"),
  envs: env_vars_for_npm_tests_no_sync_download(),
  http_server: true,
  exit_code: 1,
});
