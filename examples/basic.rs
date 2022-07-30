use std::error::Error;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use dbstruct::traits::DataStore;
use dbstruct::wrappers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Song;
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct Preferences;
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Account;

const PLAYING_DEFAULT: bool = true;

// macro input:
#[allow(dead_code)]
struct MacroInput {
    // lets say the actual name is Db
    pub queue: Vec<Song>,
    //#[dbstruct(Default("true")]
    pub playing: bool,
    //#[dbstruct(Default)]
    pub preferences: Preferences,
    pub account: Option<Account>,
}

// note only make fields pub if they are in the
// original struct
// start macro output
// note the macro would use absolute paths for everything
pub struct MacroOutput<DS: DataStore> {
    ds: DS,
    queue_len: Arc<AtomicUsize>,
}

impl<DS> MacroOutput<DS>
where
    DS: DataStore + Clone
{
    pub fn new(ds: DS) -> Result<Self, dbstruct::Error<DS::Error>> {
        let queue_len = 0; // TODO: decide where to store
                           // this in DB and how to load it
        Ok(Self {
            ds,
            queue_len: Arc::new(AtomicUsize::new(queue_len)),
        })
    }

    pub fn queue(&self) -> wrappers::Vec<Song, DS> {
        wrappers::Vec::new(self.ds.clone(), 1, self.queue_len.clone())
    }
    pub fn playing(&self) -> wrappers::DefaultValue<bool, DS> {
        wrappers::DefaultValue::new(self.ds.clone(), 2, PLAYING_DEFAULT)
    }
    pub fn preferences(&self) -> wrappers::DefaultTrait<Preferences, DS> {
        wrappers::DefaultTrait::new(self.ds.clone(), 3)
    }
    pub fn account(&self) -> wrappers::OptionValue<Account, DS> {
        wrappers::OptionValue::new(self.ds.clone(), 4)
    }
}
// end macro output

pub fn main() -> Result<(), Box<dyn Error>> {

    let ds = sled::Config::default()
        .temporary(true)
        .open()?
        .open_tree("MacroInput")?;
    let db = MacroOutput::new(ds)?;

    let last = db.queue().pop()?;
    assert_eq!(last, None);
    db.queue().push(Song {})?;
    let last = db.queue().pop()?;
    assert_eq!(last, Some(Song {}));

    let playing = db.playing().get()?;
    assert_eq!(playing, PLAYING_DEFAULT);

    let preferences = db.preferences().get()?;
    assert_eq!(preferences, Default::default());

    db.account().set(&Account {})?;
    let account = db.account().get()?;
    assert_eq!(account, Some(Account {}));

    println!("Hello, world!");

    Ok(())
}
