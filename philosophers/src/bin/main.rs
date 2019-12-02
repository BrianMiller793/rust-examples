extern crate philosophers;

use std::sync::Arc;
use philosophers::table::{Table, Philosopher, PhilosopherState};
use philosophers::threadpool;

fn main() {
    let mut pool = threadpool::ThreadPool::new(5);
    let table: Arc<Table> = Arc::new(Table::new());
    let mut hungry_philosophers: Vec<Philosopher> = gen_philosophers();

    while let Some(mut philosopher) = hungry_philosophers.pop() {
        let table = Arc::clone(&table);
        pool.execute(move || {
            println!("{}?", philosopher.name);
            let seating_attempt = philosopher.take_seat(&table);
            let forks_and_place = seating_attempt.unwrap();
            println!("{}{}",
                     philosopher.name, forks_and_place.3);
            philosopher.eat(forks_and_place, &table);
            println!("{}-", philosopher.name);
        });
    }

    pool.wait();
}

fn gen_philosophers() -> Vec<Philosopher> {
    let mut philosophers: Vec<Philosopher> = Vec::new();
    let names = [ "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
                  "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X" ];
    for name in &names {
        philosophers.push(Philosopher {
            name: name.to_string(), state: PhilosopherState::Hungry })
    }
    philosophers
}
