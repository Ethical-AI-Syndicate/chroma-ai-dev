pub fn sanitize_terminal_output(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != 0x1b {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        if i + 1 >= bytes.len() {
            i += 1;
            continue;
        }

        match bytes[i + 1] {
            b']' => {
                i = skip_osc(bytes, i + 2);
            }
            b'[' => {
                if let Some((end, keep)) = parse_csi(bytes, i + 2) {
                    if keep {
                        out.extend_from_slice(&bytes[i..=end]);
                    }
                    i = end + 1;
                } else {
                    i += 1;
                }
            }
            _ => {
                i += 2;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

fn skip_osc(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() {
        if bytes[i] == 0x07 {
            return i + 1;
        }
        if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

fn parse_csi(bytes: &[u8], mut i: usize) -> Option<(usize, bool)> {
    while i < bytes.len() {
        let b = bytes[i];
        if (b'@'..=b'~').contains(&b) {
            if b != b'm' {
                return Some((i, false));
            }

            let params = String::from_utf8_lossy(&bytes[..=i]);
            let keep = params
                .split('[')
                .next_back()
                .unwrap_or_default()
                .trim_end_matches('m')
                .split(';')
                .filter(|part| !part.is_empty())
                .all(is_safe_sgr_code);

            return Some((i, keep));
        }

        if !(b.is_ascii_digit() || b == b';') {
            return Some((i, false));
        }
        i += 1;
    }

    None
}

fn is_safe_sgr_code(part: &str) -> bool {
    if let Ok(code) = part.parse::<u16>() {
        return matches!(
            code,
            0 | 1 | 2 | 3 | 4 | 5 | 7 | 8 | 9 | 30..=37 | 39 | 40..=47 | 49 | 90..=97 | 100..=107
        );
    }
    false
}
