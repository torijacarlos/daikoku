mod alias;
mod error;
mod models;
mod settings;
mod storage;

use crate::{
    models::{get_account_balance, get_accounts_net_worth, get_wallet_liquidity_index, Wallet},
    settings::Settings,
    storage::{import, left_pad},
};
use std::{char, io};

enum State {
    Start,
    WalletSelect,
    Pin(usize),
    Wallet(Wallet),
}

fn main() {
    let mut state = State::Start;
    let mut buffer;
    let mut error = String::new();
    let available_wallets = storage::get_all_wallets_locations();
    let settings = Settings::load().unwrap();
    loop {
        buffer = String::new();
        print_cmd_available(error.clone());
        match state {
            State::Start => match io::stdin().read_line(&mut buffer) {
                Ok(n) => {
                    println!("{n}: {buffer}");
                    match buffer.trim() {
                        "w" | "wallet" => {
                            state = State::WalletSelect;
                            error = String::new();
                        }
                        "e" | "exit" => break,
                        input => error = format!("Invalid input: {}", input),
                    }
                }
                Err(_) => {
                    println!("Something crashed and burned while reading stdin");
                }
            },
            State::WalletSelect => {
                println!("Select a wallet to work with");
                println!();
                for w_index in 0..available_wallets.len() {
                    println!("    {}) {:?}", w_index, available_wallets[w_index])
                }
                println!();
                match io::stdin().read_line(&mut buffer) {
                    Ok(n) => {
                        println!("{n}: {buffer}");
                        match buffer.trim() {
                            "e" | "exit" => {
                                state = State::Start;
                                error = String::new();
                            }
                            input => match u8::from_str_radix(input, 10) {
                                Ok(n) => {
                                    if (n as usize) < available_wallets.len() {
                                        state = State::Pin(n as usize);
                                        error = String::new();
                                    } else {
                                        error = format!("Invalid wallet: {}", n);
                                    }
                                }
                                Err(_) => {
                                    error = format!("Invalid input: {}", input);
                                }
                            },
                        }
                    }
                    Err(_) => {
                        println!("Something crashed and burned while reading stdin");
                    }
                }
            }
            State::Pin(wallet_index) => {
                println!("Enter wallet pin");
                println!();
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        match import(
                            available_wallets[wallet_index].clone(),
                            &buffer.trim().to_string(),
                            &settings.crypt_key,
                        ) {
                            Ok(w) => {
                                state = State::Wallet(w);
                                error = String::new();
                            }
                            Err(e) => {
                                error = format!("Invalid pin {:?}", e);
                                state = State::WalletSelect;
                            }
                        }
                    }
                    Err(_) => {
                        println!("Something crashed and burned while reading stdin");
                    }
                }
            }
            State::Wallet(ref wallet) => {
                println!("Id: {:#?} | Alias: {}", wallet.id, wallet.alias);
                println!(
                    "Liq. Index: {} | Net Worth {}",
                    get_wallet_liquidity_index(&wallet.accounts),
                    format_money(get_accounts_net_worth(&wallet.accounts))
                );
                println!(
                    "Created: {:?} | Updated {}",
                    wallet.created_date, wallet.updated_date
                );

                let mut field_width: [usize; 6] = [0; 6];
                let mut total_width: usize = 0;
                for acc in &wallet.accounts {
                    field_width[0] = if field_width[0] > acc.id.unwrap().to_string().len() {
                        field_width[0]
                    } else {
                        acc.id.unwrap().to_string().len()
                    };
                    field_width[1] = if field_width[1] > acc.name.len() {
                        field_width[1]
                    } else {
                        acc.name.len()
                    };
                    field_width[2] = if field_width[2] > acc.acc_type.as_str().to_string().len() {
                        field_width[2]
                    } else {
                        acc.acc_type.as_str().to_string().len()
                    };
                    field_width[3] =
                        if field_width[3] > format_money(get_account_balance(acc)).len() {
                            field_width[3]
                        } else {
                            format_money(get_account_balance(acc)).len()
                        };
                    field_width[4] =
                        if field_width[4] > acc.created_date.date_naive().to_string().len() {
                            field_width[4]
                        } else {
                            acc.created_date.date_naive().to_string().len()
                        };
                    field_width[5] =
                        if field_width[5] > acc.updated_date.date_naive().to_string().len() {
                            field_width[5]
                        } else {
                            acc.updated_date.date_naive().to_string().len()
                        };
                }
                for w in field_width {
                    total_width += w;
                }
                total_width += (5 * 3) + 4;

                render_line(total_width);
                println!("Accounts:");
                render_line(total_width);
                println!(
                    "| {} | {} | {} | {} | {} | {} |",
                    left_pad(&"Id".to_string(), field_width[0]),
                    left_pad(&"Name".to_string(), field_width[1]),
                    left_pad(&"Acc Type".to_string(), field_width[2]),
                    left_pad(&"Balance".to_string(), field_width[3]),
                    left_pad(&"Created".to_string(), field_width[4]),
                    left_pad(&"Updated".to_string(), field_width[5])
                );
                render_line(total_width);
                for acc in &wallet.accounts {
                    println!(
                        "| {} | {} | {} | {} | {} | {} |",
                        left_pad(&acc.id.unwrap().to_string(), field_width[0]),
                        left_pad(&acc.name, field_width[1]),
                        left_pad(&acc.acc_type.as_str().to_string(), field_width[2]),
                        left_pad(&format_money(get_account_balance(acc)), field_width[3]),
                        left_pad(&acc.created_date.date_naive().to_string(), field_width[4]),
                        left_pad(&acc.updated_date.date_naive().to_string(), field_width[5])
                    );
                }
                render_line(total_width);
                println!();
                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }
}

fn format_money(amount: f32) -> String {
    let amnt_str = amount.to_string();
    let money_and_cents: Vec<&str> = amnt_str.split('.').collect();

    let mut whole: Vec<&str> = money_and_cents[0].split("").collect();
    whole.rotate_left(1);
    whole.pop();
    whole.pop();

    let mut whole_str = String::new();
    for w in whole.rchunks(3) {
        whole_str = ",".to_string() + &w.join("") + &whole_str;
    }
    if whole_str.starts_with(',') {
        let mut whole: Vec<&str> = whole_str.split("").collect();
        whole.rotate_left(2);
        whole.pop();
        whole.pop();
        whole.pop();
        whole_str = whole.join("");
    }

    whole_str
        + "."
        + if money_and_cents.len() == 2 {
            money_and_cents[1]
        } else {
            "00"
        }
}

fn render_line(width: usize) {
    println!("{}", (0..width).map(|_| "-").collect::<String>());
}

fn print_cmd_available(error: String) {
    print!("{}[2J", 27 as char);
    if !error.is_empty() {
        println!("There was an error");
        println!("\x1B[31m ");
        println!("  {}", error);
        println!("\x1B[0m ");
    }
    println!("=======================================");
    println!();
    println!("Available commands");
    println!();
    println!("    w(allets): Show available wallets ");
    println!("    e(xit): Exits the current operation");
    println!();
    println!("=======================================");
    println!();
}
