// SPDX-FileCopyrightText: © 2022 Svix Authors
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, collections::HashSet, error::Error as StdError, ops::Deref, str::FromStr};

use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, FromRequestParts, Query},
    BoxError,
};
use chrono::{DateTime, Utc};
use http::{request::Parts, Request};
use regex::Regex;
use sea_orm::{ColumnTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use validator::{Validate, ValidationError};

use crate::{
    core::types::{BaseId, EventTypeName, EventTypeNameSet},
    error::{Error, HttpError, Result, ValidationErrorItem},
};

pub mod patch;
use patch::UnrequiredField;

const fn default_limit() -> PaginationLimit {
    PaginationLimit(50)
}

const PAGINATION_LIMIT_CAP_HARD: bool = true;
const PAGINATION_LIMIT_CAP_LIMIT: u64 = 250;
// TODO: Should probably use lazy_static and format! to make this instead of repeating the 250
// figure at some point
const PAGINATION_LIMIT_ERROR: &str = "Given limit must not exceed 250";

#[derive(Debug, Deserialize, Validate)]
pub struct Pagination<T: Validate> {
    #[validate]
    #[serde(default = "default_limit")]
    pub limit: PaginationLimit,
    #[validate]
    pub iterator: Option<T>,
}

#[derive(Debug)]
pub struct PaginationLimit(pub u64);

impl<'de> Deserialize<'de> for PaginationLimit {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let limit = u64::deserialize(deserializer)?;

        // Want hard limits to stay the same so they can be validated
        if !PAGINATION_LIMIT_CAP_HARD && limit > PAGINATION_LIMIT_CAP_LIMIT {
            Ok(PaginationLimit(PAGINATION_LIMIT_CAP_LIMIT))
        } else {
            Ok(PaginationLimit(limit))
        }
    }
}

impl Validate for PaginationLimit {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        let mut errs = validator::ValidationErrors::new();

        if self.0 > PAGINATION_LIMIT_CAP_LIMIT {
            errs.add(
                "limit",
                validation_error(Some("pagination"), Some(PAGINATION_LIMIT_ERROR)),
            );
        }

        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReversibleIterator<T: Validate> {
    Normal(T),
    Prev(T),
}

impl<'de, T: 'static + Deserialize<'de> + Validate + From<String>> Deserialize<'de>
    for ReversibleIterator<T>
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| {
            if let Some(s) = s.strip_prefix('-') {
                ReversibleIterator::Prev(T::from(s.to_owned()))
            } else {
                ReversibleIterator::Normal(T::from(s))
            }
        })
    }
}

impl<T: Validate> Validate for ReversibleIterator<T> {
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            ReversibleIterator::Normal(val) => val.validate(),
            ReversibleIterator::Prev(val) => val.validate(),
        }
    }
}

/// For use in creating a [`ReversibleIterator`] from `before` and `after` timestamps should one not
/// already be present
pub fn iterator_from_before_or_after<I: BaseId<Output = I> + Validate>(
    iterator: Option<ReversibleIterator<I>>,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
) -> Option<ReversibleIterator<I>> {
    iterator.or_else(|| {
        before
            .map(|time| ReversibleIterator::Normal(I::start_id(time)))
            .or_else(|| after.map(|time| ReversibleIterator::Prev(I::end_id(time))))
    })
}

/// Applies sorting and filtration to a query from its iterator, sort column, and limit
pub fn apply_pagination<
    Q: QuerySelect + QueryOrder + QueryFilter,
    C: ColumnTrait,
    I: BaseId<Output = I> + Validate + Into<sea_orm::Value>,
>(
    query: Q,
    sort_column: C,
    limit: u64,
    iterator: Option<ReversibleIterator<I>>,
) -> Q {
    let query = query.limit(limit + 1);

    match iterator {
        Some(ReversibleIterator::Prev(id)) => {
            query.order_by_asc(sort_column).filter(sort_column.gt(id))
        }

        Some(ReversibleIterator::Normal(id)) => {
            query.order_by_desc(sort_column).filter(sort_column.lt(id))
        }

        None => query.order_by_desc(sort_column),
    }
}

#[derive(Serialize)]
pub struct EmptyResponse {}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T: Clone> {
    pub data: Vec<T>,
    pub iterator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_iterator: Option<String>,
    pub done: bool,
}

pub trait ModelIn {
    type ActiveModel;

    fn update_model(self, model: &mut Self::ActiveModel);
}

fn list_response_inner<T: ModelOut>(
    mut data: Vec<T>,
    limit: usize,
    is_prev_iter: bool,
    supports_prev_iterator: bool,
) -> ListResponse<T> {
    let done = data.len() <= limit;

    if data.len() > limit {
        if is_prev_iter {
            data = data.drain(data.len() - limit..).collect();
        } else {
            data.truncate(limit);
        }
    }

    let prev_iterator = if supports_prev_iterator {
        data.first().map(|x| format!("-{}", x.id_copy()))
    } else {
        None
    };
    let iterator = data.last().map(|x| x.id_copy());

    ListResponse {
        data,
        iterator,
        prev_iterator,
        done,
    }
}

pub trait ModelOut: Clone {
    fn id_copy(&self) -> String;

    fn list_response(data: Vec<Self>, limit: usize, is_prev_iter: bool) -> ListResponse<Self> {
        list_response_inner(data, limit, is_prev_iter, true)
    }

    fn list_response_no_prev(data: Vec<Self>, limit: usize) -> ListResponse<Self> {
        list_response_inner(data, limit, false, false)
    }
}

// Helper method to simplify the somewhat egregious API for creating a ValidationError
pub fn validation_error(code: Option<&'static str>, msg: Option<&'static str>) -> ValidationError {
    ValidationError {
        code: std::borrow::Cow::from(code.unwrap_or("validation")),
        message: msg.map(std::borrow::Cow::from),
        params: std::collections::HashMap::new(),
    }
}

/// Recursively searches a [`validator::ValidationErrors`] tree into a linear list of errors to be
/// sent to the user
fn validation_errors(
    acc_path: Vec<String>,
    err: validator::ValidationErrors,
) -> Vec<ValidationErrorItem> {
    err.into_errors()
        .into_iter()
        .flat_map(|(k, v)| {
            // Add the next field to the location
            let mut loc = acc_path.clone();
            loc.push(k.to_owned());

            match v {
                // If it's a [`validator::ValidationErrorsKind::Field`], then it will be a vector of
                // errors to map to [`ValidationErrorItem`]s and insert to [`out`] before the next
                // iteration
                validator::ValidationErrorsKind::Field(vec) => vec
                    .into_iter()
                    .map(|err| ValidationErrorItem {
                        loc: loc.clone(),
                        msg: err
                            .message
                            .unwrap_or(Cow::Borrowed("Validation error"))
                            .to_string(),
                        ty: "value_error".to_owned(),
                    })
                    .collect(),
                // If it is a [`validator::ValidationErrorsKind::Struct`], then it will be another
                // [`validator::ValidationErrors`] to search
                validator::ValidationErrorsKind::Struct(errors) => validation_errors(loc, *errors),

                // If it is a [`validator::ValidationErrorsKind::List`], then it will be an
                // [`std::collections::BTreeMap`] of [`validator::ValidationErrors`] to search
                validator::ValidationErrorsKind::List(map) => map
                    .into_iter()
                    .flat_map(|(k, v)| {
                        // Add the list index to the location
                        let mut loc = loc.clone();
                        loc.push(format!("[{k}]"));

                        validation_errors(loc, *v)
                    })
                    .collect(),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = Error;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self> {
        let b = bytes::Bytes::from_request(req, state).await.map_err(|e| {
            tracing::error!("Error reading body as bytes: {}", e);
            HttpError::internal_server_error(None, Some("Failed to read request body".to_owned()))
        })?;
        let mut de = serde_json::Deserializer::from_slice(&b);

        let value: T = serde_path_to_error::deserialize(&mut de).map_err(|e| {
            let mut path = e
                .path()
                .to_string()
                .split('.')
                .map(ToOwned::to_owned)
                .collect::<Vec<String>>();
            let inner = e.inner();

            let mut loc = vec!["body".to_owned()];
            loc.append(&mut path);
            HttpError::unprocessable_entity(vec![ValidationErrorItem {
                loc,
                msg: inner
                    .source()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| e.to_string()),
                ty: "value_error.jsondecode".to_owned(),
            }])
        })?;

        value.validate().map_err(|e| {
            HttpError::unprocessable_entity(validation_errors(vec!["body".to_owned()], e))
        })?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(|err| HttpError::bad_request(None, Some(err.to_string())))?;
        value.validate().map_err(|e| {
            HttpError::unprocessable_entity(validation_errors(vec!["query".to_owned()], e))
        })?;
        Ok(ValidatedQuery(value))
    }
}

impl<T> Deref for ValidatedQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This struct is slower than Query. Only use this if we need to pass arrays.
#[derive(Debug)]
pub struct MessageListFetchOptions {
    pub event_types: Option<EventTypeNameSet>,
    pub before: Option<DateTime<Utc>>,
}

#[async_trait]
impl<S> FromRequestParts<S> for MessageListFetchOptions
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        let pairs: Vec<(String, String)> =
            serde_urlencoded::from_str(parts.uri.query().unwrap_or_default())
                .map_err(|err| HttpError::bad_request(None, Some(err.to_string())))?;

        let mut before = None;
        let mut event_types = EventTypeNameSet(HashSet::<EventTypeName>::new());
        for (key, value) in pairs {
            if key == "event_types" {
                event_types.0.insert(EventTypeName(value));
            } else if key == "before" {
                before = Some(DateTime::<Utc>::from_str(&value).map_err(|_| {
                    HttpError::unprocessable_entity(vec![ValidationErrorItem {
                        loc: vec!["query".to_owned(), "before".to_owned()],
                        msg: "Unable to parse before".to_owned(),
                        ty: "value_error".to_owned(),
                    }])
                })?);
            }
        }
        let event_types = if event_types.0.is_empty() {
            None
        } else {
            Some(event_types)
        };
        Ok(MessageListFetchOptions {
            event_types,
            before,
        })
    }
}

pub async fn api_not_implemented() -> Result<()> {
    Err(HttpError::not_implemented(None, None).into())
}

pub fn validate_no_control_characters(str: &str) -> std::result::Result<(), ValidationError> {
    let re = Regex::new(r"[\x00-\x08]").unwrap();
    if re.is_match(str) {
        return Err(validation_error(
            Some("illegal_character"),
            Some("Control characters 0x00-0x08 not allowed."),
        ));
    }
    Ok(())
}

pub fn validate_no_control_characters_unrequired(
    str: &UnrequiredField<String>,
) -> std::result::Result<(), ValidationError> {
    match str {
        UnrequiredField::Absent => Ok(()),
        UnrequiredField::Some(str) => validate_no_control_characters(str),
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::{default_limit, validate_no_control_characters, validation_errors, Pagination};
    use crate::core::types::ApplicationUid;
    use crate::error::ValidationErrorItem;
    use serde_json::json;

    #[derive(Debug, Validate)]
    struct ValidationErrorTestStruct {
        #[validate(range(min = 10, message = "Below 10"))]
        a: u32,

        #[validate]
        b: ValidationErrorTestStructInner,

        #[validate]
        c: Vec<ValidationErrorTestStructInner>,
    }

    #[derive(Debug, Validate)]
    struct ValidationErrorTestStructInner {
        #[validate(range(max = 10, message = "Above 10"))]
        inner: u8,
    }

    #[test]
    fn test_validation_errors_fn() {
        let valid = ValidationErrorTestStruct {
            a: 11,
            b: ValidationErrorTestStructInner { inner: 1 },
            c: vec![
                ValidationErrorTestStructInner { inner: 2 },
                ValidationErrorTestStructInner { inner: 3 },
            ],
        };
        let invalid = ValidationErrorTestStruct {
            a: 9,
            b: ValidationErrorTestStructInner { inner: 11 },
            c: vec![
                ValidationErrorTestStructInner { inner: 12 },
                ValidationErrorTestStructInner { inner: 13 },
            ],
        };

        assert_eq!(valid.validate(), Ok(()));

        let errs = invalid.validate().unwrap_err();
        let errs = validation_errors(vec![], errs);

        assert_eq!(errs.len(), 4);

        assert!(errs.contains(&ValidationErrorItem {
            loc: vec!["a".to_owned()],
            msg: "Below 10".to_owned(),
            ty: "value_error".to_owned(),
        }));

        assert!(errs.contains(&ValidationErrorItem {
            loc: vec!["b".to_owned(), "inner".to_owned()],
            msg: "Above 10".to_owned(),
            ty: "value_error".to_owned(),
        }));

        assert!(errs.contains(&ValidationErrorItem {
            loc: vec!["c".to_owned(), "[0]".to_owned(), "inner".to_owned()],
            msg: "Above 10".to_owned(),
            ty: "value_error".to_owned(),
        }));
        assert!(errs.contains(&ValidationErrorItem {
            loc: vec!["c".to_owned(), "[1]".to_owned(), "inner".to_owned()],
            msg: "Above 10".to_owned(),
            ty: "value_error".to_owned(),
        }));
    }

    #[test]
    fn test_pagination_defaults() {
        let p: Pagination<ApplicationUid> = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p.limit.0, default_limit().0);
    }

    #[test]
    fn test_pagination_validation() {
        let p: Pagination<ApplicationUid> =
            serde_json::from_value(json!({"iterator": "$$invalid-appuid"})).unwrap();
        assert!(p.validate().is_err());

        let p: Pagination<ApplicationUid> =
            serde_json::from_value(json!({ "iterator": "valid-appuid"})).unwrap();
        p.validate().unwrap();
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct TestPaginationDeserializationStruct {
        iterator: super::ReversibleIterator<crate::core::types::MessageId>,
    }

    #[test]
    fn test_pagination_deserialization() {
        let a = serde_json::json!({"iterator": "msg_274DTsX0wVTSLvo91QopQgZrjDV"});
        let b = serde_json::json!({"iterator": "-msg_274DTsX0wVTSLvo91QopQgZrjDV"});

        assert_eq!(
            serde_json::from_value::<TestPaginationDeserializationStruct>(a).unwrap(),
            TestPaginationDeserializationStruct {
                iterator: super::ReversibleIterator::Normal(crate::core::types::MessageId(
                    "msg_274DTsX0wVTSLvo91QopQgZrjDV".to_owned()
                ))
            }
        );
        assert_eq!(
            serde_json::from_value::<TestPaginationDeserializationStruct>(b).unwrap(),
            TestPaginationDeserializationStruct {
                iterator: super::ReversibleIterator::Prev(crate::core::types::MessageId(
                    "msg_274DTsX0wVTSLvo91QopQgZrjDV".to_owned()
                ))
            }
        );
    }

    #[test]
    fn test_validate_no_control_characters() {
        let a = "A good string";
        let b = "A\u{0000} bad string";

        assert!(validate_no_control_characters(a).is_ok());
        assert!(validate_no_control_characters(b).is_err());
    }
}
