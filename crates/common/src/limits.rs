//! Size limits shared by the server and the judge. The judge must never
//! store more than these bounds, and the server must never send more.
//! Sizes are counted in bytes of JSON text, after escaping.

/// The largest JSON payload, in bytes, of one public run event.
pub const MAX_PAYLOAD_BYTES: usize = 16 * 1024;

/// The largest compile output, in bytes of JSON text, kept in a `done` event.
pub const MAX_COMPILE_OUTPUT_BYTES: usize = 8 * 1024;

/// The largest size, in bytes of JSON text, of each sample input, expected
/// value and actual value shown in a sample test event.
pub const MAX_SAMPLE_FIELD_BYTES: usize = 2 * 1024;
