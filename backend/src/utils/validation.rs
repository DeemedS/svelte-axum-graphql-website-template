use regex::Regex;

/// Validates an email format.
pub fn validate_email(email: &str) -> Result<(), String> {
    if email.trim().is_empty() {
        return Err("Email is required".into());
    }

    let email_regex = Regex::new(r"^\S+@\S+\.\S+$").unwrap();
    if !email_regex.is_match(email) {
        return Err("Invalid email address".into());
    }

    Ok(())
}

/// Validates a password based on complexity rules.
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".into());
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter".into());
    }

    if !password.chars().any(|c| "!@#$%^&*(),.?\":{}|<>".contains(c)) {
        return Err("Password must contain at least one special character".into());
    }

    Ok(())
}
