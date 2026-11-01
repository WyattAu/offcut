//! offcut-cut: render worker for offcut clip cutting.
//!
//! Pins `worker-kit` (WyattAu estate job supervisor): the render sweep is a
//! registered JobSpec on an interval trigger, supervised with failure
//! budgets and a circuit breaker — the same semantics the ledger and
//! outbox sweeps use estate-wide.
//!
//! Spool layout (default ./spool):
//!   pending/<job>.json   submitted, not started
//!   running/<job>.json   claimed
//!   done/<job>.json      ffmpeg exited 0 (result JSON alongside)
//!   failed/<job>.json    ffmpeg failed (error captured in the file)
//!
//! Job file shape:
//!   {"input": "in.mp4", "start": "00:00:01", "end": "00:00:04",
//!    "output": "clip.mp4", "reencode": false}
//!
//! Usage:
//!   offcut-cut worker  [--spool <dir>]   supervised sweep loop (worker-kit)
//!   offcut-cut exec    --job <file>      one-shot cut, bypasses the spool
//!   offcut-cut submit  --spool <dir> --input <in> --start S --end E \
//!                      --output <out> [--reencode]
//!
//! Exit codes: 0 ok, 1 job/ffmpeg failure (exec mode), 2 usage/IO.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use shutdown_kit::ShutdownGuard;
use worker_kit::{Job, JobContext, JobError, JobSpec, Trigger, WorkerSupervisor};

#[derive(Serialize, Deserialize, Clone)]
struct CutJob {
    input: String,
    start: String,
    end: String,
    output: String,
    /// false = stream copy (fast, keyframe-aligned); true = re-encode
    /// (frame-accurate, libx264 + aac). Default true.
    #[serde(default = "default_reencode")]
    reencode: bool,
}

fn default_reencode() -> bool {
    true
}

#[derive(Serialize)]
struct CutResult {
    ok: bool,
    output: Option<String>,
    error: Option<String>,
}

async fn run_ffmpeg(job: &CutJob) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.arg("-y").arg("-hide_banner").arg("-loglevel").arg("error");
    cmd.arg("-ss").arg(&job.start);
    cmd.arg("-to").arg(&job.end);
    cmd.arg("-i").arg(&job.input);
    if job.reencode {
        cmd.args(["-c:v", "libx264", "-preset", "veryfast", "-c:a", "aac"]);
    } else {
        cmd.arg("-c").arg("copy");
    }
    cmd.arg(&job.output);
    let out = cmd
        .output()
        .await
        .map_err(|e| format!("ffmpeg spawn failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).chars().take(400).collect::<String>()
        ));
    }
    Ok(())
}

// tokio-process inside an async closure; the async block borrows nothing.
async fn ffmpeg_cut(job: &CutJob) -> Result<(), String> {
    run_ffmpeg(job).await
}

fn spool_dirs(spool: &Path) -> [(PathBuf, &str); 4] {
    [
        (spool.join("pending"), "pending"),
        (spool.join("running"), "running"),
        (spool.join("done"), "done"),
        (spool.join("failed"), "failed"),
    ]
}

async fn sweep(spool: &Path) -> Result<usize, String> {
    let mut processed = 0;
    let pending = spool.join("pending");
    let running = spool.join("running");
    let done = spool.join("done");
    let failed = spool.join("failed");
    let jobs = match std::fs::read_dir(&pending) {
        Ok(d) => d,
        Err(_) => return Ok(0),
    };
    for entry in jobs.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let claimed = running.join(&name);
        std::fs::rename(&path, &claimed).map_err(|e| e.to_string())?;
        let raw = std::fs::read_to_string(&claimed).map_err(|e| e.to_string())?;
        let job: CutJob = serde_json::from_str(&raw).map_err(|e| format!("{name}: {e}"))?;
        let result = match ffmpeg_cut(&job).await {
            Ok(()) => CutResult { ok: true, output: Some(job.output.clone()), error: None },
            Err(e) => CutResult { ok: false, output: None, error: Some(e) },
        };
        let dest = if result.ok { &done } else { &failed };
        std::fs::write(
            dest.join(&name),
            serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&claimed);
        processed += 1;
    }
    Ok(processed)
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("");
    match mode {
        "worker" => worker_main(&args[1..]).await,
        "exec" => exec_main(&args[1..]).await,
        "submit" => submit_main(&args[1..]),
        _ => {
            eprintln!("usage: offcut-cut worker [--spool <dir>] | exec --job <file> | submit ...");
            std::process::exit(2);
        }
    }
}

fn spool_of(args: &[String]) -> PathBuf {
    args.iter()
        .position(|a| a == "--spool")
        .and_then(|p| args.get(p + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("spool"))
}

async fn worker_main(args: &[String]) {
    let spool = spool_of(args);
    for (dir, _) in spool_dirs(&spool) {
        std::fs::create_dir_all(&dir).expect("spool dirs");
    }
    let sweep_spool = spool.clone();
    let sweep: Job = Arc::new(move |_ctx: JobContext| {
        let spool = sweep_spool.clone();
        Box::pin(async move {
            let n = sweep(&spool).await.map_err(JobError::msg)?;
            if n > 0 {
                eprintln!("[offcut-cut] processed {n} job(s)");
            }
            Ok(())
        })
    });

    let guard = ShutdownGuard::new();
    let mut supervisor = WorkerSupervisor::new(guard.clone());
    supervisor
        .register(JobSpec {
            name: "render-cut-sweep".to_owned(),
            trigger: Trigger::Interval(Duration::from_secs(5)),
            closure: sweep,
            failure_budget: 5,
            leader: false,
            // First sweep fires at loop start: a submitted job processes
            // without waiting out the first full interval.
            fire_at_start: true,
            drain_pass: false,
            use_breaker: true,
            on_degraded: None,
        })
        .expect("register render sweep");

    eprintln!("[offcut-cut] worker online, spool = {}", spool.display());
    let supervisor = Arc::new(supervisor);
    let runner = tokio::spawn(Arc::clone(&supervisor).run());

    // Ctrl-C signals the guard; the supervisor drains and reports.
    let _ = tokio::signal::ctrl_c().await;
    guard.shutdown();
    let report = runner.await.expect("supervisor join");
    eprintln!("[offcut-cut] drained: {} job(s) supervised", report.per_job.len());
}

async fn exec_main(args: &[String]) {
    let job_path = args
        .iter()
        .position(|a| a == "--job")
        .and_then(|p| args.get(p + 1))
        .expect("exec requires --job <file>");
    let raw = std::fs::read_to_string(job_path).expect("job file");
    let job: CutJob = serde_json::from_str(&raw).expect("job JSON");
    match ffmpeg_cut(&job).await {
        Ok(()) => println!("{}", serde_json::to_string(&CutResult { ok: true, output: Some(job.output.clone()), error: None }).unwrap()),
        Err(e) => {
            println!("{}", serde_json::to_string(&CutResult { ok: false, output: None, error: Some(e.clone()) }).unwrap());
            std::process::exit(1);
        }
    }
}

fn submit_main(args: &[String]) {
    let get = |name: &str| -> Option<String> {
        args.iter().position(|a| a == name).and_then(|p| args.get(p + 1)).cloned()
    };
    let spool = PathBuf::from(get("--spool").unwrap_or_else(|| "spool".into()));
    let job = CutJob {
        input: get("--input").expect("--input required"),
        start: get("--start").expect("--start required"),
        end: get("--end").expect("--end required"),
        output: get("--output").expect("--output required"),
        reencode: !args.iter().any(|a| a == "--stream-copy"),
    };
    std::fs::create_dir_all(spool.join("pending")).expect("pending dir");
    let name = format!("cut-{}.json", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    std::fs::write(spool.join("pending").join(&name),
        serde_json::to_string_pretty(&job).expect("serialize"))
        .expect("write job");
    println!("{}", serde_json::json!({"submitted": name}));
}
