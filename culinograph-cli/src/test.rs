use super::*;
#[test]
fn cli_parses_check_subcommand() {
    let cli = Cli::try_parse_from(["culinograph", "check", "recipe.cg"]).expect("parse");
    assert!(matches!(cli.command, Command::Check { .. }));
}
