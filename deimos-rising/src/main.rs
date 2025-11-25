mod fs;
mod game;
mod hooks;
mod host;
mod logging;
mod sdl;
mod time;

use anyhow::anyhow;
use std::process::ExitCode;
use win32::Machine;

#[derive(argh::FromArgs, Debug)]
/// Deimos Rising port runner
struct Args {
    /// winapi systems to trace; see HACKING.md for docs
    #[argh(option)]
    win32_trace: Option<String>,

    /// enable debug logging
    #[argh(switch)]
    debug: bool,

    /// path to Deimos Rising folder
    #[argh(positional)]
    game_path: String,
}

/// Convert a unix command line to a Windows form.
fn command_line_to_windows(
    host: &dyn win32::host::Host,
    game_path: String,
) -> anyhow::Result<String> {
    // Convert argument to full path to exe.
    let cwd = host
        .current_dir()
        .map_err(|e| anyhow!("failed to get current dir: {e:?}"))?;
    let mut full_path = cwd
        .join(&game_path)
        .join("DeimosRising.exe")
        .normalize()
        .to_str()
        .ok_or_else(|| anyhow!("invalid path"))?
        .to_owned();

    escape_arg(&mut full_path);

    Ok(full_path)
}

fn escape_arg(arg: &mut String) {
    // TODO: correct escaping here:
    // https://learn.microsoft.com/en-us/archive/blogs/twistylittlepassagesallalike/everyone-quotes-command-line-arguments-the-wrong-way

    if !arg.contains(&['"', ' ', '\t', '\n']) {
        return;
    }

    let mut escaped = String::with_capacity(arg.len() + 2);
    escaped.push('"');
    for c in arg.chars() {
        match c {
            '"' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped.push('"');
    *arg = escaped;
}

fn main() -> anyhow::Result<ExitCode> {
    let mut args: Args = argh::from_env();

    std::env::set_current_dir(&args.game_path).unwrap();

    // Initialize logging
    logging::init(if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    });

    win32::trace::set_scheme(args.win32_trace.as_deref().unwrap_or("-"));

    let host = host::new_host();
    let cmdline = command_line_to_windows(&host, std::mem::take(&mut args.game_path))?;
    let mut machine = win32::Machine::new(Box::new(host));

    machine.set_audio(true);

    // Start the EXE (this will break before entry point)
    machine.break_on_startup();
    machine.start_exe(cmdline, None);

    // Run until we hit the startup breakpoint
    log::info!("Running until startup breakpoint...");
    while machine.run() {
        // Keep running until we hit the break
    }

    // Install game-specific hooks
    log::info!("Installing Deimos Rising hooks...");
    hooks::install_hooks(&mut machine);

    // Unblock and continue execution
    log::info!("Continuing execution...");
    machine.unblock();

    let status = run_emu(&mut machine)?;
    let exit_code = match status {
        win32::Status::Exit(code) => code,
        win32::Status::Error { message } => {
            log::error!("{}", message);
            machine.dump_state(0);
            1
        }
        _ => unreachable!("{status:?}"),
    };
    Ok(ExitCode::from(exit_code as u8))
}

fn run_emu(machine: &mut Machine) -> anyhow::Result<win32::Status> {
    let start = std::time::Instant::now();

    while machine.run() {}

    let millis = start.elapsed().as_millis() as usize;
    if millis > 0 {
        eprintln!(
            "{} instrs in {} ms: {}m/s",
            machine.emu.x86.instr_count,
            millis,
            (machine.emu.x86.instr_count / millis) / 1000
        );
        eprintln!("icache: {}", machine.emu.x86.icache.stats());
    }

    Ok(std::mem::take(&mut machine.emu.status))
}
