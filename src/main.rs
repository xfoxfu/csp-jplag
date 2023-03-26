use anyhow::Result;
use clap::Parser;
use log::*;
use std::fs;
use std::path::{Path, PathBuf};

mod opt;

fn wrap_path(path: PathBuf) -> PathBuf {
    std::env::current_dir().unwrap().join(path)
}

fn main() -> Result<()> {
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "csp_jplag=INFO");
    }
    pretty_env_logger::init();

    warn!("Pascal sources are not supported for checking");

    let mut opts: opt::Opts = opt::Opts::parse();
    opts.java = wrap_path(opts.java);
    opts.jplag = wrap_path(opts.jplag);
    opts.source_dir = wrap_path(opts.source_dir);
    opts.result_dir = wrap_path(opts.result_dir);
    opts.temp_dir = wrap_path(opts.temp_dir);

    info!("using java = {:?}", &opts.java);
    info!("using jplag = {:?}", &opts.jplag);

    if fs::read_dir(&opts.temp_dir).map(|_| true).or_else(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Ok(false)
        } else {
            Err(e)
        }
    })? == true
    {
        warn!("clear temporary directory {:?}", &opts.temp_dir);
        fs::remove_dir_all(&opts.temp_dir)?;
    }
    if fs::read_dir(&opts.result_dir).map(|_| true).or_else(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Ok(false)
        } else {
            Err(e)
        }
    })? == true
    {
        warn!("clear result directory {:?}", &opts.result_dir);
        fs::remove_dir_all(&opts.result_dir)?;
    }

    info!("processing inputs from {:?}", &opts.source_dir);
    for p in opts.problems.iter() {
        info!("processing problem {:?}", p);
        let dir = Path::new(&opts.temp_dir).join(p);
        warn!("creating directory {:?}", dir);
        fs::create_dir_all(dir)?;

        for e in fs::read_dir(&opts.source_dir)? {
            let e = e?;
            debug!("Found contestant dir {:?}", e.path());
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                for e2 in fs::read_dir(e.path())? {
                    let e2 = e2?;
                    if !e2.file_type().unwrap().is_dir() {
                        continue;
                    }
                    for e3 in fs::read_dir(e2.path())? {
                        let e3 = e3?;
                        let path = e3.path();
                        debug!("Found contestant file {:?}", path);
                        let ext = path.extension().unwrap_or(std::ffi::OsStr::new(""));
                        if e3.file_type()?.is_file()
                            && (ext == "c" || ext == "cpp")
                            && path.file_stem().unwrap() == p.as_str()
                        {
                            fs::copy(
                                &path,
                                opts.temp_dir
                                    .join(p)
                                    .join(e.file_name())
                                    .with_extension(ext),
                            )?;
                        }
                    }
                }
            }
        }
        info!("Temporary directory prepared.");
    }

    for p in opts.problems.iter() {
        let proc = std::process::Command::new(&opts.java)
            .arg("-jar")
            .arg(&opts.jplag)
            .arg(opts.temp_dir.join(p))
            .arg("-l")
            .arg("cpp")
            .arg("-r")
            .arg(opts.result_dir.join(p))
            .arg("-n")
            .arg("1000")
            .spawn()
            .expect("Failed to spawn jplag");
        proc.wait_with_output().expect("Failed to execute jplag");
        info!(
            "Problem {} report written to {:?}",
            p,
            opts.result_dir.join(p)
        );
    }

    // java -jar MatrixPlag/jplag-3.0.0-SNAPSHOT-jar-with-dependencies.jar test-src -l cpp -r result -n 1000
    Ok(())
}
