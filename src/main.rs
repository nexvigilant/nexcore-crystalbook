//! Crystalbook CLI — the door into the crate (fills ∅₂).
//!
//! Subcommands:
//!   render  — produce a self-contained HTML file
//!   verify  — check integrity and seal chain
//!   seal    — seal the document at current state
//!   info    — show document metadata and structure
//!   new     — generate the canonical Crystalbook v2.0 file

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::process::ExitCode;

use nexcore_crystalbook::document::crystalbook_v2;
use nexcore_crystalbook::execute::{StaticExecutor, execute_all};
use nexcore_crystalbook::io;
use nexcore_crystalbook::render::render_to_html;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    let cmd = args.get(1).map(String::as_str);
    match cmd {
        Some("new") => cmd_new(&args[2..]),
        Some("render") => cmd_render(&args[2..]),
        Some("verify") => cmd_verify(&args[2..]),
        Some("seal") => cmd_seal(&args[2..]),
        Some("info") => cmd_info(&args[2..]),
        Some("execute") => cmd_execute(&args[2..]),
        Some("help") | Some("--help") | Some("-h") | None => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some(unknown) => {
            eprintln!("Unknown command: {unknown}");
            print_usage();
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("Crystalbook — immutable scientific documents");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  crystalbook <command> [args]");
    eprintln!();
    eprintln!("COMMANDS:");
    eprintln!("  new     [path]         Generate the canonical Crystalbook v2.0");
    eprintln!("  render  <path> [out]   Render .crystalbook to HTML");
    eprintln!("  verify  <path>         Verify integrity and seal chain");
    eprintln!("  seal    <path> <name>  Seal the document");
    eprintln!("  info    <path>         Show document structure");
    eprintln!("  execute <path>         Execute all cells and save");
    eprintln!("  help                   Show this help");
}

// ── new ─────────────────────────────────────────────────

fn cmd_new(args: &[String]) -> ExitCode {
    let path = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crystalbook-v2.crystalbook"));

    let doc = crystalbook_v2();

    match io::save(&doc, &path) {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ── render ──────────────────────────────────────────────

fn cmd_render(args: &[String]) -> ExitCode {
    let Some(input) = args.first() else {
        eprintln!("Usage: crystalbook render <input.crystalbook> [output.html]");
        return ExitCode::FAILURE;
    };

    let doc = match io::load(&PathBuf::from(input)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let html = render_to_html(&doc);

    let output = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        let stem = PathBuf::from(input)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "crystalbook".to_string());
        PathBuf::from(format!("{stem}.html"))
    });

    match std::fs::write(&output, &html) {
        Ok(()) => {
            println!(
                "Rendered {} → {} ({} bytes)",
                input,
                output.display(),
                html.len(),
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error writing {}: {e}", output.display());
            ExitCode::FAILURE
        }
    }
}

// ── verify ──────────────────────────────────────────────

fn cmd_verify(args: &[String]) -> ExitCode {
    let Some(input) = args.first() else {
        eprintln!("Usage: crystalbook verify <path.crystalbook>");
        return ExitCode::FAILURE;
    };

    let doc = match io::load(&PathBuf::from(input)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let validation = doc.validate();
    let integrity = doc.verify_integrity();
    let chain = doc.seals.verify_chain();

    println!("Document: {}", doc.metadata.title);
    println!("Version:  {}", doc.metadata.version);
    println!("Cells:    {}", doc.cell_count());
    println!(
        "Root:     {}",
        &doc.merkle_root[..16.min(doc.merkle_root.len())]
    );
    println!();
    println!("Integrity:  {}", if integrity { "PASS" } else { "FAIL" });
    println!(
        "Validation: {}",
        if validation.is_valid() {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!("Seal chain: {chain:?}");
    println!("Sealed:     {}", if doc.is_sealed() { "YES" } else { "NO" },);

    if let nexcore_crystalbook::document::ValidationResult::Invalid(errors) = validation {
        println!();
        for err in &errors {
            println!("  ERROR: {err}");
        }
    }

    if integrity && doc.validate().is_valid() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ── seal ────────────────────────────────────────────────

fn cmd_seal(args: &[String]) -> ExitCode {
    let (Some(input), Some(signer)) = (args.first(), args.get(1)) else {
        eprintln!("Usage: crystalbook seal <path.crystalbook> <signer-name>");
        return ExitCode::FAILURE;
    };

    let path = PathBuf::from(input);
    let mut doc = match io::load(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let seal_id = doc.seal(signer);
    println!("Sealed by {signer}");
    println!("Seal ID: {}", seal_id);
    println!("Chain length: {}", doc.seals.len());

    match io::save(&doc, &path) {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error saving: {e}");
            ExitCode::FAILURE
        }
    }
}

// ── info ────────────────────────────────────────────────

fn cmd_info(args: &[String]) -> ExitCode {
    let Some(input) = args.first() else {
        eprintln!("Usage: crystalbook info <path.crystalbook>");
        return ExitCode::FAILURE;
    };

    let doc = match io::load(&PathBuf::from(input)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("Title:    {}", doc.metadata.title);
    if let Some(ref sub) = doc.metadata.subtitle {
        println!("Subtitle: {sub}");
    }
    println!("Author:   {}", doc.metadata.author);
    println!("Version:  {}", doc.metadata.version);
    println!("Created:  {}", doc.metadata.created);
    println!("Amended:  {}", doc.metadata.last_amended);
    println!("Format:   v{}", doc.crystalbook_version);
    println!();
    println!("Cells:    {}", doc.cell_count());
    println!(
        "  Text:       {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::Text)
    );
    println!(
        "  Law:        {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::Law)
    );
    println!(
        "  RustCode:   {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::RustCode)
    );
    println!(
        "  ShellCode:  {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::ShellCode)
    );
    println!(
        "  PvdslCode:  {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::PvdslCode)
    );
    println!(
        "  Diagnostic: {}",
        doc.count_by_type(&nexcore_crystalbook::cell::CellType::Diagnostic)
    );
    println!();
    println!("Merkle:   {}", doc.merkle_root);
    println!("Sealed:   {}", if doc.is_sealed() { "YES" } else { "NO" });
    println!("Seals:    {}", doc.seals.len());

    if let Some(seal) = doc.seals.latest() {
        println!("Latest:   {} by {}", seal.short_id(), seal.signer);
    }

    ExitCode::SUCCESS
}

// ── execute ─────────────────────────────────────────────

fn cmd_execute(args: &[String]) -> ExitCode {
    let Some(input) = args.first() else {
        eprintln!("Usage: crystalbook execute <path.crystalbook>");
        return ExitCode::FAILURE;
    };

    let path = PathBuf::from(input);
    let mut doc = match io::load(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error loading {input}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let executor = StaticExecutor;
    match execute_all(&mut doc.cells, &executor) {
        Ok(result) => {
            println!("{result}");
            doc.recompute_merkle();
            match io::save(&doc, &path) {
                Ok(save_result) => {
                    println!("{save_result}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error saving: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            eprintln!("Execution error: {e}");
            ExitCode::FAILURE
        }
    }
}
