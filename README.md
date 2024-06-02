# Convy

Convy is a tool to enforce a consistent commit message format across a project. It is designed to be used as a commit-msg hook in a Git repository.

## Installation

To install Convy, run the following command:
```bash
curl -sSL https://raw.githubusercontent.com/josh-tracey/convy/main/install.sh | bash
```

## Usage

To enforce a commit message format using Convy, you need to add a commit-msg hook to your Git repository. The hook should run Convy to validate the commit message format.

Here is an example of a commit-msg hook that uses Convy to validate the commit message format:

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

Save the above script as `.git/hooks/commit-msg` in your Git repository and make it executable.

Now, every time you commit a change, Convy will validate the commit message format according to the rules you have defined.

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

