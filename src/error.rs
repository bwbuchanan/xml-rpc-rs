//! Defines error types used by this library.

use Value;

use hyper::Error as HyperError;
use xml::reader::Error as XmlError;
use xml::common::TextPosition;

use std::io;
use std::fmt::{self, Formatter, Display};
use std::error::Error;

/// A request could not be executed.
///
/// This is either a lower-level error (for example, the HTTP request failed), or a problem with the
/// server (maybe it's not implementing XML-RPC correctly). If the server sends a valid response,
/// this error will not occur.
#[derive(Debug)]
pub enum RequestError {
    /// An HTTP communication error occurred while sending the request or receiving the response.
    HyperError(HyperError),

    /// The response could not be parsed. This can happen when the server doesn't correctly
    /// implement the XML-RPC spec.
    ParseError(ParseError),

    // TODO make this extensible. anything missing?
}

impl From<HyperError> for RequestError {
    fn from(e: HyperError) -> Self {
        RequestError::HyperError(e)
    }
}

impl From<ParseError> for RequestError {
    fn from(e: ParseError) -> Self {
        RequestError::ParseError(e)
    }
}

impl From<io::Error> for RequestError {
    fn from(e: io::Error) -> Self {
        RequestError::HyperError(HyperError::from(e))
    }
}

impl Display for RequestError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            RequestError::HyperError(ref err) => write!(fmt, "HTTP error: {}", err),
            RequestError::ParseError(ref err) => write!(fmt, "parse error: {}", err),
        }
    }
}

impl Error for RequestError {
    fn description(&self) -> &str {
        match *self {
            RequestError::HyperError(ref err) => err.description(),
            RequestError::ParseError(ref err) => err.description(),
        }
    }
}

/// Describes possible error that can occur when parsing a `Response`.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// Error while parsing (malformed?) XML.
    XmlError(XmlError),

    /// Could not parse the given CDATA as XML-RPC value.
    ///
    /// For example, `<value><int>AAA</int></value>` describes an invalid value.
    InvalidValue(String),

    /// Found an unexpected tag, attribute, etc.
    UnexpectedXml {
        /// A short description of the kind of data that was expected.
        expected: String,
        /// The position of the unexpected data inside the XML document.
        position: TextPosition,
    }
}

impl From<XmlError> for ParseError {
    fn from(e: XmlError) -> Self {
        ParseError::XmlError(e)
    }
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        ParseError::XmlError(XmlError::from(e))
    }
}

impl Display for ParseError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            ParseError::XmlError(ref err) => write!(fmt, "malformed XML: {}", err),
            ParseError::InvalidValue(ref desc) => write!(fmt, "invalid value: {}", desc),
            ParseError::UnexpectedXml {
                ref expected,
                ref position,
            } => {
                write!(fmt, "expected {} at {}", expected, position)
            }
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::XmlError(ref err) => err.description(),
            ParseError::InvalidValue(ref desc) => desc,
            ParseError::UnexpectedXml { .. } => "unexpected XML content",
        }
    }
}

/// A `<fault>` response - The call failed.
///
/// The XML-RPC specification requires that a `<faultCode>` and `<faultString>` is returned in the
/// `<fault>` case, further describing the error.
#[derive(Debug, PartialEq, Eq)]
pub struct Fault {
    pub fault_code: i32,
    pub fault_string: String,
}

impl Fault {
    /// Creates a `Fault` from a `Value`.
    ///
    /// The `Value` must be a `Value::Struct` with a `faultCode` and `faultString` field.
    ///
    /// Returns `None` if the value isn't a valid `Fault`.
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            &Value::Struct(ref map) => {
                match (map.get("faultCode"), map.get("faultString")) {
                    (Some(&Value::Int(fault_code)), Some(&Value::String(ref fault_string))) => {
                        Some(Fault {
                            fault_code: fault_code,
                            fault_string: fault_string.to_string(),
                        })
                    }
                    _ => None
                }
            }
            _ => None
        }
    }
}
