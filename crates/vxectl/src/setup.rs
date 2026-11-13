/*
 * Dynamic shell auto-completions engine (Clap AST introspection + live USB device discovery),
 * automatic shell profile setup, and Linux udev rules installation.
 */

use crate::cli::Cli;
use crate::device::get_device_aliases;
use atk_device::{ConnectionMode, DeviceScanner};
use atk_hid::HidManager;
use clap::CommandFactory;

/* Generates the shell hook script that delegates Tab-completion to `vxectl _complete` */
pub fn generate_completion_script(shell: &str) -> Result<String, String> {
    match shell.to_lowercase().as_str() {
        "powershell" | "pwsh" => Ok(r#"# PowerShell dynamic completion for vxectl
Register-ArgumentCompleter -Native -CommandName @('vxectl', 'vxectl.exe') -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $exe = $commandAst.CommandElements[0].Extent.Text
    $tokens = @($commandAst.CommandElements | Select-Object -Skip 1 | ForEach-Object { $_.Extent.Text })
    if ($wordToComplete -eq '') {
        $tokens += '__EMPTY__'
    }
    & $exe _complete --shell powershell -- @tokens 2>$null | ForEach-Object {
        $parts = $_ -split "`t", 2
        $val = $parts[0]
        $tip = if ($parts.Count -gt 1 -and $parts[1] -ne '') { $parts[1] } else { $val }
        [System.Management.Automation.CompletionResult]::new($val, $val, 'ParameterValue', $tip)
    }
}"#
        .to_string()),
        "bash" => Ok(r#"# Bash dynamic completion for vxectl
_vxectl_completions() {
    local IFS=$'\n'
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local args=("${COMP_WORDS[@]:1}")
    if [[ -z "$cur" ]]; then
        args[${#args[@]}-1]="__EMPTY__"
    fi
    local candidates
    candidates=$(vxectl _complete --shell bash -- "${args[@]}" 2>/dev/null)
    COMPREPLY=( $(compgen -W "$candidates" -- "$cur") )
    return 0
}
complete -F _vxectl_completions vxectl"#
        .to_string()),
        "zsh" => Ok(r#"#compdef vxectl
_vxectl() {
    local -a completions args
    local val desc
    args=("${words[@]:1}")
    if [[ -z "${words[CURRENT]}" ]]; then
        args+=("__EMPTY__")
    fi
    while IFS=$'\t' read -r val desc; do
        if [[ -n "$val" ]]; then
            if [[ -n "$desc" ]]; then
                completions+=("${val}:${desc}")
            else
                completions+=("${val}")
            fi
        fi
    done < <(vxectl _complete --shell zsh -- "${args[@]}" 2>/dev/null)
    _describe 'vxectl' completions
}"#
        .to_string()),
        "fish" => Ok(r#"# Fish dynamic completion for vxectl
complete -c vxectl -f -a "(vxectl _complete --shell fish -- (commandline -cop)[2..-1] (string replace -r '^$' '__EMPTY__' -- (commandline -ct)) 2>/dev/null)""#
        .to_string()),
        other => Err(format!(
            "Unknown shell '{}'. Supported shells: powershell, bash, zsh, fish",
            other
        )),
    }
}

pub fn handle_completions(shell: &str) {
    match generate_completion_script(shell) {
        Ok(script) => println!("{script}"),
        Err(err) => {
            eprintln!("[!] {err}");
            std::process::exit(1);
        }
    }
}

/* Collects completion candidates as (value, description) pairs */
pub fn compute_completion_candidates(raw_words: &[String]) -> Vec<(String, String)> {
    let words: Vec<String> = if raw_words.is_empty() {
        vec![String::new()]
    } else {
        raw_words
            .iter()
            .map(|w| if w == "__EMPTY__" { String::new() } else { w.clone() })
            .collect()
    };
    let cur = words.last().map(|s| s.as_str()).unwrap_or("");
    let prev = if words.len() >= 2 {
        words[words.len() - 2].as_str()
    } else {
        ""
    };

    /* 1. If previous token is -d or --device, suggest live connected USB mice + known model aliases */
    if prev == "-d" || prev == "--device" {
        let mut items: Vec<(String, String)> = Vec::new();

        /* Fast live USB HID descriptor scan (takes ~1-2 ms without blocking EEPROM reads) */
        if let Ok(hid_mgr) = HidManager::new() {
            let channels = DeviceScanner::scan_config_channels(&hid_mgr);
            for (idx, target) in channels.iter().enumerate() {
                let (model_name, mode) = DeviceScanner::resolve_device_name_and_mode(
                    target.info.vendor_id,
                    target.info.product_id,
                    target.info.product_string.as_deref(),
                );
                let mode_str = match mode {
                    ConnectionMode::Wired => "Wired USB-C",
                    ConnectionMode::WirelessDongle => "2.4GHz Wireless",
                    ConnectionMode::Bluetooth => "Bluetooth",
                    ConnectionMode::Unknown => "USB",
                };
                items.push((
                    idx.to_string(),
                    format!("[Connected #{idx}] {model_name} ({mode_str})"),
                ));
                for alias in get_device_aliases(&model_name)
                    .into_iter()
                    .filter(|a| a.len() <= 7)
                    .take(2)
                {
                    if !items.iter().any(|(v, _)| v == &alias) {
                        items.push((
                            alias,
                            format!("[Connected] {model_name} ({mode_str})"),
                        ));
                    }
                }
            }
        }

        /* Standard short aliases for all supported mouse families */
        let static_aliases = [
            ("r1pm", "VXE R1 Pro Max"),
            ("r1p", "VXE R1 Pro"),
            ("r1sep", "VXE R1 SE+"),
            ("r1se", "VXE R1 SE"),
            ("f1pm", "VGN Dragonfly F1 Pro Max"),
            ("f1p", "VGN Dragonfly F1 Pro"),
            ("f1m", "VGN Dragonfly F1 MOBA"),
            ("madr", "ATK Mad R"),
            ("madrm", "ATK Mad R Major"),
            ("madrp", "ATK Mad R+"),
            ("x1pm", "ATK Blazing Sky X1 Pro Max"),
            ("x1p", "ATK Blazing Sky X1 Pro"),
            ("x1u", "ATK Blazing Sky X1 Ultra"),
        ];
        for (alias, desc) in static_aliases {
            if !items.iter().any(|(v, _)| v == alias) {
                items.push((alias.to_string(), desc.to_string()));
            }
        }

        return filter_prefix(items, cur);
    }

    /* Find active subcommand and its position in `words` (skipping `-d <val>`) */
    let cmd = Cli::command();
    let mut active_subcmd: Option<(String, usize)> = None;
    let mut i = 0;
    let end_idx = words.len().saturating_sub(1);
    while i < end_idx {
        let w = words[i].as_str();
        if w == "-d" || w == "--device" {
            i += 2;
            continue;
        }
        if w.starts_with('-') {
            i += 1;
            continue;
        }
        /* Match against Clap subcommands and their aliases */
        for sub in cmd.get_subcommands() {
            if sub.get_name() == w || sub.get_all_aliases().any(|a| a == w) {
                active_subcmd = Some((sub.get_name().to_string(), i));
                break;
            }
        }
        if active_subcmd.is_some() {
            break;
        }
        i += 1;
    }

    /* 2. If no subcommand has been entered yet, introspect Clap `Cli::command()` for subcommands & global flags */
    let Some((sub_name, sub_idx)) = active_subcmd else {
        let mut items = Vec::new();
        for sub in cmd.get_subcommands() {
            if sub.is_hide_set() {
                continue;
            }
            let name = sub.get_name().to_string();
            let about = sub
                .get_about()
                .map(|s| s.to_string())
                .unwrap_or_else(|| name.clone());
            items.push((name, about));
        }
        items.push(("-d".to_string(), "Target specific mouse by index or short alias".to_string()));
        items.push(("--device".to_string(), "Target specific mouse by index or short alias".to_string()));
        items.push(("--help".to_string(), "Print help information".to_string()));
        items.push(("--version".to_string(), "Print version information".to_string()));
        return filter_prefix(items, cur);
    };

    /* Count how many positional arguments have been typed after the subcommand */
    let args_after_sub = &words[sub_idx + 1..end_idx];
    let positional_count = args_after_sub
        .iter()
        .filter(|a| !a.starts_with('-'))
        .count();

    /* 3. Subcommand-specific smart parameter value completions */
    let candidates: Vec<(String, String)> = match sub_name.as_str() {
        "set-polling" => vec![
            ("125".into(), "125 Hz (8.0 ms power-saving)".into()),
            ("250".into(), "250 Hz (4.0 ms standard)".into()),
            ("500".into(), "500 Hz (2.0 ms competitive)".into()),
            ("1000".into(), "1000 Hz (1.0 ms eSports standard)".into()),
            ("2000".into(), "2000 Hz (0.5 ms high-speed)".into()),
            ("4000".into(), "4000 Hz (0.25 ms 4K dongle)".into()),
            ("8000".into(), "8000 Hz (0.125 ms 8K dongle)".into()),
        ],
        "set-stage" => vec![
            ("1".into(), "Activate DPI Stage 1".into()),
            ("2".into(), "Activate DPI Stage 2".into()),
            ("3".into(), "Activate DPI Stage 3".into()),
            ("4".into(), "Activate DPI Stage 4".into()),
            ("5".into(), "Activate DPI Stage 5".into()),
            ("6".into(), "Activate DPI Stage 6".into()),
        ],
        "set-dpi" => {
            if prev == "-y" || prev == "--y" {
                dpi_presets()
            } else if positional_count == 0 {
                vec![
                    ("1".into(), "Stage 1".into()),
                    ("2".into(), "Stage 2".into()),
                    ("3".into(), "Stage 3".into()),
                    ("4".into(), "Stage 4".into()),
                    ("5".into(), "Stage 5".into()),
                    ("6".into(), "Stage 6".into()),
                ]
            } else if positional_count == 1 {
                dpi_presets()
            } else {
                vec![("--y".into(), "Separate Y-axis DPI value".into())]
            }
        }
        "set-lod" => vec![
            ("1".into(), "1.0 mm (Low lift-off distance)".into()),
            ("2".into(), "2.0 mm (High lift-off distance)".into()),
        ],
        "set-debounce" => vec![
            ("0".into(), "0 ms (Instant response)".into()),
            ("1".into(), "1 ms (Ultra-low latency)".into()),
            ("2".into(), "2 ms (Competitive)".into()),
            ("4".into(), "4 ms (Recommended default)".into()),
            ("8".into(), "8 ms (Safe anti-doubleclick)".into()),
            ("12".into(), "12 ms".into()),
            ("15".into(), "15 ms".into()),
            ("20".into(), "20 ms (Maximum stabilization)".into()),
        ],
        "set-motionsync" | "set-straightline" | "set-ripple" | "set-longrange" => vec![
            ("on".into(), "Enable parameter (ON)".into()),
            ("off".into(), "Disable parameter (OFF)".into()),
        ],
        "set-sleep" => vec![
            ("30s".into(), "30 seconds idle sleep".into()),
            ("1m".into(), "1 minute idle sleep (default)".into()),
            ("2m".into(), "2 minutes idle sleep".into()),
            ("3m".into(), "3 minutes idle sleep".into()),
            ("5m".into(), "5 minutes idle sleep".into()),
            ("20m".into(), "20 minutes idle sleep".into()),
            ("25m".into(), "25 minutes idle sleep".into()),
            ("30m".into(), "30 minutes idle sleep".into()),
        ],
        "set-dpilight" => vec![
            ("off".into(), "Turn off DPI indicator LED".into()),
            ("steady".into(), "Always On (active stage RGB color)".into()),
            ("breathing".into(), "Breathing pulse (active stage RGB color)".into()),
        ],
        "battery" => vec![("--json".into(), "Format battery telemetry as JSON".into())],
        "completions" => vec![
            ("powershell".into(), "PowerShell completion script".into()),
            ("bash".into(), "Bash completion script".into()),
            ("zsh".into(), "Zsh completion script".into()),
            ("fish".into(), "Fish completion script".into()),
        ],
        "export" | "import" => json_files_in_cwd(),
        _ => Vec::new(),
    };

    filter_prefix(candidates, cur)
}

fn dpi_presets() -> Vec<(String, String)> {
    vec![
        ("400".into(), "400 DPI (Low-sens tactical FPS)".into()),
        ("800".into(), "800 DPI (Pro eSports standard)".into()),
        ("1200".into(), "1200 DPI (Balanced)".into()),
        ("1600".into(), "1600 DPI (High precision)".into()),
        ("2400".into(), "2400 DPI".into()),
        ("3200".into(), "3200 DPI (4K desktop)".into()),
        ("6400".into(), "6400 DPI".into()),
        ("12800".into(), "12800 DPI".into()),
        ("26000".into(), "26000 DPI (PAW3395 max)".into()),
    ]
}

fn json_files_in_cwd() -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    out.push((name.to_string(), "JSON hardware profile".to_string()));
                }
            }
        }
    }
    if out.is_empty() {
        out.push(("open_atk_profile.json".into(), "OpenATK hardware profile JSON".into()));
    }
    out
}

fn filter_prefix(items: Vec<(String, String)>, prefix: &str) -> Vec<(String, String)> {
    let p_lower = prefix.to_lowercase();
    items
        .into_iter()
        .filter(|(val, _)| val.to_lowercase().starts_with(&p_lower))
        .collect()
}

/* Prints dynamic completion results formatted for the requesting shell */
pub fn handle_dynamic_complete(shell: &str, words: &[String]) {
    let candidates = compute_completion_candidates(words);
    let is_bash = shell.eq_ignore_ascii_case("bash") || shell.eq_ignore_ascii_case("plain");

    for (val, desc) in candidates {
        if is_bash || desc.is_empty() {
            println!("{val}");
        } else {
            println!("{val}\t{desc}");
        }
    }
}

/* Automatically install shell auto-completions into user shell profile or system directory */
pub fn handle_install_completions() {
    #[cfg(target_os = "windows")]
    {
        /* Sync current vxectl.exe into %USERPROFILE%\.cargo\bin if present so PATH always has the latest binary */
        if let (Ok(userprofile), Ok(current_exe)) = (std::env::var("USERPROFILE"), std::env::current_exe()) {
            let cargo_bin = std::path::Path::new(&userprofile).join(".cargo").join("bin");
            if cargo_bin.is_dir() {
                let target_exe = cargo_bin.join("vxectl.exe");
                if current_exe != target_exe {
                    if std::fs::copy(&current_exe, &target_exe).is_ok() {
                        println!("[+] Updated binary in PATH: {}", target_exe.display());
                    }
                }
            }
        }

        let mut profile_paths: Vec<std::path::PathBuf> = Vec::new();

        /* Query actual $PROFILE path from PowerShell (handles OneDrive / custom Documents folders) */
        for bin in ["pwsh", "powershell"] {
            if let Ok(output) = std::process::Command::new(bin)
                .args(["-NoProfile", "-NonInteractive", "-Command", "$PROFILE"])
                .output()
            {
                if output.status.success() {
                    let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !p.is_empty() {
                        let pb = std::path::PathBuf::from(p);
                        if !profile_paths.contains(&pb) {
                            profile_paths.push(pb);
                        }
                    }
                }
            }
        }

        if profile_paths.is_empty() {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                let docs_dir = std::path::Path::new(&userprofile).join("Documents");
                profile_paths.push(docs_dir.join("WindowsPowerShell").join("Microsoft.PowerShell_profile.ps1"));
                profile_paths.push(docs_dir.join("PowerShell").join("Microsoft.PowerShell_profile.ps1"));
            }
        }

        let ps_script = generate_completion_script("powershell").unwrap_or_default();
        let block = format!(
            "\n# >>> vxectl autocompletions >>>\n{}\n# <<< vxectl autocompletions <<<\n",
            ps_script
        );

        let mut installed_any = false;
        for profile_path in profile_paths {
            if let Some(parent) = profile_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let existing = std::fs::read_to_string(&profile_path).unwrap_or_default();
            /* Replace legacy completion block if present, or append new dynamic block */
            let cleaned = if let (Some(start), Some(end)) = (
                existing.find("# >>> vxectl autocompletions >>>"),
                existing.find("# <<< vxectl autocompletions <<<"),
            ) {
                let after = end + "# <<< vxectl autocompletions <<<".len();
                format!("{}{}", &existing[..start].trim_end(), &existing[after..])
            } else {
                existing
            };

            let updated = format!("{}\n{}", cleaned.trim_end(), block.trim());
            match std::fs::write(&profile_path, updated) {
                Ok(_) => {
                    println!(
                        "[+] Installed dynamic autocompletions to: {}",
                        profile_path.display()
                    );
                    installed_any = true;
                }
                Err(e) => {
                    eprintln!("[!] Could not write to {}: {e}", profile_path.display());
                }
            }
        }

        if installed_any {
            println!(
                "\n[✓] Dynamic autocompletions installed! Open a new PowerShell window to enjoy context-aware Tab completion."
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        let bash_script = generate_completion_script("bash").unwrap_or_default();
        let system_dir = std::path::Path::new("/usr/share/bash-completion/completions");
        if system_dir.is_dir() {
            let target_file = system_dir.join("vxectl");
            if std::fs::write(&target_file, &bash_script).is_ok() {
                println!(
                    "[+] Installed system-wide bash completions to {}",
                    target_file.display()
                );
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            let home_path = std::path::Path::new(&home);
            let user_shell = std::env::var("SHELL").unwrap_or_default();

            /* Fish user completions */
            if user_shell.contains("fish") {
                if let Ok(fish_script) = generate_completion_script("fish") {
                    let fish_dir = home_path.join(".config/fish/completions");
                    let _ = std::fs::create_dir_all(&fish_dir);
                    let fish_file = fish_dir.join("vxectl.fish");
                    if std::fs::write(&fish_file, fish_script).is_ok() {
                        println!("[+] Installed Fish completions to {}", fish_file.display());
                    }
                }
            }

            /* Zsh user completions */
            if user_shell.contains("zsh") {
                let zshrc = home_path.join(".zshrc");
                let line = "source <(vxectl completions zsh)\n";
                let existing = std::fs::read_to_string(&zshrc).unwrap_or_default();
                if !existing.contains("vxectl completions") {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&zshrc) {
                        let _ = f.write_all(line.as_bytes());
                        println!("[+] Added vxectl autocompletions to {}", zshrc.display());
                    }
                }
            }

            /* Bash user completions */
            let bashrc = home_path.join(".bashrc");
            let line = "source <(vxectl completions bash)\n";
            let existing = std::fs::read_to_string(&bashrc).unwrap_or_default();
            if !existing.contains("vxectl completions") {
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&bashrc) {
                    let _ = f.write_all(line.as_bytes());
                    println!("[+] Added vxectl autocompletions to {}", bashrc.display());
                }
            }
        }
    }
}

pub fn handle_install_udev() {
    #[cfg(target_os = "linux")]
    {
        let rules_path = std::path::Path::new("/etc/udev/rules.d/71-open-atk.rules");
        let rules_content = include_str!("../../../packaging/71-open-atk.rules");

        println!("[*] Installing udev rules to {}...", rules_path.display());
        match std::fs::write(rules_path, rules_content) {
            Ok(_) => {
                println!("[+] Successfully written 71-open-atk.rules!");
                println!("[*] Reloading and triggering udev rules...");
                let status1 = std::process::Command::new("udevadm")
                    .args(["control", "--reload-rules"])
                    .status();
                let status2 = std::process::Command::new("udevadm")
                    .arg("trigger")
                    .status();
                if status1.is_ok() && status2.is_ok() {
                    println!(
                        "[+] Udev rules applied successfully! All ATK/VXE mice can now be accessed without sudo."
                    );
                } else {
                    println!("[!] Rules written, but failed to reload udev automatically. Please run:");
                    println!("    sudo udevadm control --reload-rules && sudo udevadm trigger");
                }
            }
            Err(e) => {
                eprintln!("[!] Failed to write to {}: {}", rules_path.display(), e);
                eprintln!("    Please run this command with sudo:");
                eprintln!("    sudo vxectl install-udev");
                std::process::exit(1);
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        println!("[i] Udev rules are only required on Linux systems.");
        println!("    On Windows, USB HID access works natively out-of-the-box without additional drivers.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subcommand_introspection_completion() {
        let top = compute_completion_candidates(&["".to_string()]);
        assert!(top.iter().any(|(v, _)| v == "status"));
        assert!(top.iter().any(|(v, _)| v == "set-dpi"));
        assert!(top.iter().any(|(v, _)| v == "set-polling"));
        assert!(!top.iter().any(|(v, _)| v == "_complete"));

        let polling = compute_completion_candidates(&["set-polling".to_string(), "".to_string()]);
        assert!(polling.iter().any(|(v, _)| v == "1000"));
        assert!(polling.iter().any(|(v, _)| v == "4000"));

        let dpi_stage = compute_completion_candidates(&["set-dpi".to_string(), "".to_string()]);
        assert!(dpi_stage.iter().any(|(v, _)| v == "1"));

        let dpi_val = compute_completion_candidates(&["set-dpi".to_string(), "1".to_string(), "8".to_string()]);
        assert_eq!(dpi_val.len(), 1);
        assert_eq!(dpi_val[0].0, "800");
    }
}
