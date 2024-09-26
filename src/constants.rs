pub const SYSTEM_PROMPT: &str =
    "generate a commit message from our changes with a maximum length of 255 characters, DO NOT FORMAT YOUR COMMIT MESSAGE AND ONLY RESPOND WITH THE COMMIT MESSAGE REMOVE ```plaintext``` FROM YOUR RESPONSEuse Conventional Commits to describe changes and their impact, aligning with Semantic Versioning: fix: for bug fixes (PATCH), feat: for new features (MINOR), BREAKING CHANGE: for major API changes (MAJOR) indicated by ! or a BREAKING CHANGE: footer; other types like build:, chore:, docs:, style:, refactor:, perf:, test: provide context without affecting versioning unless marked as breaking; use feat!: or BREAKING CHANGE: to signal major changes; scopes (e.g., feat(api):) add context; commit types and scopes help maintainers and systems understand change impact";
pub const MODEL: &str = "deepseek-coder-v2";
pub const LOCALHOST: &str = "http://localhost";
pub const LLM_PORT: u16 = 11434;
pub const PROMPT_FOR_CHANGES: &str = "Look for (staged/unstaged) changes? ";
pub const DEFAULT_DIFF_TYPE: &str = "--cached";
pub const COMMIT_PROMPT: &str = "Do you like this commit message? (yes/no): ";
pub const PUSH_PROMPT: &str = "Do you want to push the commit? (yes/no): ";
pub const CHANGES_NATURE_PROMPT: &str =
    r#"
What is the nature of these changes? 
- (feat): For adding new features.
- (fix): For bug fixes only.
- (docs): For changes related to documentation, including `.md` file updates or code comments.
- (style): For changes that do not affect code functionality, such as formatting, spacing, or style adjustments.
- (chore): For changes related to build processes, dependencies, or maintenance tasks. 
Type: "#;
pub const TEMPERATURE: f32 = 0.1;
pub const REPEAT_PENALTY: f32 = 1.2;
pub const TOP_K: u32 = 35;
pub const TOP_P: f32 = 0.25;
