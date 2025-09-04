/* examples/demo.rs */

use real::{IpExtractor, extract_real_ip, extract_real_ip_strict};
use std::collections::HashMap;

fn main() {
    println!("=== Real IP Extraction Examples ===\n");

    // Example 1: Basic X-Real-IP header
    example_1_x_real_ip();

    // Example 2: X-Forwarded-For with multiple IPs
    example_2_x_forwarded_for();

    // Example 3: Cloudflare CF-Connecting-IP
    example_3_cf_connecting_ip();

    // Example 4: Multiple headers with priority
    example_4_header_priority();

    // Example 5: Fallback to remote address
    example_5_fallback();

    // Example 6: Strict mode (no private IPs)
    example_6_strict_mode();

    // Example 7: Custom extractor configuration
    example_7_custom_extractor();

    println!("=== All examples completed! ===");
}

fn example_1_x_real_ip() {
    println!("Example 1: Basic X-Real-IP header");

    let mut headers = HashMap::new();
    headers.insert("x-real-ip".to_string(), "203.0.113.45".to_string());

    match extract_real_ip(&headers, None) {
        Some(ip) => println!("Extracted IP: {}", ip),
        None => println!("No IP found"),
    }
    println!();
}

fn example_2_x_forwarded_for() {
    println!("Example 2: X-Forwarded-For with multiple IPs");

    let mut headers = HashMap::new();
    headers.insert(
        "x-forwarded-for".to_string(),
        "203.0.113.1, 192.168.1.10, 10.0.0.5".to_string(),
    );

    match extract_real_ip(&headers, None) {
        Some(ip) => println!("Extracted IP (first in chain): {}", ip),
        None => println!("No IP found"),
    }

    // Show what happens when we want the last IP
    let extractor = IpExtractor::new()
        .use_first_forwarded(false)
        .trust_private_ips(true);
    match extractor.extract(&headers, None) {
        Some(ip) => println!("Extracted IP (last in chain): {}", ip),
        None => println!("No IP found"),
    }
    println!();
}

fn example_3_cf_connecting_ip() {
    println!("Example 3: Cloudflare CF-Connecting-IP");

    let mut headers = HashMap::new();
    headers.insert("cf-connecting-ip".to_string(), "198.51.100.42".to_string());
    headers.insert("x-forwarded-for".to_string(), "203.0.113.1".to_string());

    match extract_real_ip(&headers, None) {
        Some(ip) => println!("Extracted IP (CF has higher priority): {}", ip),
        None => println!("No IP found"),
    }
    println!();
}

fn example_4_header_priority() {
    println!("Example 4: Multiple headers with priority");

    let mut headers = HashMap::new();
    headers.insert("x-real-ip".to_string(), "203.0.113.100".to_string());
    headers.insert("cf-connecting-ip".to_string(), "198.51.100.200".to_string());
    headers.insert("x-forwarded-for".to_string(), "192.0.2.50".to_string());

    println!("Headers present:");
    for (key, value) in &headers {
        println!("  {}: {}", key, value);
    }

    match extract_real_ip(&headers, None) {
        Some(ip) => println!("Extracted IP (x-real-ip has highest priority): {}", ip),
        None => println!("No IP found"),
    }
    println!();
}

fn example_5_fallback() {
    println!("Example 5: Fallback to remote address");

    let headers = HashMap::new(); // Empty headers
    let remote_addr = "192.0.2.123";

    match extract_real_ip(&headers, Some(remote_addr.to_string())) {
        Some(ip) => println!("Using fallback IP: {}", ip),
        None => println!("No IP found"),
    }
    println!();
}

fn example_6_strict_mode() {
    println!("Example 6: Strict mode (rejects private IPs from headers)");

    let mut headers = HashMap::new();
    headers.insert("x-real-ip".to_string(), "192.168.1.100".to_string()); // Private IP
    let fallback = "203.0.113.50"; // Public IP

    println!("Header contains private IP: 192.168.1.100");
    println!("Fallback public IP: {}", fallback);

    // Regular mode (trusts private IPs)
    match extract_real_ip(&headers, Some(fallback.to_string())) {
        Some(ip) => println!("Regular mode result: {}", ip),
        None => println!("Regular mode: No IP found"),
    }

    // Strict mode (rejects private IPs from headers)
    match extract_real_ip_strict(&headers, Some(fallback.to_string())) {
        Some(ip) => println!("Strict mode result: {}", ip),
        None => println!("Strict mode: No IP found"),
    }
    println!();
}

fn example_7_custom_extractor() {
    println!("Example 7: Custom extractor configuration");

    let mut headers = HashMap::new();
    headers.insert("custom-real-ip".to_string(), "203.0.113.200".to_string());
    headers.insert("x-real-ip".to_string(), "192.168.1.50".to_string());

    // Custom extractor with different header priority
    let custom_extractor = IpExtractor::new()
        .with_headers(vec!["custom-real-ip".to_string(), "x-real-ip".to_string()])
        .trust_private_ips(true);

    match custom_extractor.extract(&headers, None) {
        Some(ip) => println!("Custom extractor result: {}", ip),
        None => println!("Custom extractor: No IP found"),
    }

    // Show what the default extractor would find
    let default_extractor = IpExtractor::default().trust_private_ips(true);
    match default_extractor.extract(&headers, None) {
        Some(ip) => println!("Default extractor result: {}", ip),
        None => println!("Default extractor: No IP found"),
    }
    println!();
}
