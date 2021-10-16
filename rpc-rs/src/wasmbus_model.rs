// This file is generated automatically using wasmcloud/weld-codegen and smithy model definitions
//

#![allow(dead_code, clippy::needless_lifetimes)]
use serde::{Deserialize, Serialize};

pub const SMITHY_VERSION: &str = "1.0";

/// Capability contract id, e.g. 'wasmcloud:httpserver'
pub type CapabilityContractId = String;

/// signed 16-bit int
pub type I16 = i16;

/// signed 32-bit int
pub type I32 = i32;

/// signed 64-bit int
pub type I64 = i64;

/// signed byte
pub type I8 = i8;

/// list of identifiers
pub type IdentifierList = Vec<String>;

/// unsigned 16-bit int
pub type U16 = i16;

/// unsigned 32-bit int
pub type U32 = i32;

/// unsigned 64-bit int
pub type U64 = i64;

/// unsigned byte
pub type U8 = i8;

/// Rust codegen traits
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodegenRust {
    /// if true, disables deriving 'Default' trait
    #[serde(rename = "noDeriveDefault")]
    #[serde(default)]
    pub no_derive_default: bool,
    /// if true, disables deriving 'Eq' trait
    #[serde(rename = "noDeriveEq")]
    #[serde(default)]
    pub no_derive_eq: bool,
}

/// indicates that a trait or class extends one or more bases
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Extends {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<IdentifierList>,
}

/// A non-empty string (minimum length 1)
pub type NonEmptyString = String;

/// Overrides for serializer & deserializer
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Serialization {
    /// (optional setting) Override field name when serializing and deserializing
    /// By default, (when `name` not specified) is the exact declared name without
    /// casing transformations. This setting does not affect the field name
    /// produced in code generation, which is always lanaguage-idiomatic
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// This trait doesn't have any functional impact on codegen. It is simply
/// to document that the defined type is a synonym, and to silence
/// the default validator that prints a notice for synonyms with no traits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Synonym {}

/// The unsignedInt trait indicates that one of the number types is unsigned
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UnsignedInt {}

/// a protocol defines the semantics
/// of how a client and server communicate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Wasmbus {
    /// indicates this service's operations are handled by an actor (default false)
    #[serde(rename = "actorReceive")]
    #[serde(default)]
    pub actor_receive: bool,
    /// capability id such as "wasmcloud:httpserver"
    /// always required for providerReceive, but optional for actorReceive
    #[serde(rename = "contractId")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_id: Option<CapabilityContractId>,
    /// indicates this service's operations are handled by an provider (default false)
    #[serde(rename = "providerReceive")]
    #[serde(default)]
    pub provider_receive: bool,
}

/// data sent via wasmbus
/// This trait is required for all messages sent via wasmbus
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WasmbusData {}
