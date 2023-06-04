pub(crate) mod system;

pub struct Command {
    name: String,
    args: Vec<String>,
}

impl From<&str> for Command {
    fn from(line: &str) -> Command {
        if let Some((cmd, args)) = line.split_once(' ') {
            Command {
                name: cmd.to_string(),
                args: args.split(' ').map(|s| s.to_string()).collect::<Vec<_>>(),
            }
        } else {
            Command {
                name: line.to_string(),
                args: Vec::new(),
            }
        }
    }
}
