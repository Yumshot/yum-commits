use clap::Parser;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const SYSTEM_PROMPT: &str =
    "You are to act as a Senior Fullstack Engineer, Create a git commit description, Your response should only return the description, it should be in this format    (type): Use one of the following types:
           (feat): For new features.
           (fix): For bug fixes.
           (docs): For documentation changes.
           (style): For changes that don't affect the meaning of the code (e.g., formatting, spacing).
           (refactor): For refactoring existing code without changing its functionality.
           (perf): For performance improvements.
           (test): For adding or updating tests.
           (chore): For changes that are related to build, dependencies, etc.
    *   `[short description]`: Keep this concise and focused on the main change.
";
const MODEL_DIR: &str = "C:/Users/eikoo/Documents/Code/Rust/zed-intro/src/data/models/Meta-Llama-3.1-8B-Instruct-Q8_0.gguf";
const MODEL: &str = "llama3.1";

#[derive(Parser)]
struct Args {
    /// Target directory containing the git repo
    target: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let target = Path::new(&args.target);

    // Check if the target is a Git repository
    if !is_git_repo(target) {
        eprintln!("Target directory is not a git repository.");
        return Ok(());
    }

    // Check for uncommitted changes
    if !has_changes(target) {
        println!("No changes to commit.");
        return Ok(());
    }

    // Get the changes that need to be committed
    let changes = get_changes(target);

    let commit_message = generate_commit_message(&changes).await?;
    // generate_commit_message(&changes).await?;
    // Commit the changes
    commit_changes(target, &commit_message)?;

    // println!("Changes committed with message: {}", commit_message);

    Ok(())
}

fn is_git_repo(target: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_or(false, |output| output.status.success())
}

fn has_changes(target: &Path) -> bool {
    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("Failed to run git status");
    !output.stdout.is_empty()
}

fn get_changes(target: &Path) -> String {
    // Get unstaged changes
    let unstaged_output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .output()
        .expect("Failed to get unstaged git diff");

    // Get staged changes
    let staged_output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .arg("--cached") // This is equivalent to `--staged`
        .output()
        .expect("Failed to get staged git diff");

    // Convert outputs to strings
    let unstaged_changes = String::from_utf8_lossy(&unstaged_output.stdout).to_string();
    let staged_changes = String::from_utf8_lossy(&staged_output.stdout).to_string();

    // Combine both changes
    format!("{}{}", unstaged_changes, staged_changes)
}

async fn generate_commit_message(changes: &str) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let commit_message = send_to_llm_for_diagnosis(changes).await?;
        println!("\nGenerated Commit Message:\n{}", commit_message);

        // Ask user for feedback
        print!("Do you like this commit message? (yes/no): ");
        io::stdout().flush()?; // Flush to ensure the prompt appears before input
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // Check user input
        if input.trim().eq_ignore_ascii_case("yes") {
            return Ok(commit_message);
        } else {
            println!("Generating a new commit message...");
        }
    }
}

fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("add")
        .arg(".")
        .status()?;

    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .status()?;

    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("push")
        .status()?;

    Ok(())
}

async fn send_to_llm_for_diagnosis(changes: &str) -> Result<String, anyhow::Error> {
    // Initialize Ollama with the local model directory
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // Define the prompt using your changes
    let prompt = format!("{} for these changes:\n{}", SYSTEM_PROMPT, changes);

    // Create a generation request
    let request = GenerationRequest::new(MODEL.to_string(), prompt);

    // Generate the response from the model
    let res = ollama.generate(request).await?;

    // Print the response
    println!("Generated Commit Message: {}", res.response);

    Ok(res.response)
}
