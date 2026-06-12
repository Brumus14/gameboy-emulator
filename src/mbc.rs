use crate::memory::Address;

pub trait Mbc {
    fn read(&self, address: Address);
    fn write(&mut self, address: Address);
}
