use clap::Parser;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::Ollama;
use std::error::Error;
use std::io::{ self, Write };
use std::path::Path;
use std::process::Command;

const SYSTEM_PROMPT: &str =
    "You are a Senior Fullstack Engineer, You are tasked to create a git commit message for the following changes, your commit messages should be clear and concise Conventional Commits use specific types to describe changes and their impact, aligning with Semantic Versioning:

fix: Bug fix, correlates with a PATCH version update.
feat: New feature, correlates with a MINOR version update.
BREAKING CHANGE: Indicates a major API change, correlating with a MAJOR version update. Can be indicated with a ! or a BREAKING CHANGE: footer.
Other types (e.g., build:, chore:, docs:, style:, refactor:, perf:, test:) provide context but don't affect versioning unless marked as breaking.

Examples:

Use feat!: or BREAKING CHANGE: to signal major changes.
Scopes (e.g., feat(api):) provide additional context.
Commit types and scopes help maintainers and automated systems understand the nature and impact of changes. ";
const MODEL: &str = "llama3.1:latest";

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
    // generate_commit_message(&changes).await?;
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
    // let unstaged_output = Command::new("git")
    //     .arg("-C")
    //     .arg(target)
    //     .arg("diff")
    //     .output()
    //     .expect("Failed to get unstaged git diff");

    let staged_output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .arg("--cached") // This is equivalent to `--staged`
        .output()
        .expect("Failed to get staged git diff");

    // Convert the output to a string
    // let unstaged_changes = String::from_utf8_lossy(&unstaged_output.stdout).to_string();
    let staged_changes = String::from_utf8_lossy(&staged_output.stdout).to_string();

    // Combine the unstaged and staged changes
    // format!("{}{}", unstaged_changes, staged_changes)
    staged_changes
}

// Generate a commit message for the changes
async fn generate_commit_message(changes: &str) -> Result<String, Box<dyn Error>> {
    // Send the changes to the LLM for diagnosis
    let mut commit_message = send_to_llm_for_diagnosis(changes).await?;
    loop {
        // Print the generated commit message
        println!("\nGenerated Commit Message:\n{}", commit_message);

        // Ask the user if they like the generated commit message
        print!("Do you like this commit message? (yes/no): ");
        io::stdout().flush()?; // Flush to ensure the prompt appears

        let mut input = String::new();
        io::stdin().read_line(&mut input)?; // Read user input

        // If the user likes the generated commit message, return it
        if input.trim().eq_ignore_ascii_case("yes") {
            return Ok(commit_message);
        } else {
            println!("Generating a new commit message...");
            let changes_revamp = format!(
                "Our commit message wasn't good, try again and be sure to use the following instructions: {}{}",
                SYSTEM_PROMPT,
                commit_message
            );
            commit_message = send_to_llm_for_diagnosis(&changes_revamp).await?; // Pass the reference to the new string
        }
    }
}

// Commit the changes with the generated commit message
fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Run the command `git add .` in the target directory
    // Command::new("git").arg("-C").arg(target).arg("add").arg(".").status()?;

    // Run the command `git commit -m <message>` in the target directory
    Command::new("git").arg("-C").arg(target).arg("commit").arg("-m").arg(message).status()?;

    // Ask the user if they want to push the commit
    print!("Do you want to push the commit? (yes/no): ");
    io::stdout().flush()?; // Flush to ensure the prompt appears before input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // Check user input
    if input.trim().eq_ignore_ascii_case("yes") {
        // Run the command `git push` in the target directory
        Command::new("git").arg("-C").arg(target).arg("push").status()?;
        println!("Changes committed and pushed.");
    } else {
        println!("Changes committed but not pushed.");
    }

    Ok(())
}

// Send the changes to the LLM for diagnosis
async fn send_to_llm_for_diagnosis(changes: &str) -> Result<String, anyhow::Error> {
    let ollama = Ollama::new("http://localhost".to_string(), 11434);

    // ask user the nature of the changes
    print!(
        "What is the nature of these changes? 
    - (feat): For adding new features.
    - (fix): For bug fixes only.
    - (docs): For changes related to documentation, including `.md` file updates or code comments.
    - (style): For changes that do not affect code functionality, such as formatting, spacing, or style adjustments.
    - (chore): For changes related to build processes, dependencies, or maintenance tasks. 
    Type: "
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    // Add the nature of the changes to the prompt
    let prompt = format!("{} for these changes:\n{} type:{}", SYSTEM_PROMPT, changes, input);

    // Feed options for the AI
    let options = GenerationOptions::default()
        .temperature(0.2)
        .repeat_penalty(1.5)
        .top_k(25)
        .top_p(0.25);

    // Create a request & send for the LLM
    let res = ollama
        .generate(GenerationRequest::new(MODEL.to_string(), prompt).options(options)).await
        .unwrap();

    // Return the generated commit message
    Ok(res.response)
}
