use std::path::Path;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Transaction{
    Deposit(u16, u32, f32),
    Withdrawal(u16, u32),
    Dispute(u16,u32),
    Resolve(u16, u32),
    Chargeback(u16, u32),
}

#[derive(Clone, Copy, Debug)]
pub struct Client {
    available: f32,
    held: f32,
    total: f32,
    locked: bool
}

impl Client {
    pub fn available(&self) -> f32 {
        self.available
    }
    
    pub fn held(&self) -> f32 {
        self.held
    }
    
    pub fn total(&self) -> f32 {
       self.total + self.held 
    }
    
    pub fn lock(&mut self) {
        self.locked = true;
    }
}

pub struct Engine {
    transactions: std::vec::Vec<Transaction>,
    clients: std::vec::Vec<Client>,
}

impl Engine {
    pub fn new() -> Self { unimplemented!()}

    pub fn read_file(&mut self, file: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn from_str(&mut self, data: &str) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
    
fn main() {
    println!("Hello, world!");
}
