use super::*;
#[test]
fn cli_parses_check_subcommand() {
    let cli = Cli::try_parse_from(["culinator", "check", "recipe.cg"]).expect("parse");
    assert!(matches!(cli.command, Command::Check { .. }));
}

#[test]
fn cli_parses_nested_recipe_command() {
    let cli = Cli::try_parse_from([
        "culinator",
        "--database",
        "catalog.sqlite3",
        "--format",
        "json",
        "recipe",
        "get",
        "toast",
    ])
    .expect("parse");
    assert_eq!(cli.database, PathBuf::from("catalog.sqlite3"));
    assert!(matches!(
        cli.command,
        Command::Recipe {
            command: RecipeCommand::Get { .. }
        }
    ));
}

#[test]
fn destructive_commands_require_explicit_confirmation_flag() {
    let cli = Cli::try_parse_from(["culinator", "recipe", "delete", "toast"]).expect("parse");
    assert!(matches!(
        cli.command,
        Command::Recipe {
            command: RecipeCommand::Delete { yes: false, .. }
        }
    ));
}

#[test]
fn cli_exposes_scan_import_and_automation_output() {
    let cli = Cli::try_parse_from([
        "culinator",
        "--format",
        "json",
        "import",
        "scan",
        "front.jpg",
        "back.jpg",
        "--target-language",
        "en",
    ])
    .expect("parse");
    assert!(matches!(
        cli.command,
        Command::Import {
            command: ImportCommand::Scan { images, .. }
        } if images.len() == 2
    ));
}
