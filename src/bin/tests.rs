use std::io;
use std::io::Write;

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32
}

impl Rectangle {
    fn _area(&self) -> u32 {
        self.width*self.height
    }

    fn _can_hold(&self, r: &Rectangle) -> bool {
        r.width <= self.width && r.height <= self.height
    }
}

fn _add_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i+1)
    }
}

fn _get_dangling_pointer() -> String {
    let s = String::from("hello");
    return s;
}

pub fn _fibonacci(n: i32) -> i32 {
    if n < 0 {
        return 0;
    }

    if n <= 1 {
        return n;
    }

    let mut p = 0;
    let mut f = 1;
    for _ in 2..(n+1) {
        let fp = f;
        f = f+p;
        p = fp;
    }

    return f;
}

fn _play_guessing_game() {
    println!("guess the number\n");
    let target = rand::thread_rng().gen_range(1, 101);

    let mut first = true;
    let mut guess = 0;
    while first || guess != target {
        if !first {
            println!("go {}", if target-guess > 0 {"up"} else {"down"});
        } else {
            first = false;
        }

        guess = _get_guess();
    }

    println!("you won");
}

fn _get_guess() -> i32 {
    print!("enter your guess: ");
    io::stdout().flush().unwrap();

    let mut guess = String::new();
    io::stdin()
        .read_line(&mut guess)
        .expect("failed to read line");

    return guess.trim().parse().expect("enter a number");
}
