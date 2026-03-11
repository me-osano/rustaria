//! CLI command implementations.

use anyhow::Result;

use super::args::*;
use crate::aria2::Aria2;
use crate::config::Config;
use crate::queue::JobQueue;

/// Execute a CLI command.
pub async fn execute(
    command: Command,
    _config: &Config,
    queue: &JobQueue,
    aria2: &Aria2,
) -> Result<()> {
    match command {
        Command::Add(args) => cmd_add(args, queue, aria2).await,
        Command::Pause(args) => cmd_pause(args, queue, aria2).await,
        Command::Resume(args) => cmd_resume(args, queue, aria2).await,
        Command::Remove(args) => cmd_remove(args, queue, aria2).await,
        Command::List(args) => cmd_list(args, queue).await,
        Command::Status(args) => cmd_status(args, queue, aria2).await,
        Command::Config(args) => cmd_config(args).await,
    }
}

async fn cmd_add(args: AddArgs, queue: &JobQueue, _aria2: &Aria2) -> Result<()> {
    for url in &args.urls {
        let job_id = queue
            .add(
                url,
                args.output.clone(),
                args.dir.clone(),
                args.category.clone(),
                args.tags.clone(),
            )
            .await?;

        println!("Added job {}: {}", job_id, url);

        if args.start {
            queue.start(job_id).await?;
            println!("Started job {}", job_id);
        }
    }

    Ok(())
}

async fn cmd_pause(args: PauseArgs, queue: &JobQueue, _aria2: &Aria2) -> Result<()> {
    for id in &args.ids {
        if id == "all" {
            queue.pause_all().await?;
            println!("Paused all downloads");
        } else {
            let job_id: i64 = id.parse()?;
            queue.pause(job_id).await?;
            println!("Paused job {}", job_id);
        }
    }

    Ok(())
}

async fn cmd_resume(args: ResumeArgs, queue: &JobQueue, _aria2: &Aria2) -> Result<()> {
    for id in &args.ids {
        if id == "all" {
            queue.resume_all().await?;
            println!("Resumed all downloads");
        } else {
            let job_id: i64 = id.parse()?;
            queue.resume(job_id).await?;
            println!("Resumed job {}", job_id);
        }
    }

    Ok(())
}

async fn cmd_remove(args: RemoveArgs, queue: &JobQueue, _aria2: &Aria2) -> Result<()> {
    for id in &args.ids {
        let job_id: i64 = id.parse()?;
        queue.remove(job_id, args.delete_files).await?;
        println!("Removed job {}", job_id);
    }

    Ok(())
}

async fn cmd_list(args: ListArgs, queue: &JobQueue) -> Result<()> {
    let jobs = queue.list(args.status.as_deref(), args.limit).await?;

    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&jobs)?);
        }
        "csv" => {
            println!("id,url,status,progress,size");
            for job in jobs {
                println!(
                    "{},{},{},{:.1}%,{}",
                    job.id, job.url, job.status, job.progress, job.total_size
                );
            }
        }
        _ => {
            // Table format
            println!(
                "{:<6} {:<50} {:<10} {:>8} {:>12}",
                "ID", "URL", "STATUS", "PROGRESS", "SIZE"
            );
            println!("{}", "-".repeat(90));
            for job in jobs {
                let url_display = if job.url.len() > 48 {
                    format!("{}...", &job.url[..45])
                } else {
                    job.url.clone()
                };
                println!(
                    "{:<6} {:<50} {:<10} {:>7.1}% {:>12}",
                    job.id,
                    url_display,
                    job.status,
                    job.progress,
                    format_size(job.total_size)
                );
            }
        }
    }

    Ok(())
}

async fn cmd_status(args: StatusArgs, queue: &JobQueue, aria2: &Aria2) -> Result<()> {
    let job_id: i64 = args.id.parse()?;
    let job = queue.get(job_id).await?;

    // Get real-time status from aria2 if active
    let aria2_status = if let Some(gid) = &job.gid {
        aria2.status(gid).await.ok()
    } else {
        None
    };

    match args.format.as_str() {
        "json" => {
            let output = serde_json::json!({
                "job": job,
                "aria2": aria2_status,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        _ => {
            println!("Job ID:      {}", job.id);
            println!("URL:         {}", job.url);
            println!("Status:      {}", job.status);
            println!("Progress:    {:.1}%", job.progress);
            println!("Size:        {}", format_size(job.total_size));
            println!("Downloaded:  {}", format_size(job.downloaded));

            if let Some(status) = aria2_status {
                println!("Speed:       {}/s", format_size(status.speed()));
                println!("Connections: {}", status.connections);
            }

            if let Some(category) = &job.category {
                println!("Category:    {}", category);
            }

            if !job.tags.is_empty() {
                println!("Tags:        {}", job.tags.join(", "));
            }
        }
    }

    Ok(())
}

async fn cmd_config(args: ConfigArgs) -> Result<()> {
    if args.show {
        let config_path = crate::config::default_config_path();
        let content = std::fs::read_to_string(&config_path)?;
        println!("{}", content);
    } else if args.edit {
        let config_path = crate::config::default_config_path();
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        std::process::Command::new(editor)
            .arg(&config_path)
            .status()?;
    } else if let Some(kv) = args.set {
        let parts: Vec<&str> = kv.splitn(2, '=').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid format. Use: --set key=value");
        }
        println!("Setting {}={}", parts[0], parts[1]);
        // TODO: Implement config modification
    } else if let Some(key) = args.get {
        println!("Getting value for: {}", key);
        // TODO: Implement config reading
    }

    Ok(())
}

/// Format bytes as human-readable size.
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
