mod account;
mod transaction;
mod types;
mod wallet;

pub use account::*;
pub use transaction::*;
pub use types::*;
pub use wallet::*;

// @todo:schedule: Should be able to define transactions that occur every given time
// assigned to an account
// ie. Payroll |every 15 days| by X amount
// ie. Payroll |every 1st and 17th| by X amount
// ie. Payroll |twice a month| by X amount
//
// @todo:forecast: Should be able to see net worth move with the give scheduled transactions 
