pub const SYSTEM_PROMPT: &str =
    "Generate a commit message up to 255 characters, adhering to the Conventional Commits specification and Semantic Versioning guidelines. Respond ONLY with the commit message, without any formatting or surrounding text. Use the following types to describe changes: fix: for bug fixes (PATCH), feat: for new features (MINOR), BREAKING CHANGE: or feat!: for major changes (MAJOR). Other types like build:, chore:, docs:, style:, refactor:, perf:, and test: provide context without affecting versioning unless marked as breaking. Scopes (e.g., feat(api):) are optional but add useful context for maintainers.";
pub const MODEL: &str = "llama3.2:3b";
pub const LOCALHOST: &str = "http://localhost";
pub const LLM_PORT: u16 = 11434;
pub const DEFAULT_DIFF_TYPE: &str = "--cached";
pub const TEMPERATURE: f32 = 0.2;
pub const REPEAT_PENALTY: f32 = 1.3;
pub const TOP_K: u32 = 35;
pub const TOP_P: f32 = 0.25;
pub const TARGET_DIRECTORY: &str =
    "Please enter the target directory containing the Git repository:";
pub const CHANGES_TYPE_INQUERY: &str = "What type of changes would you like to commit?";
pub const CHANGES_NATURE_INQUERY: &str = "What is the nature of the changes? bugfix, feature, chore, docs, style, refactor, perf, test, build, or feat!";
pub const PUSH_CHANGES_INQUERY: &str = "Do you want to push the changes?";
pub const APPROVE_COMMIT_INQUERY: &str = "Do you approve this commit message?";
