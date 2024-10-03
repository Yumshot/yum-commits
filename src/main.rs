use ollama_rs::{
    generation::completion::request::GenerationRequest, // Module for generating completion requests using Ollama
    generation::options::GenerationOptions, // Module for configuring options for generation requests
    Ollama, // Struct representing the Ollama language model
};
use std::error::Error; // Trait for defining custom errors
use std::path::Path; // Type representing a path to a file or directory
use std::process::Command; // Module for executing external commands
mod constants;
use constants::*;
use requestty::Question; // Import Requestty for interactive command-line questions

// Function to prompt the user for the target Git repository directory
fn prompt_target_directory() -> String {
    // Prompt the user to input the target directory
    let question = Question::input("target")
        .message("Please enter the target directory containing the Git repository:")
        .build();
    
    // Retrieve the input from the user
    let answer = requestty::prompt_one(question).unwrap();
    answer.as_string().unwrap().to_string() // Convert the input to a String
}

// Main function with async/await handling
#[tokio::main(flavor = "current_thread")] // Macro to define an asynchronous main function that runs on a single thread
async fn main() -> Result<(), Box<dyn Error>> {
    // Prompt the user for the target directory using Requestty
    let target_dir = prompt_target_directory();
    
    // Convert the parsed target directory string into a Path object for easier file operations
    let target = Path::new(&target_dir);

    // Check if the specified target directory is indeed a Git repository
    if !is_git_repo(target).await? {
        eprintln!("Target directory is not a git repository."); // Print error message if it's not a Git repo
        return Ok(()); // Exit the program successfully since the directory is invalid
    }

    // Check if there are any changes in the target Git repository
    if !has_changes(target).await? {
        println!("No changes to commit."); // Print message if there are no changes
        return Ok(());
    }

    // Prompt the user for the type of changes (staged or unstaged)
    let changes_type = prompt_changes_type(target).await?;

    // Generate a commit message based on the identified changes
    let commit_message = generate_commit_message(&changes_type).await?;

    // Commit the identified changes to the Git repository with the generated commit message
    commit_changes(target, &commit_message)?;

    Ok(()) // Return success if all operations are completed successfully
}

// Function to check if a directory is a Git repository
async fn is_git_repo(target: &Path) -> Result<bool, std::io::Error> {
    // Execute the 'git rev-parse --is-inside-work-tree' command in the target directory
    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .and_then(|output| {
            if output.status.success() {
                Ok(true)
            } else {
                Ok(false)
            }
        })
}

// Function to check if there are changes in the Git repository
async fn has_changes(target: &Path) -> Result<bool, std::io::Error> {
    // Execute the 'git status --porcelain' command in the target directory
    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("status")
        .arg("--porcelain")
        .output()?;

    Ok(!output.stdout.is_empty()) // Return true if there are any changes
}

// Function to prompt the user for changes type (staged or unstaged)
async fn prompt_changes_type(target: &Path) -> Result<String, std::io::Error> {
    // Use Requestty to ask the user for staged or unstaged changes
    let question = Question::select("changes_type")
        .message("What type of changes would you like to commit?")
        .choices(vec!["staged".to_string(), "unstaged".to_string()])
        .build();

    let answer = requestty::prompt_one(question).unwrap();
    let changes_type = answer.as_list_item().unwrap().text.clone();

    let diff_type = match changes_type.as_str() {
        "staged" => "--cached",
        "unstaged" => "",
        _ => DEFAULT_DIFF_TYPE,
    };

    let output = Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("diff")
        .arg(diff_type)
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string()) // Return the diff output
}

// Function to generate a commit message using LLM (Large Language Model)
async fn generate_commit_message(changes: &str) -> Result<String, Box<dyn Error>> {
    let mut commit_message = send_to_llm_for_diagnosis(changes).await?;

    // Keep generating commit messages until the user approves
    loop {
        println!("\nGenerated Commit Message:\n{}", commit_message);

        // Use Requestty to ask if the user approves the message
        let question = Question::confirm("approve")
            .message("Do you approve this commit message?")
            .default(true)
            .build();
        
        let answer = requestty::prompt_one(question).unwrap();
        if answer.as_bool().unwrap() {
            return Ok(commit_message); // If approved, return the commit message
        }

        println!("Generating a new commit message...");
        // If not approved, generate a new one with additional context
        let changes_revamp = format!("{} {}{}", FOLLOW_UP, SYSTEM_PROMPT, commit_message);
        commit_message = send_to_llm_for_diagnosis(&changes_revamp).await?;
    }
}

// Function to commit changes to Git repository
fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn Error>> {
    Command::new("git")
        .arg("-C")
        .arg(target)
        .arg("commit")
        .arg("-m")
        .arg(message)
        .status()?;

    // Use Requestty to ask if the user wants to push the changes
    let question = Question::confirm("push")
        .message("Do you want to push the changes?")
        .default(true)
        .build();

    let answer = requestty::prompt_one(question).unwrap();
    if answer.as_bool().unwrap() {
        Command::new("git")
            .arg("-C")
            .arg(target)
            .arg("push")
            .status()?;
        println!("Changes committed and pushed.");
    } else {
        println!("Changes committed but not pushed.");
    }

    Ok(())
}

// Asynchronous function to send changes to an LLM for diagnosis and generate a commit message
async fn send_to_llm_for_diagnosis(changes: &str) -> Result<String, anyhow::Error> {
    let ollama = Ollama::new(LOCALHOST.to_string(), LLM_PORT); // Create a new instance of the Ollama client

    // Use Requestty to prompt the user to specify the nature of changes (bugfix, feature, etc.)
    let question = Question::input("changes_nature")
        .message("What is the nature of the changes (e.g., bugfix, feature)?")
        .build();

    let answer = requestty::prompt_one(question).unwrap();
    let input = answer.as_string().unwrap();

    // Construct the full prompt for the LLM including the nature of changes and the original changes
    let prompt = format!("{} for these changes:\n{} type:{}", SYSTEM_PROMPT, changes, input);

    // Set up generation options with specified parameters (temperature, repeat penalty, etc.)
    let options = GenerationOptions::default()
        .temperature(TEMPERATURE)
        .repeat_penalty(REPEAT_PENALTY)
        .top_k(TOP_K)
        .top_p(TOP_P);

    // Send the prompt to the LLM for diagnosis and get a response
    let res = ollama.generate(GenerationRequest::new(MODEL.to_string(), prompt).options(options)).await.unwrap();

    Ok(res.response) // Return the generated commit message from the LLM response
}

