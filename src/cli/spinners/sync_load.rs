use crate::utils::env::is_ci_environment;
use anyhow::Result;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle};
use std::{
    io::{self, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::time::sleep;

/// For visualization of loadings, infos,...
/// during the synchronization process
pub struct SyncLoading {
    /// In case the synchronization should be completely silent
    should_message: bool,
    cli: Option<Cli>,
}
pub struct Cli {
    /// - All progress bars during the synchronization
    /// - should be under this
    main_progress: MultiProgress,
    /// - For symbolizing statuses of each Knots folder.
    /// - If they are synced it's   else 
    /// - Graph will look like:  ——— ———
    node_graph: Option<ProgressBar>,
}

impl SyncLoading {
    /// Synchronization with no info
    /// NOT even messages
    pub fn empty() -> Self {
        Self {
            should_message: false,
            cli: None,
        }
    }

    /// This will just stderr all info messages
    /// No CLI spinners or TUI things
    pub fn basic() -> Self {
        Self {
            should_message: true,
            cli: None,
        }
    }

    /// Only spinners and loading bars will occur
    /// Node graph won't be created
    /// If it's CI environment, stderr will
    /// produce info messages instead of spinners
    pub fn simple_cli() -> Self {
        if is_ci_environment() {
            Self {
                cli: None,
                should_message: true,
            }
        } else {
            Self {
                cli: Some(Cli {
                    node_graph: None,
                    main_progress: MultiProgress::new(),
                }),
                should_message: true,
            }
        }
    }

    /// This will create full CLI with
    /// Loading bars, spinners and Node graph
    /// If it's CI environment, stderr will
    /// produce info messages instead of spinners
    pub fn full_cli() -> Self {
        if is_ci_environment() {
            Self {
                cli: None,
                should_message: true,
            }
        } else {
            let m = MultiProgress::new();
            let node_graph = m.add(ProgressBar::new_spinner());
            node_graph.set_style(
                ProgressStyle::with_template(" {prefix:.cyan}{spinner:.blue}{msg}")
                    .unwrap()
                    // .tick_chars("󰪞󰪟󰪠󰪡󰪢󰪣󰪤󰪥")
                    // .tick_chars("○◎●◎◌"),
                    .tick_chars(""),
            );
            node_graph.enable_steady_tick(Duration::from_millis(500));
            Self {
                should_message: true,
                cli: Some(Cli {
                    main_progress: m,
                    node_graph: Some(node_graph),
                }),
            }
        }
    }

    pub fn update_node(&self, index: usize, total_remotes: usize) {
        self.set_done_nodes(index);
        self.set_remaining_node(index, total_remotes);
    }

    pub fn node_finish(&self, total_remotes: usize) {
        if let Some(ref cli) = self.cli
            && let Some(ref node_graph) = cli.node_graph
        {
            let mut final_graph = String::new();
            for _ in 0..(total_remotes.saturating_sub(1)) {
                final_graph.push_str(&format!("{} {}", "".green(), "———".green()));
            }
            if total_remotes > 0 {
                final_graph.push_str(&format!("{}", "".green()));
            }

            node_graph.set_style(ProgressStyle::with_template(" {msg:.green} ").unwrap());
            node_graph.finish_with_message(format!(
                "{}    {}",
                final_graph.green(),
                "[All Knots Synced]".green()
            ));
        }
    }

    /// This will set all done nodes in Node graph
    pub fn set_done_nodes(&self, index: usize) {
        if let Some(ref cli) = self.cli
            && let Some(ref node_graph) = cli.node_graph
        {
            let mut prefix = String::new();
            for i in 0..index {
                let node_str = "".cyan().to_string();
                let pipe_str = if i == index - 1 {
                    format!(
                        "{}{}{}",
                        "—".cyan(),
                        "—".truecolor(137, 179, 188),
                        "—".blue()
                    )
                } else {
                    "———".cyan().to_string()
                };

                prefix.push_str(&format!("{} {}", node_str, pipe_str));
            }
            node_graph.set_prefix(prefix);
        }
    }

    /// Will create all remaining nodes with `[Syncing Knot {current}/{total}]`
    pub fn set_remaining_node(&self, index: usize, total_remotes: usize) {
        if let Some(ref cli) = self.cli
            && let Some(ref node_graph) = cli.node_graph
        {
            let mut msg = String::new();
            for _ in (index + 1)..total_remotes {
                msg.push_str(" ———");
            }

            msg.push_str(&format!(
                "   [Syncing Knot {:02}/{:02}]",
                index + 1,
                total_remotes
            ));
            node_graph.set_message(msg);
        }
    }

    /// Standard print
    pub fn print<S: AsRef<str>>(&self, message: S) -> Result<()> {
        if let Some(ref cli) = self.cli {
            cli.main_progress.println(message)?;
        } else if self.should_message {
            let msg = message.as_ref();
            eprintln!("{msg}")
        }
        Ok(())
    }

    /// This will print out only if it's CI environment
    pub fn ci_print<S: AsRef<str>>(&self, message: S) {
        if is_ci_environment() {
            let msg = message.as_ref();
            eprintln!("{msg}");
        }
    }

    /// Instead of printing on the top of node_graph,
    /// you can print under it instead
    /// Example:
    /// ```
    /// $  ——— ———  [Syncing Knot {current}/{total}]
    /// $ Here will be the message
    /// ```
    /// On no CLI this will just print it out into stderr
    pub fn print_under<S: AsRef<str>>(&self, message: S) -> Result<()> {
        let msg = message.as_ref();
        if let Some(ref cli) = self.cli {
            let log_pb = cli.main_progress.add(ProgressBar::new(0));
            log_pb.set_style(ProgressStyle::with_template("{msg}")?);
            log_pb.finish_with_message(msg.to_string());
        } else if self.should_message {
            eprintln!("{msg}");
        }
        Ok(())
    }

    /// In case you need to hide the spinners, node graph,...
    /// Nothing will happen when it's non-CLI
    pub fn cli_clear_and_hide(&self) -> Result<()> {
        if let Some(ref cli) = self.cli {
            cli.main_progress.clear()?;
            cli.main_progress
                .set_draw_target(ProgressDrawTarget::hidden());
        }
        Ok(())
    }

    pub fn cli_clear(&self) -> Result<()> {
        if let Some(ref cli) = self.cli {
            cli.main_progress.clear()?;
        }
        Ok(())
    }

    /// Will restore all graphs and spinners
    pub fn cli_restore(&self) {
        if let Some(ref cli) = self.cli
            // Doesn't make sense to unhide
            // Something that is not hidden
            && cli.main_progress.is_hidden()
        {
            cli.main_progress
                .set_draw_target(ProgressDrawTarget::stderr());
        }
    }

    /// Won't return the progress bar if this is CI/CD environment
    pub fn create_sync_progress_bar(&self, total_tasks: usize) -> Option<ProgressBar> {
        use std::fmt::Write;
        if let Some(ref cli) = self.cli {
            let pb = cli.main_progress.add(ProgressBar::new(total_tasks as u64));
            pb.set_style(
                    ProgressStyle::with_template(
                        " {spinner:.green} [{elapsed_precise}] [{bar:25.cyan/blue}] {pos}/{len} ({rate}, ETA {eta}) {wide_msg}",
                    )
                    .unwrap()
                    .with_key("rate", |state: &ProgressState, w: &mut dyn Write| {
                        write!(w, "{:.0} Files/s", state.per_sec()).unwrap();
                    })
                    .progress_chars("█▉▊▋▌▍▎▏ ")
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
                );
            pb.enable_steady_tick(Duration::from_millis(80));
            Some(pb)
        } else {
            None
        }
    }

    /// In CI environment this will print out message into stderr
    pub fn create_spinner<S: AsRef<str>>(&self, message: S) -> Option<ProgressBar> {
        let msg = message.as_ref();
        if let Some(ref cli) = self.cli {
            let pb = cli.main_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::with_template(" {spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
                    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
            );
            pb.set_message(msg.to_string());
            pb.enable_steady_tick(Duration::from_millis(80));
            Some(pb)
        } else {
            if self.should_message {
                eprintln!("{msg}");
            }
            None
        }
    }

    pub fn inc(
        progress_bar: Option<&ProgressBar>,
        delta: u64,
        done_tasks: usize,
        total_tasks: usize,
    ) {
        if let Some(bar) = progress_bar {
            bar.inc(delta);
        }

        if total_tasks > 0 && !is_ci_environment() {
            update_progress(done_tasks as u8);
        }
    }
}
/// Uses Terminal's loading bar escape sequence
/// On CI version, this will do nothing
pub fn update_progress(percent: u8) {
    if !is_ci_environment() {
        print!("\x1b]9;4;1;{percent}\x07");
        // it's just visual info
        // no reason to crash whole app because of this
        // I prefer successful synchronization and possible broken
        // terminal, than broken synchronization with broken terminal
        let _ = io::stdout().flush();
    }
}

/// Clear Terminal's loading bar
/// On CI version, this will do nothing
pub fn clear_progress() {
    if !is_ci_environment() {
        print!("\x1b]9;4;0;0\x07");
        // it's just visual info
        // no reason to crash whole app because of this
        // I prefer successful synchronization and possible broken
        // terminal, than broken synchronization with broken terminal
        let _ = io::stdout().flush();
    }
}

/// Will start spinner process on separate task
/// This returns `stop_spinner: AtomicBool` on not CI environment
pub fn use_terminal_spinner() -> Option<Arc<AtomicBool>> {
    if !is_ci_environment() {
        let term_spinner_stop = Arc::new(AtomicBool::new(false));
        let term_spinner_stop_cl = Arc::clone(&term_spinner_stop);
        tokio::spawn(async move {
            while !term_spinner_stop_cl.load(Ordering::Relaxed) {
                print!("\x1b]9;4;3;0\x07");
                // it's just visual info
                // no reason to crash whole app because of this
                // I prefer successful synchronization and possible broken
                // terminal, than broken synchronization with broken terminal
                let _ = io::stdout().flush();
                sleep(Duration::from_millis(80)).await;
            }
            clear_terminal_spinner();
        });
        Some(term_spinner_stop)
    } else {
        None
    }
}

/// On CI version, this will do nothing
pub fn clear_terminal_spinner() {
    if !is_ci_environment() {
        print!("\x1b]9;4;0;0\x07");
        // it's just visual info
        // no reason to crash whole app because of this
        // I prefer successful synchronization and possible broken
        // terminal, than broken synchronization with broken terminal
        let _ = io::stdout().flush();
    }
}
