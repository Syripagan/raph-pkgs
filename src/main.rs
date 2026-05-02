use std::process::Command;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::collections::HashSet;
use colored::Colorize;

// Structure of the package file
#[derive(Debug)]
struct Package {
    name: String,
    version: String,
    description: String,
    source: String,
    build_system: BuildSystem,
    depends: Vec<String>,
    configure_args: Vec<String>,
}

#[derive(Debug)]
enum BuildSystem {
    Autotools,
    Cmake,
    Meson,
    Cargo,
    Python,
    Make,
    Manual {
        build_commands: Vec<String>,
        install_command: String,
    },
}

// Parsing packages
fn parse_package(path: &str) -> Result<Package, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path, e))?;

    let mut name = String::new();
    let mut version = String::new();
    let mut description = String::new();
    let mut source = String::new();
    let mut build_system_str = String::new();
    let mut depends = Vec::new();
    let mut configure_args: Vec<String> = Vec::new();
    let mut build_commands: Vec<String> = Vec::new();
    let mut install_command = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with('[') || line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            let value = value.strip_prefix('"').and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value)
                .to_string();

            match key {
                "name"            => name = value,
                "version"         => version = value,
                "description"     => description = value,
                "source"          => source = value,
                "system"          => build_system_str = value,
                "depends"         => {
                    depends = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "configure_args"  => {
                    configure_args = value
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                "build_commands"  => {
                    build_commands = value
                        .split("&&")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "install_command" => install_command = value,
                _ => {}
            }
        }
    }

    let build_system = match build_system_str.as_str() {
        "autotools" => BuildSystem::Autotools,
        "cmake"     => BuildSystem::Cmake,
        "meson"     => BuildSystem::Meson,
        "cargo"     => BuildSystem::Cargo,
        "python"    => BuildSystem::Python,
        "make"      => BuildSystem::Make,
        "manual"    => {
            if build_commands.is_empty() {
                return Err("manual build system requires 'build_commands' field".to_string());
            }
            if install_command.is_empty() {
                return Err("manual build system requires 'install_command' field".to_string());
            }
            BuildSystem::Manual { build_commands, install_command }
        }
        other => return Err(format!("unknown build system: '{}'", other)),
    };

    if name.is_empty() || source.is_empty() {
        return Err("name and source are required fields in package".to_string());
    }

    Ok(Package {
        name,
        version,
        description,
        source,
        build_system,
        depends,
        configure_args,
    })
}

// Fetch package (local → remote)
const RAD_REPO_RAW: &str =
    "https://raw.githubusercontent.com/Syripagan/radpkg/main";

fn fetch_package(pkg_name: &str) -> Result<String, String> {
    let local_path = format!("{}.toml", pkg_name);

    if Path::new(&local_path).exists() {
        println!("[rad] using local {}", local_path);
        return Ok(local_path);
    }

    let url  = format!("{}/{}.toml", RAD_REPO_RAW, pkg_name);
    let dest = format!("/tmp/rad/recipes/{}.toml", pkg_name);
    fs::create_dir_all("/tmp/rad/recipes").unwrap();

    println!("[rad] fetching recipe from {}...", url);
    let status = Command::new("wget")
        .args(["-q", "-O", &dest, &url])
        .status()
        .map_err(|e| format!("wget failed: {}", e))?;

    if !status.success() {
        return Err(format!("couldn't find package '{}' locally or in remote repo.", pkg_name));
    }

    Ok(dest)
}

// Download and extract sources
fn download_and_extract(pkg: &Package) -> Result<String, String> {
    let work_dir = format!("/tmp/rad/build/{}", pkg.name);
    fs::create_dir_all(&work_dir)
        .map_err(|e| format!("cannot create build dir: {}", e))?;

    if pkg.source.ends_with(".git")
        || (pkg.source.contains("github.com") && !pkg.source.contains(".tar"))
    {
        println!("[rad] git detected. Cloning {}...", pkg.source);
        let status = Command::new("git")
            .args(["clone", "--recursive", &pkg.source, &work_dir])
            .status()
            .map_err(|e| format!("git clone failed: {}", e))?;
        if !status.success() {
            return Err("git clone failed".to_string());
        }
        return Ok(work_dir);
    }
    let archive_name = pkg.source.split('/').last().unwrap_or("source.tar.gz");
    let archive_path = format!("{}/{}", work_dir, archive_name);

    println!("[rad] downloading {}...", pkg.source);
    let status = Command::new("wget")
        .args(["-c", &pkg.source, "-O", &archive_path])
        .status()
        .map_err(|e| format!("[rad] {} download failed: {}", "error:".red(), e))?;
    if !status.success() {
        return Err("download failed".to_string());
    }

    println!("[rad] Extracting {}...", archive_name);
    if archive_path.ends_with(".zip") {
        let status = Command::new("unzip")
            .args([&archive_path, "-d", &work_dir])
            .status()
            .map_err(|e| format!("[rad] {} extraction failed: {}", "error:".red(), e))?;
        if !status.success() {
            return Err("extraction failed".to_string());
        }
    }
        
    else {
        let status = Command::new("tar")
            .args(["-xf", &archive_path, "-C", &work_dir])
            .status()
            .map_err(|e| format!("[rad] {} extraction failed: {}", "error:".red(), e))?;
        if !status.success() {
            return Err("extraction failed".to_string());
        }
    }

    let versioned = format!("{}/{}-{}", work_dir, pkg.name, pkg.version);
    let plain     = format!("{}/{}", work_dir, pkg.name);

    if Path::new(&versioned).exists() {
        Ok(versioned)
    } else if Path::new(&plain).exists() {
        Ok(plain)
    } else {
        let entry = fs::read_dir(&work_dir)
            .map_err(|e| e.to_string())?
            .flatten()
            .find(|e| e.path().is_dir());
        entry
            .map(|e| e.path().to_string_lossy().to_string())
            .ok_or_else(|| "yes, i am stupid and could not find extracted source directory".to_string())
    }
}

// Build, install
fn build_and_install(
    pkg: &Package,
    src_dir: &str,
    prefix: &str,
    dest_dir: &str,
) -> Result<(), String> {
    fs::create_dir_all(dest_dir).unwrap();

    match &pkg.build_system {

        BuildSystem::Autotools => {
            println!("[rad] Build system: autotools");
            let mut cmd = Command::new("./configure");
            cmd.arg(format!("--prefix={}", prefix)).current_dir(src_dir);
            for arg in &pkg.configure_args { cmd.arg(arg); }
            run_cmd(cmd, "configure")?;
            run_cmd(make_cmd(src_dir, &["-j4"]), "make")?;
            run_cmd(
                make_cmd(src_dir, &[&format!("DESTDIR={}", dest_dir), "install"]),
                "make install",
            )?;
        }

        BuildSystem::Make => {
            println!("[rad] Build system: make");
            let mut args: Vec<String> = vec!["-j4".into()];
            for arg in &pkg.configure_args { args.push(arg.clone()); }
            let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            run_cmd(make_cmd(src_dir, &args_ref), "make")?;
            run_cmd(
                make_cmd(src_dir, &[
                    &format!("DESTDIR={}", dest_dir),
                    &format!("PREFIX={}", prefix),
                    "install",
                ]),
                "make install",
            )?;
        }

        BuildSystem::Cmake => {
            println!("[rad] Build system: cmake + ninja");
            let build_dir = format!("{}/build", src_dir);
            fs::create_dir_all(&build_dir).unwrap();
            let mut cmd = Command::new("cmake");
            cmd.arg("..")
               .arg("-GNinja")
               .arg(format!("-DCMAKE_INSTALL_PREFIX={}", prefix))
               .current_dir(&build_dir);
            for arg in &pkg.configure_args { cmd.arg(arg); }
            run_cmd(cmd, "cmake")?;
            run_cmd(ninja_cmd(&build_dir, &[]), "ninja")?;
            run_cmd(ninja_install_cmd(&build_dir, dest_dir), "ninja install")?;
        }

        BuildSystem::Meson => {
            println!("[rad] Build system: meson + ninja");
            let build_dir = format!("{}/build", src_dir);
            let mut cmd = Command::new("meson");
            cmd.arg("setup")
               .arg(&build_dir)
               .arg(format!("--prefix={}", prefix))
               .current_dir(src_dir);
            for arg in &pkg.configure_args { cmd.arg(arg); }
            run_cmd(cmd, "meson setup")?;
            run_cmd(ninja_cmd(&build_dir, &[]), "ninja")?;
            run_cmd(ninja_install_cmd(&build_dir, dest_dir), "ninja install")?;
        }

        BuildSystem::Cargo => {
            println!("[rad] build system: cargo");
            let mut cmd = Command::new("cargo");
            cmd.arg("build").arg("--release").current_dir(src_dir);
            run_cmd(cmd, "cargo build")?;
            let bin_dest = format!("{}{}/bin", dest_dir, prefix);
            fs::create_dir_all(&bin_dest).unwrap();
            let bin_src = format!("{}/target/release/{}", src_dir, pkg.name);
            fs::copy(&bin_src, format!("{}/{}", bin_dest, pkg.name))
                .map_err(|e| format!("[rad] {} copy binary failed: {}", "error:".red(), e))?;
        }

        BuildSystem::Python => {
            println!("[rad] build system: python (pip)");
            let mut cmd = Command::new("pip");
            cmd.args(["install", "--prefix", prefix, "--root", dest_dir, "."])
               .current_dir(src_dir);
            run_cmd(cmd, "pip install")?;
        }

        // Manual
        BuildSystem::Manual { build_commands, install_command } => {
            println!("[rad] Build system: manual");

            for (i, cmd_str) in build_commands.iter().enumerate() {
                println!("[rad] Build step {}/{}: {}", i + 1, build_commands.len(), cmd_str);
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(cmd_str)
                    .current_dir(src_dir)
                    .status()
                    .map_err(|e| format!("[rad] {} build step failed to start: {}", "error:".red(), e))?;
                if !status.success() {
                    return Err(format!("[rad] {} build step failed: {}", "error:".red(), cmd_str));
                }
            }

            // Install
            println!("[rad] Install step: {}", install_command);
            let status = Command::new("sh")
                .arg("-c")
                .arg(install_command)
                .env("DESTDIR", dest_dir)
                .current_dir(src_dir)
                .status()
                .map_err(|e| format!("[rad] {} install step failed to start: {}", "error:".red(), e))?;
            if !status.success() {
                return Err(format!("install step failed: {}", install_command));
            }
        }
    }

    println!("[rad] build finished! Files are in {}", dest_dir);
    Ok(())
}

// Helpers
fn run_cmd(mut cmd: Command, label: &str) -> Result<(), String> {
    println!("[rad] Running: {}...", label);
    let status = cmd.status()
        .map_err(|e| format!("{} failed to start: {}", label, e))?;
    if !status.success() {
        return Err(format!("{} exited with status: {}", label, status));
    }
    Ok(())
}

fn make_cmd(dir: &str, args: &[&str]) -> Command {
    let mut c = Command::new("make");
    for a in args { c.arg(a); }
    c.current_dir(dir);
    c
}

fn ninja_cmd(dir: &str, args: &[&str]) -> Command {
    let mut c = Command::new("ninja");
    for a in args { c.arg(a); }
    c.current_dir(dir);
    c
}

fn ninja_install_cmd(build_dir: &str, dest_dir: &str) -> Command {
    let mut c = Command::new("ninja");
    c.arg("install").env("DESTDIR", dest_dir).current_dir(build_dir);
    c
}

// Cossacks registry yo
fn register_package_files(pkg_name: &str, dest_dir: &str) -> std::io::Result<()> {
    let db_path = "/var/lib/rad/installed";
    fs::create_dir_all(db_path)?;

    let manifest_path = format!("{}/{}", db_path, pkg_name);
    let mut manifest = fs::File::create(&manifest_path)?;
    let dest_path = Path::new(dest_dir);

    collect_files(dest_path, dest_path, &mut manifest)?;
    Ok(())
}

fn collect_files(root: &Path, current: &Path, manifest: &mut fs::File) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path  = entry.path();
        if path.is_dir() {
            collect_files(root, &path, manifest)?;
        } else {
            let relative = path.strip_prefix(root).unwrap();
            writeln!(manifest, "/{}", relative.display())?;
        }
    }
    Ok(())
}

// Merge image to system
fn merge_to_system(dest_dir: &str) -> Result<(), String> {
    println!("[rad] merging files to system...");
    let status = Command::new("cp")
        .args(["-af", &format!("{}/.", dest_dir), "/"])
        .status()
        .map_err(|e| format!("[rad] {} failed to run cp: {}", "error:".red(), e))?;
    if !status.success() {
        return Err("Merge failed".to_string());
    }
    Ok(())
}

// Remove
fn remove_package(pkg_name: &str) -> std::io::Result<()> {
    let manifest_path = format!("/var/lib/rad/installed/{}", pkg_name);
    if !Path::new(&manifest_path).exists() {
        println!("[rad] package {} is not installed.", pkg_name);
        return Ok(());
    }
    println!("[rad] removing package: {}...", pkg_name);
    let content = fs::read_to_string(&manifest_path)?;
    for line in content.lines() {
        if line == "/usr/share/info/dir" { continue; }
        let path = Path::new(line);
        if path.exists() {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("[rad] could not remove {}: {}", line, e);
            }
        }
    }
    fs::remove_file(&manifest_path)?;
    println!("[rad] Package {} succesfully cleaned from your fantastic system", pkg_name);
    Ok(())
}

// Install (with dependency resolution)
fn is_installed(name: &str) -> bool {
    Path::new(&format!("/var/lib/rad/installed/{}", name)).exists()
}

fn install_package(pkg_name: &str, prefix: &str, processing: &mut HashSet<String>) {
    if processing.contains(pkg_name) {
        eprintln!("[rad] Circular dependency detected: {}!", pkg_name);
        return;
    }
    if is_installed(pkg_name) {
        println!("[rad] {} is already installed, skipping.", pkg_name);
        return;
    }

    processing.insert(pkg_name.to_string());

    let rad_path = match fetch_package(pkg_name) {
        Ok(p)  => p,
        Err(e) => { eprintln!("[rad] {} {}", "error:".red(), e); processing.remove(pkg_name); return; }
    };

    let pkg = match parse_package(&rad_path) {
        Ok(p)  => p,
        Err(e) => { eprintln!("[rad] {} {}", "parse error:".red(), e); processing.remove(pkg_name); return; }
    };

    for dep in &pkg.depends {
        if !is_installed(dep) {
            println!("[rad] resolving dependency: {}", dep);
            install_package(dep, prefix, processing);
        }
    }

    println!("[rad] Package: {} {}", pkg.name, pkg.version);
    println!("[rad] Info: {}", pkg.description);
    println!("[rad] Source: {}", pkg.source);

    let src_dir = match download_and_extract(&pkg) {
        Ok(d)  => d,
        Err(e) => { eprintln!("[rad] error: {}", e); processing.remove(pkg_name); return; }
    };

    let dest_dir = format!("/tmp/rad/image/{}", pkg_name);
    let _ = fs::remove_dir_all(&dest_dir);
    fs::create_dir_all(&dest_dir).unwrap();

    if let Err(e) = build_and_install(&pkg, &src_dir, prefix, &dest_dir) {
        eprintln!("[rad] build error: {}", e);
        processing.remove(pkg_name);
        return;
    }

    println!("[rad] Indexing files for {}...", pkg_name);
    if let Err(e) = register_package_files(pkg_name, &dest_dir) {
        eprintln!("[rad] registration error: {}", e);
    }

    if let Err(e) = merge_to_system(&dest_dir) {
        eprintln!("[rad] merge error: {}", e);
        processing.remove(pkg_name);
        return;
    }

    let build_dir = format!("/tmp/rad/build/{}", pkg_name);
    let _ = fs::remove_dir_all(&build_dir);
    let _ = fs::remove_dir_all(&dest_dir);

    processing.remove(pkg_name);
    println!("[rad] Installation of '{}' finished successfully.", pkg_name);
}

// Main
fn main() {
    let args: Vec<String> = env::args().collect();
    let version = "0.2.1";
    let prefix  = "/usr";
    if args.len() < 2 {
        println!("[rad] {} please specify valid argument, to see them you should use -h or --help", "error:".red());
        return;
    }

    match args[1].as_str() {
        "-h" | "--help" => {
            println!(
                "{} v{}\n\n\
                Usage: rad [command]\n\n\
                Commands:\n\
                   -h, --help              print this menu\n\
                   -V, --version           print rad version\n\
                   -i, --install <pkg>     install a package\n\
                   -r, --remove  <pkg>     remove a package\n\
                   -l, --list              list installed packages\n\n\
                Packages are searched:\n\
                   1. Locally:   ./<pkg>.toml\n\
                   2. Remote:    {}/\u{003c}pkg\u{003e}.toml", "Radrix Automated TOML-packages Handler".bold(),
                version.yellow(), RAD_REPO_RAW
            );
        }
        "-V" | "--version" => println!("rad version: {}", version),

        "-i" | "--install" => {
            let mut processing = HashSet::new();
            match args.get(2) {
                Some(name) => install_package(name, prefix, &mut processing),
                None       => eprintln!("[rad] {} specify the package name", "error:".red()),
            }
        }

        "-r" | "--remove" => {
            match args.get(2) {
                Some(name) => {
                    if let Err(e) = remove_package(name) {
                        eprintln!("[rad] {} {}", "removal error:".red(), e);
                    }
                }
                None => eprintln!("[rad] {} specify the package name", "error:".red()),
            }
        }

        "-l" | "--list" => {
            let db_path = "/var/lib/rad/installed";
            match fs::read_dir(db_path) {
                Ok(entries) => {
                    println!("[rad] installed packages:");
                    for entry in entries.flatten() {
                        if let Ok(name) = entry.file_name().into_string() {
                            println!("  - {}", name);
                        }
                    }
                }
                Err(_) => println!("[rad] no packages installed yet."),
            }
        }

        "--pew" => println!("Я стріляю: {} {} {} {}", "ратата,".yellow(), "папапа,".blue(), "піф-паф,".red(), "йоу!".green()),
        "--hello" => println!("Ну здоров, типу \"Hello World\", кста microslop параша, погоджуєшся?"),
        other => eprintln!("[rad] {} unknown argument '{}', to see valid ones you should use -h or --help", "error:".red(), other),
    }
}
