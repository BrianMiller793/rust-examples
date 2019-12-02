extern crate parking_lot;
use self::parking_lot::{Mutex, MutexGuard};
use std::{thread, time};

// Settings have two forks, and a Mutex to act as a seating place.
pub struct Setting {
    left_fork: usize,
    right_fork: usize,
    place_index: usize,
    place: Mutex<()>,
}

pub type ForksAndPlace<'a> = 
    (Option<MutexGuard<'a, ()>>, // Left fork TryLock
     Option<MutexGuard<'a, ()>>, // Right fork TryLock
     MutexGuard<'a, ()>,         // Place lock
     usize);                     // Place index

pub type SeatingResult<'a> = Result<ForksAndPlace<'a>, ()>;

pub struct Table {
    forks: [Mutex<()>; 5],      // Each philosopher must have two forks to eat.
    settings: [Setting; 5],     // Each philosopher shall occupy one seat.
    is_seating: Mutex<()>,      // Philosopher is currently seating.
}

#[derive(Copy, Clone)]
pub enum PhilosopherState {
    Hungry,
    Eating,
    Thinking,
    Waiting,
}

#[derive(Clone)]
pub struct Philosopher {
    pub state: PhilosopherState,
    pub name: String,
}

impl Philosopher {
    /// Seat a philosopher at the table.
    /// The first attempt is to find a setting with two available forks.
    /// If that fails, then the next avaiable setting is returned.
    /// Returns SeatingResult
    pub fn take_seat<'a>(&self, table: &'a Table)
        -> SeatingResult<'a> {
        let _is_seating = table.is_seating.lock();
        let mut left_lock_index: usize = 0;
        let mut right_lock_index: usize = 0;
        let mut place_index: usize = 0;

        // Find a seat with two available forks.
        let mut setting: Option<&Setting> = table.settings
            .iter()
            .find(|s| {
                  let left_lock = table.forks[s.left_fork].try_lock();
                  let right_lock = table.forks[s.right_fork].try_lock();
                  let place_lock = s.place.try_lock();
                  let have_both_forks =
                      match place_lock { Some(_) => true, None => false } &&
                      match left_lock { Some(_) => true, None => false } &&
                      match right_lock { Some(_) => true, None => false };
                  if have_both_forks {
                      left_lock_index = s.left_fork;
                      right_lock_index = s.right_fork;
                      place_index = s.place_index;
                  }
                  have_both_forks
            });

        if match setting { Some(_) => true, None => false} {
            return Ok((table.forks[left_lock_index].try_lock(),
                       table.forks[right_lock_index].try_lock(),
                       table.settings[place_index].place.lock(),
                       place_index));
        }

        // If no seats with two forks are available, get any vacant seat.
        // Do not acquire the forks here, as that leads to conflict later,
        // because a mutex cannot be released until it goes out of scope.
        setting = table.settings.iter()
            .find(|s| {
                let place_lock = s.place.try_lock();
                if match place_lock {Some(_) => true, None => false} {
                    left_lock_index = s.left_fork;
                    right_lock_index = s.right_fork;
                    place_index = s.place_index;
                }
                match place_lock {Some(_) => true, None => false}
            });

        // Return both forks as not acquired, even if one may be available.
        if match setting { Some(_) => true, None => false} {
            return Ok((None,
                       None,
                       table.settings[place_index].place.lock(),
                       place_index));
        }

        Err(())
    }

    // Philosopher possibly waits on forks, and then eats.
    // Forks are released when seating goes out of scope.
    pub fn eat(&mut self, seating: ForksAndPlace, table: &Table) {
        let _left_guard: MutexGuard<()>;
        let _right_guard: MutexGuard<()>;
        let left_fork = table.settings[seating.3].left_fork;
        let right_fork = table.settings[seating.3].right_fork;

        if match seating.0 {None => true, Some(_) => false} {
            println!("{}{} ({})", self.name, seating.3, left_fork);
            _left_guard = table.forks[left_fork].lock();
        }

        if match seating.1 {None => true, Some(_) => false} {
            println!("{}{}  {} ({})", self.name, seating.3, left_fork, right_fork);
            _right_guard = table.forks[right_fork].lock();
        }

        println!("{}{}  {}  {}", self.name, seating.3, left_fork, right_fork);
        let sleep_time = time::Duration::from_millis(50);
        thread::sleep(sleep_time);
        println!("{}{}", self.name, seating.3);
        self.state = PhilosopherState::Thinking;
    }
}

impl Table {
    pub fn new() -> Table {
        Table {
            forks: [
                Mutex::new(()),
                Mutex::new(()),
                Mutex::new(()),
                Mutex::new(()),
                Mutex::new(()),
            ],
            settings: [
                Setting {left_fork: 4, right_fork: 0,
                         place_index: 0, place: Mutex::new(())},
                Setting {left_fork: 0, right_fork: 1,
                         place_index: 1, place: Mutex::new(())},
                Setting {left_fork: 1, right_fork: 2,
                         place_index: 2, place: Mutex::new(())},
                Setting {left_fork: 2, right_fork: 3,
                         place_index: 3, place: Mutex::new(())},
                Setting {left_fork: 4, right_fork: 3, // Left-handed
                         place_index: 4, place: Mutex::new(())},
            ],
            is_seating: Mutex::new(())
        }
    }

    /// Get the total count of place settings.
    pub fn count(&self) -> usize {
        self.settings.len()
    }

    pub fn available_settings(&self) -> usize {
        self.settings
            .iter()
            .filter(|s| match s.place.try_lock()
                    {Some(_) => true, None => false,})
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count() {
        let table = Table::new();

        assert_eq!(5, table.count());
    }

    #[test]
    fn available_settings() {
        let table = Table::new();

        assert_eq!(5, table.available_settings());
    }

    #[test]
    fn seat_one_philosopher() {
        let table = Table::new();
        let philosopher = Philosopher {
            state: PhilosopherState::Hungry, name: "Alice".to_string() };

        let seated = philosopher.take_seat(&table);
        assert!(seated.is_ok());

        let available_settings = table.available_settings();
        assert_eq!(4, available_settings);
    }

    #[test]
    // This tests placing philosophers.  The mutex pair in `seated` goes
    // out of scope after the assert.
    fn seat_all_philosophers() {
        let table = Table::new();
        let mut seats = 5;
        let names = ["Alice", "Bob", "Charlie", "Dave", "Edgar"];
        let mut occupied_seats: Vec<SeatingResult> = Vec::new();

        for name in &names {
            let philosopher = Philosopher {
                state: PhilosopherState::Hungry, name: name.to_string() };
            let seated = philosopher.take_seat(&table);
            assert!(seated.is_ok());

            // Result must be saved, or seat is lost
            // In practice, the result would be in the philosopher's thread
            occupied_seats.push(seated);

            seats = seats - 1;
            assert_eq!(seats, table.available_settings());
        }

        assert_eq!(0, table.available_settings());
    }

    #[test]
    fn seat_too_many_philosophers() {
        let table = Table::new();
        let names = ["Alice", "Bob", "Charlie", "Dave", "Edgar"];
        let mut occupied_seats: Vec<SeatingResult> = Vec::new();

        // Fill the table with philosophers
        for name in &names {
            let philosopher = Philosopher {
                state: PhilosopherState::Hungry, name: name.to_string() };
            let seated = philosopher.take_seat(&table);
            assert!(seated.is_ok());
            occupied_seats.push(seated);
        }

        // Five places have been taken, none available.
        assert_eq!(0, table.available_settings());

        // Sorry, no love for Ringo.
        let philosopher = Philosopher {
            state: PhilosopherState::Hungry, name: "Ringo".to_string() };
        let seated = philosopher.take_seat(&table);
        match seated {
            Err(_) => assert!(true),
            Ok(_) => assert!(false),
        }
    }

    #[test]
    fn philosopher_eats() {
        let table = Table::new();
        let mut philosopher = Philosopher {
            state: PhilosopherState::Hungry, name: "Alice".to_string() };

        let seated = philosopher.take_seat(&table);
        assert!(seated.is_ok());
        let forks_and_place = seated.unwrap();
        assert_eq!(0, forks_and_place.3);
        assert!(match forks_and_place.0 {Some(_) => true, None => false});
        assert!(match forks_and_place.1 {Some(_) => true, None => false});

        // Use the forks
        philosopher.eat(forks_and_place, &table);
        assert!(match philosopher.state {
            PhilosopherState::Thinking => true,
            PhilosopherState::Hungry => false,
            PhilosopherState::Eating => false,
            PhilosopherState::Waiting => false,
        });

        // Forks should now be released, and available.
        let left_fork = table.forks[0].try_lock();
        let right_fork = table.forks[1].try_lock();
        assert!(match left_fork {Some(_) => true, None => false});
        assert!(match right_fork {Some(_) => true, None => false});
    }
}

