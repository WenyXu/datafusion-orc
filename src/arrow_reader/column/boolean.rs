use snafu::OptionExt;

use crate::arrow_reader::column::present::new_present_iter;
use crate::arrow_reader::column::{Column, NullableIterator};
use crate::arrow_reader::Stripe;
use crate::error::{InvalidColumnSnafu, Result};
use crate::proto::stream::Kind;
use crate::reader::decode::boolean_rle::BooleanIter;

pub fn new_boolean_iter(column: &Column, stripe: &Stripe) -> Result<NullableIterator<bool>> {
    let present = new_present_iter(column, stripe)?.collect::<Result<Vec<_>>>()?;
    let rows: usize = present.iter().filter(|&p| *p).count();

    let iter = stripe
        .stream_map
        .get(column, Kind::Data)
        .map(|reader| {
            Box::new(BooleanIter::new(reader, rows))
                as Box<dyn Iterator<Item = Result<bool>> + Send>
        })
        .context(InvalidColumnSnafu { name: &column.name })?;

    Ok(NullableIterator {
        present: Box::new(present.into_iter()),
        iter,
    })
}
