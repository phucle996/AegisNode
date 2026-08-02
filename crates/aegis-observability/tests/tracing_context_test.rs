// Integration Test cho W3C Distributed Tracing (Phase 26 Production Observability)

use aegis_observability::W3cTraceContext;

#[test]
fn test_w3c_traceparent_format_and_parse() {
    // 1. Sinh ngẫu nhiên Trace Context
    let ctx = W3cTraceContext::generate_new();

    assert_eq!(ctx.version, "00");
    assert_eq!(ctx.trace_id.len(), 32);
    assert_eq!(ctx.parent_id.len(), 16);
    assert_eq!(ctx.trace_flags, "01");

    // 2. Format sang header string
    let header = ctx.format_traceparent();
    assert!(header.starts_with("00-"));

    // 3. Parse lại từ header string -> Phải trả về dữ liệu nguyên vẹn
    let parsed = W3cTraceContext::parse_traceparent(&header).expect("Parse W3C traceparent thất bại");
    assert_eq!(parsed, ctx);
}

#[test]
fn test_w3c_traceparent_invalid_header_rejection() {
    // Header sai định dạng -> Phải bị từ chối với lỗi Validation
    assert!(W3cTraceContext::parse_traceparent("invalid-header").is_err());
    assert!(W3cTraceContext::parse_traceparent("01-traceid-parentid-01").is_err());
}
