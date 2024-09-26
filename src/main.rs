use clap::Parser;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::Ollama;
use std::error::Error;
use std::io::{ self, Write };
use std::path::Path;
use std::process::Command;

mod constants;
use constants::*;

#[derive(Parser)]
struct Args {
    /// Target directory containing the git repo
    target: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let target = Path::new(&args.target);

    if !is_git_repo(target) {
        eprintln!("Target directory is not a git repository.");
        return Ok(());
    }

    if !has_changes(target) {
        println!("No changes to commit.");
        return Ok(());
    }

    let changes = prompt_changes_type(target)?;
    let commit_message = generate_commit_message(&changes).await?;
    commit_changes(target, &commit_message)?;

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

fn prompt_changes_type(target: &Path) -> Result<String, Box<dyn Error>> {
    print!("{}", PROMPT_FOR_CHANGES);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let changes_type = input.trim().to_lowercase();

    let diff_type = match changes_type.as_str() {
        "staged" => "--cached",
        "unstaged" => "",
        _ => {
            println!("Invalid input, defaulting to staged changes.");
            DEFAULT_DIFF_TYPE
        }
    };

    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .arg(diff_type)
        .output()
        .expect("Failed to get git diff");

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn generate_commit_message(changes: &str) -> Result<String, Box<dyn Error>> {
    let mut commit_message = send_to_llm_for_diagnosis(changes).await?;
    loop {
        println!("\nGenerated Commit Message:\n{}", commit_message);
        print!("{}", COMMIT_PROMPT);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("yes") {
            return Ok(commit_message);
        }

        println!("Generating a new commit message...");
        let changes_revamp = format!(
            "Our commit message wasn't good, most likely too long or not correct to the context, try again and be sure to use the following instructions: {}{}",
            SYSTEM_PROMPT,
            commit_message
        );
        commit_message = send_to_llm_for_diagnosis(&changes_revamp).await?;
    }
}

fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn Error>> {
    Command::new("git").arg("-C").arg(target).arg("commit").arg("-m").arg(message).status()?;

    print!("{}", PUSH_PROMPT);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("yes") {
        Command::new("git").arg("-C").arg(target).arg("push").status()?;
        println!("Changes committed and pushed.");
    } else {
        println!("Changes committed but not pushed.");
    }

    Ok(())
}

async fn send_to_llm_for_diagnosis(changes: &str) -> Result<String, anyhow::Error> {
    let ollama = Ollama::new(LOCALHOST.to_string(), LLM_PORT);

    print!("{}", CHANGES_NATURE_PROMPT);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let prompt = format!("{} for these changes:\n{} type:{}", SYSTEM_PROMPT, changes, input);
    let options = GenerationOptions::default()
        .temperature(TEMPERATURE)
        .repeat_penalty(REPEAT_PENALTY)
        .top_k(TOP_K)
        .top_p(TOP_P);

    let res = ollama
        .generate(GenerationRequest::new(MODEL.to_string(), prompt).options(options)).await
        .unwrap();

    Ok(res.response)
}
