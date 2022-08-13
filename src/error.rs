// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Copyright 2022 Oxide Computer Company

use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Message(String),

    Eof,
    Syntax,
    ExpectedBoolean,
    ExpectedInteger,
    ExpectedString,
    ExpectedNull,
    ExpectedArray,
    ExpectedEnum,
    TrailingBytes,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::Syntax => formatter.write_str("unexpected synatx"),
            Error::ExpectedBoolean => formatter.write_str("expected boolean"),
            Error::ExpectedInteger => formatter.write_str("expected integer"),
            Error::ExpectedString => formatter.write_str("expected string"),
            Error::ExpectedNull => formatter.write_str("expected end of null"),
            Error::ExpectedArray => formatter.write_str("expected end of array"),
            Error::ExpectedEnum => formatter.write_str("expected end of enum"),
            Error::TrailingBytes => formatter.write_str("unexpected trailing bytes"),

        }
    }
}

impl std::error::Error for Error {}
