use chrono::{Days, NaiveDate};
use snafu::OptionExt;

use crate::arrow_reader::column::present::new_present_iter;
use crate::arrow_reader::column::{Column, NullableIterator};
use crate::arrow_reader::Stripe;
use crate::error::{self, Result};
use crate::proto::stream::Kind;
use crate::reader::decode::get_direct_signed_rle_reader;

pub const UNIX_EPOCH_FROM_CE: i32 = 719_163;

pub struct DateIterator {
    data: Box<dyn Iterator<Item = Result<i64>> + Send>,
}

pub fn convert_date(data: i64) -> Result<NaiveDate> {
    let days = Days::new(data.unsigned_abs());
    // safe unwrap as is valid date
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let date = if data.is_negative() {
        epoch.checked_sub_days(days)
    } else {
        epoch.checked_add_days(days)
    };
    date.context(error::AddDaysSnafu)
}

impl Iterator for DateIterator {
    type Item = Result<NaiveDate>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.next() {
            Some(Ok(data)) => Some(convert_date(data)),
            Some(Err(err)) => Some(Err(err)),
            None => None,
        }
    }
}

pub fn new_date_iter(column: &Column, stripe: &Stripe) -> Result<NullableIterator<NaiveDate>> {
    let present = new_present_iter(column, stripe)?.collect::<Result<Vec<_>>>()?;

    let data = stripe
        .stream_map
        .get(column, Kind::Data)
        .map(|reader| get_direct_signed_rle_reader(column, reader))
        .context(error::InvalidColumnSnafu { name: &column.name })??;

    Ok(NullableIterator {
        present: Box::new(present.into_iter()),
        iter: Box::new(DateIterator { data }),
    })
}
