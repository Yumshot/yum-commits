use ollama_rs::{
    generation::completion::request::GenerationRequest,
    generation::options::GenerationOptions,
    Ollama,
};
use std::error::Error;
use std::path::Path;
use std::process::{ Command, Output };
mod constants;
use constants::*;
use requestty::Question;

mod git_operations {
    use super::*;

    pub fn run_git_command(args: &[&str], target: &Path) -> Result<Output, std::io::Error> {
        Command::new("git").arg("-C").arg(target).args(args).output()
    }

    pub async fn is_git_repo(target: &Path) -> Result<bool, std::io::Error> {
        let output = run_git_command(&["rev-parse", "--is-inside-work-tree"], target)?;
        Ok(output.status.success())
    }

    pub async fn has_changes(target: &Path) -> Result<bool, std::io::Error> {
        let output = run_git_command(&["status", "--porcelain"], target)?;
        Ok(!output.stdout.is_empty())
    }

    pub async fn get_changes(target: &Path, changes_type: &str) -> Result<String, std::io::Error> {
        let diff_type = match changes_type {
            "staged" => "--cached",
            "unstaged" => "",
            _ => DEFAULT_DIFF_TYPE,
        };

        let output = run_git_command(&["diff", diff_type, ":(exclude)Cargo.lock"], target)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn commit_changes(target: &Path, message: &str) -> Result<(), Box<dyn Error>> {
        run_git_command(&["commit", "-m", message], target)?;
        Ok(())
    }

    pub fn push_changes(target: &Path) -> Result<(), Box<dyn Error>> {
        run_git_command(&["push"], target)?;
        Ok(())
    }
}

mod llm_operations {
    use super::*;

    pub async fn generate_commit_message_from_llm(
        changes: &str,
        nature_of_changes: &str
    ) -> Result<String, anyhow::Error> {
        let ollama = Ollama::new(LOCALHOST.to_string(), LLM_PORT);

        let prompt = format!(
            "{} \nCHANGES: {} \nNATURE OF CHANGES: {}",
            SYSTEM_PROMPT,
            changes,
            nature_of_changes
        );
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
}

mod user_interaction {
    use super::*;

    pub fn prompt_target_directory() -> String {
        let question = Question::input("target").message(TARGET_DIRECTORY).build();
        let answer = requestty::prompt_one(question).unwrap();
        answer.as_string().unwrap().to_string()
    }

    pub fn prompt_changes_type() -> String {
        let question = Question::select("changes_type")
            .message(CHANGES_TYPE_INQUERY)
            .choices(vec!["staged".to_string(), "unstaged".to_string()])
            .build();
        let answer = requestty::prompt_one(question).unwrap();
        answer.as_list_item().unwrap().text.clone()
    }

    pub fn prompt_nature_of_changes() -> String {
        let question = Question::input("changes_nature").message(CHANGES_NATURE_INQUERY).build();
        let answer = requestty::prompt_one(question).unwrap();
        answer.as_string().unwrap().to_string()
    }

    pub fn confirm_commit_message(commit_message: &str) -> bool {
        println!("\nGenerated Commit Message:\n{}", commit_message);
        let question = Question::confirm("approve")
            .message(APPROVE_COMMIT_INQUERY)
            .default(true)
            .build();
        requestty::prompt_one(question).unwrap().as_bool().unwrap()
    }

    pub fn confirm_push_changes() -> bool {
        let question = Question::confirm("push")
            .message(PUSH_CHANGES_INQUERY)
            .default(true)
            .build();
        requestty::prompt_one(question).unwrap().as_bool().unwrap()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = user_interaction::prompt_target_directory();
    let target = Path::new(&target_dir);

    if !git_operations::is_git_repo(target).await? {
        eprintln!("{}", DIRECTORY_NOT_GIT_REPO);
        return Ok(());
    }

    if !git_operations::has_changes(target).await? {
        println!("{}", NO_COMMIT_CHANGES);
        return Ok(());
    }

    let changes_type = user_interaction::prompt_changes_type();
    let changes = git_operations::get_changes(target, &changes_type).await?;

    let nature_of_changes = user_interaction::prompt_nature_of_changes();
    let mut commit_message = llm_operations::generate_commit_message_from_llm(
        &changes,
        &nature_of_changes
    ).await?;

    while !user_interaction::confirm_commit_message(&commit_message) {
        commit_message = llm_operations::generate_commit_message_from_llm(
            &changes,
            &nature_of_changes
        ).await?;
    }

    git_operations::commit_changes(target, &commit_message)?;

    if user_interaction::confirm_push_changes() {
        git_operations::push_changes(target)?;
        println!("{}", CHANGES_PUSHED);
    } else {
        println!("{}", CHANGES_NOT_PUSHED);
    }

    Ok(())
}
