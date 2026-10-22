use crate::security::policy::CommandClass;

/// Host-owned interpretation of TinyBox's structural shell facts.
///
/// TinyBox reports live command segments and redirection only. OpenHuman adds
/// the policy fields below locally so command permission and autonomy remain
/// host decisions instead of bus-contract vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CommandAnalysis {
    pub segments: Vec<CommandSegment>,
    pub hidden_execution: bool,
    pub redirection: bool,
    pub expansion: bool,
    pub tee: bool,
    pub background: bool,
    pub literal_words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CommandSegment {
    pub source: String,
    pub command: String,
    pub basename: String,
    pub normalized_name: String,
    pub arguments: Vec<String>,
    pub class: CommandClass,
    pub leading_env_assignment: bool,
    pub dangerous_env_prefix: bool,
    pub executor: bool,
}

impl CommandAnalysis {
    pub(crate) fn from_shell_analysis(facts: tinybox_bus::ShellAnalysis) -> Self {
        let mut analysis = Self {
            segments: Vec::new(),
            hidden_execution: false,
            redirection: facts.redirection,
            expansion: false,
            tee: false,
            background: false,
            literal_words: Vec::new(),
        };

        for source in facts.segments {
            let words = source.split_whitespace().collect::<Vec<_>>();
            let Some(raw_command) = words.first().copied() else {
                continue;
            };
            let command = raw_command.trim_matches(|c: char| matches!(c, '\'' | '"'));
            let basename = command.rsplit(['/', '\\']).next().unwrap_or(command);
            let normalized_name = basename.to_ascii_lowercase();
            let arguments = words
                .iter()
                .skip(1)
                .map(|word| {
                    word.trim_matches(|c: char| matches!(c, '\'' | '"'))
                        .to_owned()
                })
                .collect::<Vec<_>>();
            let lower = source.to_ascii_lowercase();
            let leading_env_assignment = command.contains('=')
                && command.split_once('=').is_some_and(|(name, _)| {
                    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                });
            let dangerous_env_prefix =
                words
                    .iter()
                    .take_while(|word| word.contains('='))
                    .any(|word| {
                        let name = word
                            .split('=')
                            .next()
                            .unwrap_or_default()
                            .to_ascii_uppercase();
                        matches!(
                            name.as_str(),
                            "LD_PRELOAD"
                                | "LD_LIBRARY_PATH"
                                | "DYLD_INSERT_LIBRARIES"
                                | "DYLD_LIBRARY_PATH"
                                | "PATH"
                        )
                    });
            let executor = matches!(
                normalized_name.as_str(),
                "sh" | "bash"
                    | "zsh"
                    | "fish"
                    | "python"
                    | "python2"
                    | "python3"
                    | "node"
                    | "ruby"
                    | "perl"
                    | "php"
                    | "lua"
                    | "awk"
                    | "xargs"
            );
            let class = classify_host_command(&normalized_name, &arguments, &lower);
            analysis.hidden_execution |= lower.contains("$(")
                || lower.contains("<(")
                || lower.contains(">(")
                || source.contains('`')
                || has_background_operator(&source);
            analysis.expansion |= source.contains('$') || source.contains('`');
            analysis.background |= has_background_operator(&source);
            analysis.tee |= words
                .iter()
                .any(|word| word == &"tee" || word.ends_with("/tee"));
            analysis.literal_words.extend(
                words
                    .iter()
                    .map(|word| {
                        word.trim_matches(|c: char| matches!(c, '\'' | '"' | '<' | '>' | '&' | '|'))
                            .to_owned()
                    })
                    .filter(|word| !word.is_empty()),
            );
            analysis.segments.push(CommandSegment {
                source: source.clone(),
                command: source.clone(),
                basename: basename.to_owned(),
                normalized_name,
                arguments,
                class,
                leading_env_assignment,
                dangerous_env_prefix,
                executor,
            });
        }
        analysis
    }
}

fn has_background_operator(source: &str) -> bool {
    let bytes = source.as_bytes();
    bytes.iter().enumerate().any(|(index, byte)| {
        *byte == b'&'
            && bytes.get(index + 1) != Some(&b'&')
            && (index == 0 || bytes[index - 1] != b'&')
            && (index == 0 || bytes[index - 1] != b'>')
    })
}

pub(super) fn classify_host_command(base: &str, args: &[String], source: &str) -> CommandClass {
    if matches!(
        base,
        "sudo"
            | "su"
            | "mkfs"
            | "dd"
            | "shutdown"
            | "reboot"
            | "halt"
            | "poweroff"
            | "useradd"
            | "userdel"
            | "usermod"
            | "passwd"
    ) || source.contains("rm -rf /")
        || source.contains("rm -fr /")
        || source.contains(":(){:|:&};:")
    {
        return CommandClass::Destructive;
    }
    if matches!(
        base,
        "curl" | "wget" | "ssh" | "scp" | "sftp" | "ftp" | "telnet" | "nc" | "ncat" | "netcat"
    ) {
        return CommandClass::Network;
    }
    let first = args.first().map(String::as_str).unwrap_or_default();
    if matches!(base, "npm" | "pnpm" | "yarn" | "pip" | "pip3")
        && matches!(first, "install" | "add" | "remove" | "uninstall" | "update")
        || base == "cargo" && matches!(first, "add" | "remove" | "install" | "update")
    {
        return CommandClass::Install;
    }
    if matches!(
        base,
        "ls" | "pwd"
            | "cat"
            | "head"
            | "tail"
            | "grep"
            | "rg"
            | "find"
            | "stat"
            | "wc"
            | "sort"
            | "echo"
            | "printf"
            | "whoami"
            | "date"
    ) || base == "git" && matches!(first, "status" | "diff" | "log" | "show" | "branch")
        || base == "cargo" && matches!(first, "check" | "test" | "tree" | "metadata")
    {
        return CommandClass::Read;
    }
    CommandClass::Write
}
