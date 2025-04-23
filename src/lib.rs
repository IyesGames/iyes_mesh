pub mod checksum;
pub mod descriptor;
pub mod header;

pub mod read;
pub mod write;

pub mod io;

pub mod mesh;

pub const FORMAT_VERSION: u16 = 1;
pub const MAGIC: [u8; 4] = [b'I', b'y', b'M', b'A'];

pub type HashMap<K, V> = rapidhash::RapidHashMap<K, V>;
pub type HashSet<T> = rapidhash::RapidHashSet<T>;
