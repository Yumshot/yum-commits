use clap::Parser;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use std::io::{ self, Write };
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
    *   '[short description]': Keep this concise and focused on the main change.
";
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

    // Check if the directory is a git repository
    if !is_git_repo(target) {
        eprintln!("Target directory is not a git repository.");
        return Ok(());
    }

    // Check if there are changes in the target directory
    if !has_changes(target) {
        println!("No changes to commit.");
        return Ok(());
    }

    // Get all the changes in the target directory
    let changes = get_changes(target);

    // Generate a commit message for the changes
    let commit_message = generate_commit_message(&changes).await?;

    // Commit the changes with the generated commit message
    commit_changes(target, &commit_message)?;
    Ok(())
}

// Check if the target directory is a git repository
fn is_git_repo(target: &Path) -> bool {
    // Run the command `git rev-parse --is-inside-work-tree` in the target directory
    // If the command is successful, then the directory is a git repository
    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .map_or(false, |output| output.status.success())
}

// Check if there are changes in the target directory
fn has_changes(target: &Path) -> bool {
    // Run the command `git status --porcelain` in the target directory
    // If the command is successful and the output is not empty, then there are changes
    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("Failed to run git status");
    !output.stdout.is_empty()
}

// Get all the changes in the target directory
fn get_changes(target: &Path) -> String {
    // Get the unstaged and staged changes in the target directory
    let unstaged_output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .output()
        .expect("Failed to get unstaged git diff");

    let staged_output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .arg("--cached") // This is equivalent to `--staged`
        .output()
        .expect("Failed to get staged git diff");

    // Convert the output to a string
    let unstaged_changes = String::from_utf8_lossy(&unstaged_output.stdout).to_string();
    let staged_changes = String::from_utf8_lossy(&staged_output.stdout).to_string();

    // Combine the unstaged and staged changes
    format!("{}{}", unstaged_changes, staged_changes)
}

// Generate a commit message for the changes
async fn generate_commit_message(changes: &str) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        // Send the changes to the LLM for diagnosis
        let commit_message = send_to_llm_for_diagnosis(changes).await?;

        // Print the generated commit message
        println!("\nGenerated Commit Message:\n{}", commit_message);

        // Ask the user if they like the generated commit message
        print!("Do you like this commit message? (yes/no): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        // If the user likes the generated commit message, return it
        if input.trim().eq_ignore_ascii_case("yes") {
            return Ok(commit_message);
        } else {
            println!("Generating a new commit message...");
        }
    }
}

// Commit the changes with the generated commit message
fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Run the command `git add .` in the target directory
    Command::new("git").arg("-C").arg(target).arg("add").arg(".").status()?;

    // Run the command `git commit -m <message>` in the target directory
    Command::new("git").arg("-C").arg(target).arg("commit").arg("-m").arg(message).status()?;

    // Run the command `git push` in the target directory
    Command::new("git").arg("-C").arg(target).arg("push").status()?;

    Ok(())
}

// Send the changes to the LLM for diagnosis
async fn send_to_llm_for_diagnosis(changes: &str) -> Result<String, anyhow::Error> {
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // Create a prompt for the LLM
    let prompt = format!("{} for these changes:\n{}", SYSTEM_PROMPT, changes);

    // Create a request for the LLM
    let request = GenerationRequest::new(MODEL.to_string(), prompt);

    // Send the request to the LLM
    let res = ollama.generate(request).await?;

    // Print the generated commit message
    println!("Generated Commit Message: {}", res.response);

    // Return the generated commit message
    Ok(res.response)
}
