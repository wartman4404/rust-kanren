#![feature(get_type_id)]
#![feature(raw)] // used by VarWrapper::get_wrapped_value().

extern crate ref_slice;

#[macro_use]
pub mod macros;
///! Contains `State`, which performs unification, and `Var`, the variable type it operates on.
#[macro_use]
pub mod core;
///! Contains `List`, a singly-linked list of variables.
pub mod list;
///! Contains iterators for combining `State`s.
pub mod iter;
///! Contains definitions of commonly used methods.  There's not much here, yet.
pub mod builtins;
///! Contains `Fd`, which represents a finite-domain value.
pub mod finitedomain;
///! Contains a number of built-in constraints.
pub mod constraints;
