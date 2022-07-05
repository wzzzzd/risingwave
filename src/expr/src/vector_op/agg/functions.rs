// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use risingwave_common::array::{Array, ListRef, ListValue, StructRef, StructValue};
use risingwave_common::error::{ErrorCode, Result};

/// Essentially `RTFn` is an alias of the specific Fn. It was aliased not to
/// shorten the `where` clause of `GeneralAgg`, but to workaround an compiler
/// error`[E0582`]: binding for associated type `Output` references lifetime `'a`,
/// which does not appear in the trait input types.
pub trait RTFn<'a, T, R>: Send + 'static
where
    T: Array,
    R: Array,
{
    fn eval(
        &mut self,
        result: Option<<R as Array>::OwnedItem>,
        input: Option<<T as Array>::RefItem<'a>>,
    ) -> Result<Option<<R as Array>::OwnedItem>>;
}

impl<'a, T, R, Z> RTFn<'a, T, R> for Z
where
    T: Array,
    R: Array,
    Z: Send
        + 'static
        + Fn(
            Option<<R as Array>::OwnedItem>,
            Option<<T as Array>::RefItem<'a>>,
        ) -> Result<Option<<R as Array>::OwnedItem>>,
{
    fn eval(
        &mut self,
        result: Option<<R as Array>::OwnedItem>,
        input: Option<<T as Array>::RefItem<'a>>,
    ) -> Result<Option<<R as Array>::OwnedItem>> {
        self.call((result, input))
    }
}

use std::convert::From;
use std::ops::Add;

use risingwave_common::types::ScalarRef;

pub fn sum<R, T>(result: Option<R>, input: Option<T>) -> Result<Option<R>>
where
    R: From<T> + Add<Output = R> + Copy,
{
    let res = match (result, input) {
        (_, None) => result,
        (None, Some(i)) => Some(R::from(i)),
        (Some(r), Some(i)) => Some(r + R::from(i)),
    };
    Ok(res)
}

pub fn min<'a, T>(result: Option<T::ScalarType>, input: Option<T>) -> Result<Option<T::ScalarType>>
where
    T: ScalarRef<'a> + PartialOrd,
    T::ScalarType: PartialOrd,
{
    let res = match (result, input) {
        (None, _) => input.map(|x| x.to_owned_scalar()),
        (Some(r), None) => Some(r),
        (Some(r), Some(i)) => {
            let i = i.to_owned_scalar();
            Some(if r < i { r } else { i })
        }
    };
    Ok(res)
}

pub fn min_str<'a>(r: Option<String>, i: Option<&'a str>) -> Result<Option<String>> {
    min(r, i)
}

pub fn min_struct<'a>(
    r: Option<StructValue>,
    i: Option<StructRef<'a>>,
) -> Result<Option<StructValue>> {
    min(r, i)
}

pub fn min_list<'a>(r: Option<ListValue>, i: Option<ListRef<'a>>) -> Result<Option<ListValue>> {
    min(r, i)
}

pub fn max<'a, T>(result: Option<T::ScalarType>, input: Option<T>) -> Result<Option<T::ScalarType>>
where
    T: ScalarRef<'a> + PartialOrd,
    T::ScalarType: PartialOrd,
{
    let res = match (result, input) {
        (None, _) => input.map(|x| x.to_owned_scalar()),
        (Some(r), None) => Some(r),
        (Some(r), Some(i)) => {
            let i = i.to_owned_scalar();
            Some(if r > i { r } else { i })
        }
    };
    Ok(res)
}

pub fn max_str<'a>(r: Option<String>, i: Option<&'a str>) -> Result<Option<String>> {
    max(r, i)
}

pub fn max_struct<'a>(
    r: Option<StructValue>,
    i: Option<StructRef<'a>>,
) -> Result<Option<StructValue>> {
    max(r, i)
}

pub fn max_list<'a>(r: Option<ListValue>, i: Option<ListRef<'a>>) -> Result<Option<ListValue>> {
    max(r, i)
}

/// create table t(v1 int);
/// insert into t values (null);
/// select count(*) from t; gives 1.
/// select count(v1) from t; gives 0.
/// select sum(v1) from t; gives null
pub fn count<T>(result: Option<i64>, input: Option<T>) -> Result<Option<i64>> {
    let res = match (result, input) {
        (None, None) => Some(0),
        (Some(r), None) => Some(r),
        (None, Some(_)) => Some(1),
        (Some(r), Some(_)) => Some(r + 1),
    };
    Ok(res)
}

pub fn count_str(r: Option<i64>, i: Option<&str>) -> Result<Option<i64>> {
    count(r, i)
}

pub fn count_struct(r: Option<i64>, i: Option<StructRef<'_>>) -> Result<Option<i64>> {
    count(r, i)
}

pub fn count_list(r: Option<i64>, i: Option<ListRef<'_>>) -> Result<Option<i64>> {
    count(r, i)
}

pub fn agg_str(result: Option<String>, input: Option<&str>) -> Result<Option<String>> {
    let res = match (result, input) {
        (None, _) => input.map(&str::to_string),
        (Some(r), None) => Some(r),
        (Some(r), Some(i)) => Some(r + i),
    };
    Ok(res)
}

pub struct SingleValue {
    count: usize,
}

impl SingleValue {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl<'a, T> RTFn<'a, T, T> for SingleValue
where
    T: Array,
{
    fn eval(
        &mut self,
        _result: Option<<T as Array>::OwnedItem>,
        input: Option<<T as Array>::RefItem<'a>>,
    ) -> Result<Option<<T as Array>::OwnedItem>> {
        self.count += 1;
        if self.count > 1 {
            Err(ErrorCode::InternalError(
                "SingleValue aggregation can only accept exactly one value. But there is more than one.".to_string(),
            )
              .into())
        } else {
            Ok(input.map(|x| x.to_owned_scalar()))
        }
    }
}
