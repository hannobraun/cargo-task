#![forbid(unsafe_code)]
#![forbid(missing_docs)]
#![forbid(warnings)]
//! Ultra-Lightweight Zero-Dependency Rust Cargo Task Runner.
//!
//! - Platform Agnostic - runs on any platform that cargo runs on.
//! - Zero-Dependency - the task manager itself installs almost instantly.
//! - Rust Task Logic - stop writing separate bash and powershell scripts.
//! - Take a look at the ['.cargo-task' directory](./.cargo-task) in this repo for examples.
//!
//! ```shell
//! cargo install -f cargo-task
//! cargo help task
//! ```
//!
//! ## Creating `cargo-task` automation tasks.
//!
//! ```shell
//! cargo task ct-init
//! cd .cargo-task
//! cargo new --bin my-task
//! cd ..
//! cargo task my-task
//! ```
//!
//! - `cargo task ct-init` - creates the `.cargo-task` directory and `.gitignore`.
//! - `cargo task my-task` - runs the crate named `my-task` defined in the `.cargo-task` directory.
//!
//! It's that simple!
//!
//! ## Script-like single file tasks.
//!
//! If you don't want to commit a whole directory / Cargo.toml etc... you can
//! specify cargo-task tasks as single files.
//!
//! Just create a file in your `.cargo-task` directory named something like
//! `my-task.ct.rs` and write it as you would a `main.rs`.
//!
//! This will also create a `my-task` cargo task. You can even specify cargo
//! crate dependencies via AtAt directive `@ct-cargo-deps@` (see below).
//!
//! ## Customizing how tasks are executed.
//!
//! `cargo-task` uses a metadata format called AtAt - because it uses `@` signs:
//!
//! ```ignore
//! /*
//! @ct-default@ true @@
//! @ct-task-deps@
//! one
//! two
//! @@
//! */
//! ```
//!
//! Some things to know about AtAt:
//! - protocol: `@key@ value @@`.
//! - the first `@` for the key must be the first character on a line.
//! - the value is terminated by a two ats, "`@@`".
//! - the value can contain newlines or be on a single line.
//! - you probably want it in a rust comment block : )
//!
//! These directives will be read from your `main.rs` file when parsing the
//! `.cargo-task` crates.
//!
//! ### Default tasks.
//!
//! ```ignore
//! /*
//! @ct-default@ true @@
//! */
//! ```
//!
//! Default tasks will be executed if the task list is empty on `cargo task`
//! invocations.
//!
//! ### Bootstrap tasks.
//!
//! ```ignore
//! /*
//! @ct-bootstrap@ true @@
//! */
//! ```
//!
//! Bootstrap tasks will *always* be executed before any task-list tasks.
//! Also, the cargo-task metadata will be reloaded after bootstrap tasks
//! are executed. You can use this to download / install / configure
//! additional tasks.
//!
//! ### Cargo dependencies.
//!
//! ```ignore
//! /*
//! @ct-cargo-deps@
//! num_cpus = "1"
//! serde = { version = "1", features = [ "derive" ] }
//! @@
//! */
//! ```
//!
//! Write them just as you would in your Cargo.toml.
//!
//! ### Task dependencies.
//!
//! ```ignore
//! /*
//! @ct-task-deps@
//! my-first-dependency
//! my-second-dependency
//! @@
//! */
//! ```
//!
//! ### Minimum cargo-task version.
//!
//! ```ignore
//! /*
//! @ct-min-version@ 0.0.7 @@
//! */
//! ```
//!
//! Require at least a minimum version of cargo-task to prompt users
//! to upgrade if you are depending on features.
//! Note, this directive works well when combined with `@ct-bootstrap@`
//!
//! A whitespace delimited list of tasks that must be run prior to the current
//! task. Can be on a single line or multiple lines.
//!
//! ## The magic `cargo_task_util` module.
//!
//! - [cargo_task_util on docs.rs](https://docs.rs/cargo-task/latest/cargo_task/cargo_task_util/index.html)
//!
//! This module will be available at the root of your crate during build time.
//! To include it, simply add a `mod` directive in your `main.rs` file.
//!
//! ```ignore
//! /*
//! @ct-default@ true @@
//! */
//!
//! mod cargo_task_util;
//! use cargo_task_util::*;
//!
//! fn main() {
//!     // cargo task metadata env helper
//!     let env = ct_env();
//!
//!     // print out some cool cargo-task metadata
//!     // (this does the same thing as `cargo task ct-meta`)
//!     println!("{:#?}", env);
//!
//!     // also includes cargo-task special log helpers
//!     ct_warn!("ahh, a warning! {:?}", std::time::SystemTime::now());
//! }
//! ```
//!
//! ## Exporting environment variables to configure other tasks.
//!
//! `cargo_task_util::CTEnv` also includes a utility for exporting environment
//! variables.
//!
//! If you just use the rust `std::env::set_var` function, the variable will
//! be set for the current task execution, but no other tasks will see it.
//!
//! Instead you can use `cargo_task_util::CTEnv::set_env` function.
//!
//! You probably want to do this in a "bootstrap" task so it is available
//! to other tasks that are run later.
//!
//! ```ignore
//! /*
//! @ct-bootstrap@ true @@
//! */
//!
//! mod cargo_task_util;
//! use cargo_task_util::*;
//!
//! fn main() {
//!     // cargo task metadata env helper
//!     let env = ct_env();
//!
//!     // set a variable that will be available in other tasks.
//!     env.set_env("MY_VARIABLE", "MY_VALUE");
//! }
//! ```

pub mod at_at;
pub mod cargo_task_util;
mod env_loader;
mod task;

#[cfg(windows)]
include!(concat!(env!("OUT_DIR"), "\\ver.rs"));
#[cfg(not(windows))]
include!(concat!(env!("OUT_DIR"), "/ver.rs"));

/// The .cargo-task directory name
const CARGO_TASK_DIR: &str = ".cargo-task";

/// The .cargo-task/.gitignore file
#[cfg(windows)]
const CT_DIR_GIT_IGNORE: &str = ".cargo-task\\.gitignore";
#[cfg(not(windows))]
const CT_DIR_GIT_IGNORE: &str = ".cargo-task/.gitignore";

/// Source content of .cargo-task/.gitignore for 'ct-init'
#[cfg(windows)]
const CT_DIR_GIT_IGNORE_SRC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "\\.cargo-task\\.gitignore"
));
#[cfg(not(windows))]
const CT_DIR_GIT_IGNORE_SRC: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.cargo-task/.gitignore"
));

/// Source-code content of cargo_task_util.rs
const CARGO_TASK_UTIL_SRC: &[u8] = include_bytes!("cargo_task_util.rs");

mod exec;
pub use exec::*;
