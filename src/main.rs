use std::fs;
use std::env;
use std::io;
use std::io::Read;
use std::io::Write;

enum Token {
    Meow,
    Mrrp,
    Mrowp
}

#[derive(PartialEq)]#[derive(Debug)]
enum TranspileToken {
    Next,
    Previous,
    Increment,
    Decrement,
    Output,
    Input,
    LoopStart,
    LoopEnd,
    Other
}

fn transpile_token(token: &TranspileToken) -> (Token, Token) {
    match token {
        TranspileToken::Next => (Token::Meow, Token::Meow),
        TranspileToken::Previous => (Token::Meow, Token::Mrrp),
        TranspileToken::Increment => (Token::Meow, Token::Mrowp),
        TranspileToken::Decrement => (Token::Mrrp, Token::Meow),
        TranspileToken::Output => (Token::Mrrp, Token::Mrrp),
        TranspileToken::Input => (Token::Mrrp, Token::Mrowp),
        TranspileToken::LoopStart => (Token::Mrowp, Token::Meow),
        TranspileToken::LoopEnd => (Token::Mrowp, Token::Mrrp),
        TranspileToken::Other => (Token::Mrowp, Token::Mrowp),
    }
}

fn compile_token(token: &(Token, Token)) -> TranspileToken {
    match token {
        (Token::Meow, Token::Meow) => TranspileToken::Next,
        (Token::Meow, Token::Mrrp) => TranspileToken::Previous,
        (Token::Meow, Token::Mrowp) => TranspileToken::Increment,
        (Token::Mrrp, Token::Meow) => TranspileToken::Decrement,
        (Token::Mrrp, Token::Mrrp) => TranspileToken::Output,
        (Token::Mrrp, Token::Mrowp) => TranspileToken::Input,
        (Token::Mrowp, Token::Meow) => TranspileToken::LoopStart,
        (Token::Mrowp, Token::Mrrp) => TranspileToken::LoopEnd,
        (Token::Mrowp, Token::Mrowp) => TranspileToken::Other
    }
}

fn token_to_string(token: Token) -> String {
    match token {
        Token::Meow => String::from("Meow"),
        Token::Mrrp => String::from("Mrrp"),
        Token::Mrowp => String::from("Mrowp"),
    }
}

fn string_to_token(input: &str) -> Token {
    match input {
        "Meow" => Token::Meow,
        "Mrrp" => Token::Mrrp,
        "Mrowp" => Token::Mrowp,
        _ => panic!("Unknown token")
    }
}

fn transpiletokens_to_string(tokens: Vec<TranspileToken>) -> String {
    let tokens: Vec<String> = tokens
        .iter()
        .map(|x| transpile_token(x))
        .map(|(a, b)| token_to_string(a) + " " + &token_to_string(b))
        .collect();

    tokens.join(" ")
}

fn string_to_transpiletokens(input: String) -> Vec<TranspileToken> {
    input
        .chars()
        .map(|x| {
            match x {
                '>' => TranspileToken::Next,
                '<' => TranspileToken::Previous,
                '+' => TranspileToken::Increment,
                '-' => TranspileToken::Decrement,
                '.' => TranspileToken::Output,
                ',' => TranspileToken::Input,
                '[' => TranspileToken::LoopStart,
                ']' => TranspileToken::LoopEnd,
                _ => TranspileToken::Other
            }
        })
        .collect()
}

fn transpiletoken_to_char(token: TranspileToken) -> char {
    match token {
        TranspileToken::Next => '>',
        TranspileToken::Previous => '<',
        TranspileToken::Increment => '+',
        TranspileToken::Decrement => '-',
        TranspileToken::Output => '.',
        TranspileToken::Input => ',',
        TranspileToken::LoopStart => '[',
        TranspileToken::LoopEnd => ']',
        TranspileToken::Other => '\n',
    }
}

fn interpret_transpiletokens(tokens: Vec<TranspileToken>) {
    let mut data_pointer: u16 = 0;
    let mut data: [u8; 65536] = [0; 65536];

    let mut token_counter = 0;

    let mut loops: Vec<usize> = vec![];

    loop {
        let token = tokens.get(token_counter);
        if token.is_none() {
            break;
        }

        match token.unwrap() {
            TranspileToken::Next => data_pointer = data_pointer.wrapping_add(1),
            TranspileToken::Previous => data_pointer = data_pointer.wrapping_sub(1),
            TranspileToken::Increment => data[data_pointer as usize] = data[data_pointer as usize].wrapping_add(1),
            TranspileToken::Decrement => data[data_pointer as usize] = data[data_pointer as usize].wrapping_sub(1),
            TranspileToken::Output => {
                print!("{}", data[data_pointer as usize] as char);
                io::stdout().flush().ok();
            },
            TranspileToken::Input => {
                let mut buf = [0; 1];
                io::stdin().read_exact(&mut buf)
                    .expect("Invalid input");

                data[data_pointer as usize] = buf[0];
            },
            TranspileToken::LoopStart => {
                if data[data_pointer as usize] == 0 {
                    let mut count = 1;

                    while count != 0 && token_counter < tokens.len() {
                        token_counter += 1;

                        if tokens[token_counter as usize] == TranspileToken::LoopStart {
                            count += 1;
                        } else if tokens[token_counter as usize] == TranspileToken::LoopEnd {
                            count -= 1;
                        }
                    }
                } else {
                    loops.push(token_counter)
                }
            },
            TranspileToken::LoopEnd => {
                if data[data_pointer as usize] == 0 {
                    loops.pop();
                } else {
                    token_counter = *loops.last()
                        .expect("Unexpected exit of loop")
                }
            },
            TranspileToken::Other => (),
        }

        token_counter += 1
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 3, "Usage: cmeow <mode: compile | transpile | run> <file>");

    let mode = &args[1].to_ascii_lowercase();
    let filename = &args[2];

    let contents = fs::read_to_string(filename)
            .expect("Should have been able to read the file");

    if mode == "compile" {
        let tokens: Vec<&str> = contents
            .split_ascii_whitespace()
            .collect();

        let tokens: Vec<(Token, Token)> = tokens
            .chunks_exact(2)
            .map(|x| (string_to_token(*x.first().unwrap()), string_to_token(*x.last().unwrap())))
            .collect();

        let compiled_string: String = tokens
            .iter()
            .map(|x| transpiletoken_to_char(compile_token(x)))
            .collect();

        println!("{}", compiled_string)
    } else if mode == "run" {
        let tokens: Vec<&str> = contents
            .split_ascii_whitespace()
            .collect();

        let tokens: Vec<TranspileToken> = tokens
            .chunks_exact(2)
            .map(|x| compile_token(&(string_to_token(*x.first().unwrap()), string_to_token(*x.last().unwrap()))))
            .collect();

        interpret_transpiletokens(tokens)
    } else if mode == "transpile" {
        let tokens = string_to_transpiletokens(contents);

        println!("{}", transpiletokens_to_string(tokens));
    } else {
        panic!("Invalid mode!")
    }
}
