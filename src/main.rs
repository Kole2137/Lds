use anyhow::{bail, Result};
use clap::{ArgAction, Parser};
use full_moon::tokenizer::{self, Symbol, TokenType};
use std::{env, fs, path::PathBuf};

const SEMICOLON: TokenType = TokenType::Symbol {
    symbol: (Symbol::Semicolon),
};

const LEFT_BRACE: TokenType = TokenType::Symbol {
    symbol: (Symbol::LeftBrace),
};
const RIGHT_BRACE: TokenType = TokenType::Symbol {
    symbol: (Symbol::RightBrace),
};

const LEFT_BRACKET: TokenType = TokenType::Symbol {
    symbol: (Symbol::LeftBracket),
};
const RIGHT_BRACKET: TokenType = TokenType::Symbol {
    symbol: (Symbol::RightBracket),
};

const FUNCTION: TokenType = TokenType::Symbol {
    symbol: (Symbol::Function),
};
const END: TokenType = TokenType::Symbol {
    symbol: (Symbol::End),
};

fn format(file_path: &PathBuf, array: &bool) -> Result<()> {
    let code = fs::read_to_string(file_path)?;
    let tokens = tokenizer::tokens(&code)?;

    let mut formatted = String::new();
    let mut code_lines: Vec<String> = code.lines().map(String::from).collect();
    let mut fn_depth = 0;
    let mut depth = 0;

    for token in tokens {
        if token.token_type() == &LEFT_BRACE || token.token_type() == &LEFT_BRACKET {
            depth += 1;
        } else if token.token_type() == &RIGHT_BRACE || token.token_type() == &RIGHT_BRACKET {
            depth -= 1;
        } else if token.token_type() == &FUNCTION && depth != 0 {
            fn_depth += 1;
        } else if token.token_type() == &END && depth != 0 {
            fn_depth -= 1;
        }

        if token.token_type() == &SEMICOLON {
            let line = token.start_position().line() - 1;

            if depth != 0 && *array {
                continue;
            }

            if code_lines[line].matches(';').count() == 1 {
                if depth == 0 || fn_depth != 0 {
                    code_lines[line] = code_lines[line].replace(';', "");
                } else {
                    code_lines[line] = code_lines[line].replace(';', ",");
                }
            } else {
                code_lines[line].pop();

                if depth != 0 && fn_depth == 0 {
                    code_lines[line].push(',');
                }
            }
        }
    }

    for line in code_lines {
        formatted.push_str(&line);
        formatted.push('\n');
    }

    if !code.ends_with('\n') {
        formatted.pop();
    }

    fs::write(file_path, formatted)?;

    println!("Formatted {} successfully!", file_path.display());

    Ok(())
}

fn format_dir(path: PathBuf, recursive: bool, array: &bool) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();

        if path.is_file() {
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if file_name.ends_with(".lua") || file_name.ends_with(".luau") {
                format(&path, array)?;
            }
        } else if recursive {
            format_dir(path, recursive, array)?;
        }
    }

    Ok(())
}

#[derive(Parser)]
struct Args {
    /// Path to the file or directory to format
    path: String,

    /// Format all files in the directory recursively
    #[arg(short, long, action = ArgAction::SetTrue)]
    recursive: bool,

    /// Keep semicolons at the end of array items
    #[arg(short, long, action = ArgAction::SetTrue)]
    array: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut path = PathBuf::from(&args.path);

    if !path.is_absolute() {
        path = env::current_dir()?.join(&path);
    }

    if !path.exists() {
        bail!("Path {} does not exist!", path.display())
    }

    if path.is_file() {
        format(&path, &args.array)?;
        return Ok(());
    }

    format_dir(path, args.recursive, &args.array)
}
