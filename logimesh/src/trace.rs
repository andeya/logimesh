// Copyright Andeya Lee 2024
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.
//! Provides building blocks for tracing distributed programs.
//!
//! A trace is logically a tree of causally-related events called spans. Traces are tracked via a
//! [context](Context) that identifies the current trace, span, and parent of the current span.  In
//! distributed systems, a context can be sent from client to server to connect events occurring on
//! either side.
//!
//! This crate's design is based on [opencensus
//! tracing](https://opencensus.io/core-concepts/tracing/).
pub use ::tarpc::trace;
