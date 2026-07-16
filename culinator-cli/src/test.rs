use super::*;
#[test]
fn cli_parses_check_subcommand() {
    let cli = Cli::try_parse_from(["culinator", "check", "recipe.cg"]).expect("parse");
    assert!(matches!(cli.command, Command::Check { .. }));
}
