//! Catalogue read path for the server. The server reads only public catalogue tables through its
//! own role (CAT-R3, CAT-A9).

pub mod dto;
pub mod handlers;
pub mod lookup;
pub mod read;
