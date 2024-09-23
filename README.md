<p align="center">
  <a href="" rel="noopener">
 <img width=200px height=200px style="border-radius: 100px;" src="https://cdn.discordapp.com/attachments/1287871659625283596/1287871677346218165/Yum_Commits_Logo.jpg?ex=66f31f77&is=66f1cdf7&hm=e66f09c96f64efcd1c9f72b65063cbee0b20a960871536263e3cc7c073a77317&" alt="Project logo"></a>
</p>

<h3 align="center">Yum Commits</h3>

<div align="center">

[![Status](https://img.shields.io/badge/status-active-success.svg)]()
[![GitHub Issues](https://img.shields.io/github/issues/Yumshot/yum-commits.svg)](https://github.com/Yumshot/yum-commits/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/Yumshot/yum-commits.svg)](https://github.com/Yumshot/yum-commits/pulls)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](/LICENSE)

</div>

---

<p align="center"> Use Local LLM (Ollama) to simplify writing your git commit messages.
    <br>
</p>

## 📝 Table of Contents

- [About](#about)
- [Getting Started](#getting_started)
- [Deployment](#deployment)
- [Usage](#usage)
- [Built Using](#built_using)
- [TODO](../TODO.md)
- [Contributing](../CONTRIBUTING.md)
- [Authors](#authors)
- [Acknowledgments](#acknowledgement)

## 🧐 About <a name = "about"></a>
The "Yum-Commits" project is a CLI tool designed to streamline the process of creating commit messages for Git repositories by leveraging AI technology. Aimed at developers, this tool automates the generation of concise and informative commit messages that adhere to standard Git commit message conventions. The tool integrates with Git repositories to analyze changes, detect uncommitted modifications, and generate commit messages that describe the updates in a consistent format, such as `feat`, `fix`, `docs`, and other common types, ensuring clarity and consistency in version control documentation.

By utilizing the Ollama AI model, Yum-Commits generates commit messages based on the context of the changes detected within the Git repository. It prompts the user with a proposed commit message and offers the flexibility to iterate until the message meets the user's expectations. This approach not only saves time but also enhances the quality of commit messages, making it an invaluable tool for developers aiming to maintain a well-documented codebase.

## 🏁 Getting Started <a name = "getting_started"></a>

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See [deployment](#deployment) for notes on how to deploy the project on a live system.

### Prerequisites
To use Yum-Commits, you'll need to have Rust installed on your machine, along with several Rust libraries, and Ollama running locally. Below is a list of required software and dependencies, along with instructions on how to set them up.

#### Software Requirements:
1. **Rust**: Install Rust by following the instructions on the [Rust official website](https://www.rust-lang.org/tools/install).
2. **Ollama**: Install and run Ollama locally. You can download and set up Ollama by following the guide on [Ollama's website](https://ollama.com/).

### Ollama
```bash
ollama pull deepseek-coder-v2 (or whatever model you would like to use)
```

#### Rust Dependencies:
Add the following dependencies to your `Cargo.toml` file to ensure your project has all the necessary libraries:

```toml
[dependencies]
anyhow = "1.0.89"          # Error handling library
clap = { version = "4.5.18", features = ["derive"] }  # Command line argument parsing
ollama-rs = "0.2.1"        # Client library to interact with the Ollama API
tokio = { version = "1.40.0", features = ["full"] }  # Asynchronous runtime for Rust
```

#### Installation Steps:
1. **Clone the repository** and navigate to the project directory:
   ```bash
   git clone <repository_url>
   cd yum-commits
   ```

2. **Install the required dependencies** using Cargo:
   ```bash
   cargo build
   ```

3. **Run Ollama** locally, ensuring it is configured to listen on the correct endpoint (`http://localhost:11434`) as required by the application.

These steps will set up your environment to work with Yum-Commits, allowing you to automate your commit message generation with AI-powered insights.


## 🎈 Usage <a name="usage"></a>


## 🚀 Deployment <a name = "deployment"></a>

Add additional notes about how to deploy this on a live system.

## ⛏️ Built Using <a name = "built_using"></a>

- [Rust](https://www.rust-lang.org/tools/install) - Language
- [Ollama](https://ollama.com/download) - LLM Runner

## ✍️ Authors <a name = "authors"></a>

- [@Yumshot](https://github.com/Yumshot) - Idea & Initial work
