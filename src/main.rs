use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use bit_vec::BitVec;
use csv::StringRecord;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transaction {
    Deposit(u16, u32, f64),
    Withdrawal(u16, u32, f64),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Client {
    pub id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, {:.4}, {:.4}, {:.4}, {}",
            self.id, self.available, self.held, self.total, self.locked,
        )
    }
}

impl Client {
    pub fn new(id: u16) -> Self {
        Self {
            id,
            available: 0.0f64,
            held: 0.0f64,
            total: 0.0f64,
            locked: false,
        }
    }
}

#[derive(Default, Debug)]
pub struct Engine {
    transactions: std::vec::Vec<f64>,
    clients: std::vec::Vec<Option<Client>>,
    disbutes: BitVec,
}

impl Engine {
    /// Create a new engine
    pub fn new() -> Self {
        let transactions = vec![0.0f64; u32::MAX as usize];
        let clients = vec![None; u16::MAX as usize];
        let disbutes = BitVec::from_elem(u32::MAX as usize, false);

        Self {
            transactions,
            clients,
            disbutes,
        }
    }

    /// Read csv records from a file
    pub fn read_file(&mut self, file: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let f = File::open(file)?;
        let reader = BufReader::new(f);

        let mut csv_reader = csv::Reader::from_reader(reader);
        for record in csv_reader.records() {
            let record = record?;

            self.parse_record(&record).and_then(|record| {
                self.handle_record(record);
                Some(())
            });
        }
        Ok(())
    }

    /// Read csv records from a str
    pub fn from_str(&mut self, csv: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = csv::Reader::from_reader(csv.as_bytes());
        for record in reader.records() {
            let record = record?;

            self.parse_record(&record).and_then(|record| {
                self.handle_record(record);
                Some(())
            });
        }
        Ok(())
    }

    /// Handle a record or transaction
    pub fn handle_record(&mut self, record: Transaction) {
        match record {
            Transaction::Deposit(id, tx, amount) => self.transaction(id, tx, amount),
            Transaction::Withdrawal(id, tx, amount) => self.transaction(id, tx, -amount),
            Transaction::Dispute(id, tx) => self.dispute(id, tx),
            Transaction::Resolve(id, tx) => self.resolve(id, tx),
            Transaction::Chargeback(id, tx) => self.chargeback(id, tx),
        }
    }

    pub fn transaction(&mut self, id: u16, tx: u32, amount: f64) {
        if let Some(Some(client)) = self.clients.get_mut(id as usize) {
            if client.locked {
                return;
            }
            client.total += amount;
            client.available += amount;
        } else {
            let mut client = Client::new(id);
            client.available = amount;
            client.total = amount;
            self.clients[id as usize] = Some(client);
        }
        self.transactions[tx as usize] = amount;
    }

    /// Dispute a transaction
    pub fn dispute(&mut self, id: u16, tx: u32) {
        if let Some(Some(client)) = self.clients.get_mut(id as usize) {
            if let Some(amount) = self.transactions.get(tx as usize) {
                client.available -= amount;
                client.held += amount;
                self.disbutes.set(tx as usize, true);
            }
        }
    }

    /// Resolve a dispute
    pub fn resolve(&mut self, id: u16, tx: u32) {
        if let Some(Some(client)) = self.clients.get_mut(id as usize) {
            if let Some(amount) = self.transactions.get(tx as usize) {
                if Some(true) == self.disbutes.get(tx as usize) {
                    client.available += amount;
                    client.held -= amount;
                    self.disbutes.set(tx as usize, false);
                }
            }
        }
    }

    /// Handle a chargeback
    ///
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn. This means that the clients held funds and
    /// total funds should decrease by the amount previously disputed. If a chargeback occurs the
    /// client's account should be immediately frozen.
    pub fn chargeback(&mut self, id: u16, tx: u32) {
        if let Some(Some(client)) = self.clients.get_mut(id as usize) {
            if let Some(amount) = self.transactions.get(tx as usize) {
                if Some(true) == self.disbutes.get(tx as usize) {
                    client.total -= amount;
                    client.held -= amount;
                    client.locked = true;
                    self.disbutes.set(tx as usize, false);
                }
            }
        }
    }

    /// Parse a StringRecord into a Transaction
    pub fn parse_record(&self, record: &StringRecord) -> Option<Transaction> {
        match &record[0] {
            "deposit" => {
                let client_id: u16 = record[1].trim().parse().unwrap();
                let tx: u32 = record[2].trim().parse().unwrap();
                let amount: f64 = record[3].trim().parse().unwrap();
                return Some(Transaction::Deposit(client_id, tx, amount));
            }
            "withdrawal" => {
                let client_id: u16 = record[1].trim().parse().unwrap();
                let tx: u32 = record[2].trim().parse().unwrap();
                let amount: f64 = record[3].trim().parse().unwrap();
                return Some(Transaction::Withdrawal(client_id, tx, amount));
            }
            "dispute" => {
                let client_id: u16 = record[1].trim().parse().unwrap();
                let tx: u32 = record[2].trim().parse().unwrap();
                return Some(Transaction::Dispute(client_id, tx));
            }
            "resolve" => {
                let client_id: u16 = record[1].trim().parse().unwrap();
                let tx: u32 = record[2].trim().parse().unwrap();
                return Some(Transaction::Resolve(client_id, tx));
            }
            "chargeback" => {
                let client_id: u16 = record[1].trim().parse().unwrap();
                let tx: u32 = record[2].trim().parse().unwrap();
                return Some(Transaction::Chargeback(client_id, tx));
            }
            _ => {
                eprintln!("Unknown transaction type: {:?}", &record[0]);
                return None;
            }
        }
    }

    /// Print the client list to file
    fn dump_clients(&self) {
        println!("client, available, held, total, locked");
        self.clients
            .iter()
            .filter_map(|c| *c)
            .for_each(|client| println!("{}", client));
    }

    #[cfg(test)]
    pub fn get_client_mut(&mut self, index: usize) -> Option<&Client> {
        if let Some(Some(client)) = self.clients.get(index) {
            return Some(client);
        }
        None
    }

    #[cfg(test)]
    pub fn get_disputes(&self) -> &BitVec {
        return &self.disbutes;
    }

    #[cfg(test)]
    pub fn get_transactions(&self) -> &BitVec {
        return &self.disbutes;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args().nth(1).expect("No csv file provided");
    let mut engine = Engine::new();
    engine.read_file(&Path::new(&file))?;
    engine.dump_clients();
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn read_line() {
        let mut engine = Engine::new();

        let csv = "type, client, tx, amount
deposit, 1, 1, 1.0";

        let res = engine.from_str(&csv);
        assert!(res.is_ok())
    }

    #[test]
    fn parse_records() {
        let engine = Engine::new();

        let records = [
            StringRecord::from(vec!["deposit", "1", "1", "1.0"]),
            StringRecord::from(vec!["withdrawal", "1", "1", "1.0"]),
            StringRecord::from(vec!["dispute", "1", "1", ""]),
            StringRecord::from(vec!["resolve", "1", "1", ""]),
            StringRecord::from(vec!["chargeback", "1", "1", ""]),
        ];

        let expected = [
            Transaction::Deposit(1, 1, 1.0f64),
            Transaction::Withdrawal(1, 1, 1.0f64),
            Transaction::Dispute(1, 1),
            Transaction::Resolve(1, 1),
            Transaction::Chargeback(1, 1),
        ];

        records.into_iter().enumerate().for_each(|(i, record)| {
            assert!(engine.parse_record(&record).unwrap() == expected[i]);
        });
    }

    #[test]
    fn handle_record() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Withdrawal(1, 1, 1.0f64),
            Transaction::Deposit(1, 1, 2.0f64),
        ];

        engine.handle_record(records[0]);
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);

        engine.handle_record(records[1]);
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 1.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 1.0f64);

        engine.handle_record(records[2]);
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 3.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 3.0f64);
    }

    #[test]
    fn disbute() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(1, 1),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);

        assert!(engine.get_disputes().get(1) == Some(true));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);
    }

    #[test]
    fn resolve() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(1, 1),
            Transaction::Resolve(1, 1),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);
        engine.handle_record(records[2]);

        assert!(engine.get_disputes().get(1) == Some(false));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);
    }

    #[test]
    fn chargeback() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(1, 1),
            Transaction::Chargeback(1, 1),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);
        engine.handle_record(records[2]);

        assert!(engine.get_disputes().get(1) == Some(false));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().locked);
    }

    #[test]
    fn dispute_no_client() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(2, 1),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);

        assert!(engine.get_disputes().get(2) == Some(false));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);
    }

    #[test]
    fn dispute_no_tx() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(1, 2),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);

        assert!(engine.get_disputes().get(1) == Some(false));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);
    }

    #[test]
    fn resolve_no_dispute() {
        let mut engine = Engine::new();

        let records = [
            Transaction::Deposit(1, 1, 2.0f64),
            Transaction::Dispute(1, 1),
            Transaction::Resolve(1, 2),
        ];

        engine.handle_record(records[0]);
        engine.handle_record(records[1]);
        engine.handle_record(records[2]);

        assert!(engine.get_disputes().get(1) == Some(true));
        assert!(engine.get_client_mut(1 as usize).unwrap().available == 0.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().held == 2.0f64);
        assert!(engine.get_client_mut(1 as usize).unwrap().total == 2.0f64);
    }
}
