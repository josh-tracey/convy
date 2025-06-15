# Convy

Convy is a tool to enforce a consistent commit message format across a project. It is designed to be used as a commit-msg hook in a Git repository.

## Installation

To install Convy, run the following command:
```bash
 cargo install --git https://github.com/josh-tracey/convy
```

if you have issues with the above method try

```
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo install --git https://github.com/josh-tracey/convy
```

## Usage

To get started with Convy and enforce commit message validation in your Git repository, run the `init` command:

```bash
convy init
```

This command performs the following actions:
1. Creates a default `.convy.toml` configuration file in the root of your repository if one doesn't exist.
2. Creates (or overwrites) the `.git/hooks/commit-msg` script with the necessary logic to validate commit messages using `convy parse`.
3. Makes the `.git/hooks/commit-msg` script executable.

After running `convy init`, Convy will automatically validate your commit messages each time you make a commit.

**Important Note on `convy init` Behavior:**
Previously, the `init` command also automatically added `.convy.toml` and the hook script to Git, committed them, and pushed them. This is no longer the case. After running `convy init`, you will now be guided to manually perform the following Git operations:
- `git add .convy.toml .git/hooks/commit-msg`
- `git commit -m "feat: initialize convy for commit message validation"` (or a similar message)
- `git push`

This change gives you more control over your commit history.

The `commit-msg` hook script installed by `convy init` contains the following logic:
```bash
#!/bin/bash

commit_msg_file=$1

# Read the commit message from the file
commit_msg=$(cat "$commit_msg_file")

# Run convy parse and capture the entire output (including errors)
convy_result=$(convy parse "$commit_msg" 2>&1)  # Redirect stderr to stdout

# Check if Convy's output contains any error message
if echo "$convy_result" | grep -q "Error:"; then
    echo -e "\033[31mError:\033[0m Commit message does not match the required format:"
    echo "$convy_result"
    exit 1  # Reject the commit 
else
    echo -e "\033[32mCommit message format is valid.\033[0m"
    exit 0  # Allow the commit
fi
```

## Configuration

Convy allows you to define custom commit types in a configuration file. The configuration file should be named `.convy.toml` and placed in the root of your Git repository.

Here is an example of a `.convy.toml` file that defines custom commit types and disables the check for breaking changes footer:

```toml
additional_types = [
    "revert",
    "wip"
]
require_breaking_change_footer = false
```

### Generating Changelogs

As this this projects doesn't currently focus on generating changelogs, it is recommended to use a tool like [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) to generate changelogs.

A good tool is change which can be used without installation by running the following command:
```bash
 curl -s "https://raw.githubusercontent.com/adamtabrams/change/master/change" | sh -s -- init
```

## Contributing

If you have any suggestions, bug reports, or feature requests, please open an issue on GitHub.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
