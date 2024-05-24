use std::{
    fs::{remove_file, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use walkdir::WalkDir;

/// Simple program to remove ocr text layer from PDF files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to file or directory
    #[arg(short, long)]
    path: PathBuf,

    /// Remove the original PDF file
    #[arg(short, long, default_value_t = false)]
    delete: bool,
}

fn is_pdf(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };

    let mut buffer = [0; 4];
    if let Err(_) = file.read_exact(&mut buffer) {
        return false;
    }

    buffer == [0x25, 0x50, 0x44, 0x46]
}

fn remove_ocr(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = path
        .parent()
        .ok_or("Failed to get parent directory")?
        .join("removed-ocr");

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
        println!("Created output directory: {:?}", &output_dir);
    }

    let output_file = output_dir.join(path.file_name().ok_or("Failed to get file name")?);

    let output = Command::new("gswin64.exe")
        .arg("-o")
        .arg(output_file)
        .arg("-sDEVICE=pdfwrite")
        .arg("-dFILTERTEXT")
        .arg(&path)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to process")?
    }
}

fn main() {
    let args = Args::parse();

    let path_exists = args.path.exists();

    if !path_exists {
        println!("Path {} does not exist", args.path.display());
    } else {
        if args.path.is_file() == true {
            let pdf_file = args.path;
            let result = remove_ocr(&pdf_file);

            if result.is_err() {
                eprintln!("Failed to process: {:?}", &pdf_file);
            } else {
                println!("Processed: {:?}", &pdf_file);
                if args.delete == true {
                    let result = remove_file(&pdf_file);
                    if result.is_err() {
                        eprintln!("Failed to delete: {:?}", pdf_file);
                    } else {
                        println!("Deleted: {:?}", pdf_file);
                    }
                }
            }
        } else if args.path.is_dir() {
            let path = args.path;
            let mut pdf_files: Vec<PathBuf> = Vec::new();

            for entry in WalkDir::new(&path) {
                if let Ok(entry) = entry {
                    if entry.file_type().is_file() {
                        let path = entry.path();
                        if is_pdf(path) {
                            pdf_files.push(path.to_path_buf());
                        }
                    }
                }
            }

            for pdf_file in pdf_files {
                let result = remove_ocr(&pdf_file);
                if result.is_err() {
                    eprintln!("Failed to process: {:?}", &pdf_file);
                } else {
                    println!("Processed: {:?}", &pdf_file);
                    if args.delete == true {
                        let result = remove_file(&pdf_file);
                        if result.is_err() {
                            eprintln!("Failed to delete: {:?}", pdf_file);
                        } else {
                            println!("Deleted: {:?}", pdf_file);
                        }
                    }
                }
            }
        } else {
            println!("Path {} is not a file or directory", args.path.display());
        }
    }
}
