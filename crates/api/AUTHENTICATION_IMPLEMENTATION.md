# Authentication Implementation

## Overview

This document describes the implementation of Task 21: Discrete Login and Registration for the Crypto Trading Platform Enhancements.

## Requirements Implemented

### Requirement 17.1, 17.2, 17.3: Minimal Registration Fields
- Registration requires only: email, password, and one wallet address
- No collection of phone numbers, physical addresses, or real names
- Email format validation (must have @ and . with characters before and after @)
- Password strength validation (minimum 8 characters, uppercase, lowercase, digit)

### Requirement 17.4: Secure Password Hashing
- Implemented Argon2 password hashing algorithm
- Uses secure salt generation with OsRng
- Password verification with constant-time comparison

### Requirement 17.5: Optional 2FA Support
- TOTP-based 2FA implementation
- QR code generation for easy setup with authenticator apps
- Secret generation using secure random bytes
- Code verification with 30-second time window and ±1 step tolerance for clock skew
- Enable/disable 2FA with password verification

### Requirement 17.6: JWT Authentication
- JWT token generation with 24-hour expiration
- Token verification with signature validation
- User ID stored in token claims

### Requirements 20.1, 20.2: User Tag Generation
- Automatic generation of unique user tags on registration
- Format: "Trader_" + 6 random alphanumeric characters
- Uniqueness validation with retry mechanism (max 10 attempts)

## Database Schema Changes

### Migration 20240101000017: User Tag Support
```sql
ALTER TABLE users ADD COLUMN user_tag VARCHAR(50) UNIQUE;
ALTER TABLE users ADD COLUMN show_email_publicly BOOLEAN DEFAULT FALSE;
CREATE INDEX idx_users_user_tag ON users(user_tag);
```

### Migration 20240101000018: 2FA Support
```sql
ALTER TABLE users ADD COLUMN totp_secret VARCHAR(255);
ALTER TABLE users ADD COLUMN totp_enabled BOOLEAN DEFAULT FALSE;
ALTER TABLE users ADD COLUMN totp_verified_at TIMESTAMP;
CREATE INDEX idx_users_totp_enabled ON users(totp_enabled);
```

## API Endpoints

### User Registration
- **POST** `/api/users/register`
- **Body**: `{ email, password, wallet_address }`
- **Response**: User object with generated user_tag
- **Status Codes**:
  - 201: Created successfully
  - 400: Invalid email or weak password
  - 409: User already exists

### User Login
- **POST** `/api/users/login`
- **Body**: `{ email, password }`
- **Response**: `{ user, token }`
- **Status Codes**:
  - 200: Login successful
  - 401: Invalid credentials

### Enable 2FA
- **POST** `/api/users/2fa/enable`
- **Body**: `{ user_id }`
- **Response**: `{ secret, qr_code }` (SVG QR code)
- **Status Codes**:
  - 200: 2FA setup initiated
  - 400: Invalid user_id format

### Verify and Enable 2FA
- **POST** `/api/users/2fa/verify`
- **Body**: `{ user_id, code }`
- **Response**: Success message
- **Status Codes**:
  - 200: 2FA enabled successfully
  - 401: Invalid code

### Disable 2FA
- **POST** `/api/users/2fa/disable`
- **Body**: `{ user_id, password }`
- **Response**: Success message
- **Status Codes**:
  - 200: 2FA disabled successfully
  - 401: Invalid password

## Implementation Details

### Password Validation Rules
- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one digit

### Email Validation Rules
- Minimum 5 characters
- Must contain @ symbol with at least one character before it
- Must contain . in domain part after @

### JWT Token Structure
```json
{
  "sub": "user-uuid",
  "exp": 1234567890
}
```

### User Tag Format
- Prefix: "Trader_"
- Suffix: 6 random characters from [A-Z0-9]
- Example: "Trader_A7X9K2"

### TOTP Configuration
- Algorithm: SHA1
- Time step: 30 seconds
- Code length: 6 digits
- Clock skew tolerance: ±1 time step (±30 seconds)

## Security Considerations

1. **Password Storage**: Passwords are never stored in plain text, only Argon2 hashes
2. **JWT Secret**: Must be configured via JWT_SECRET environment variable
3. **2FA Secret**: Stored encrypted in database, only accessible to authenticated user
4. **Rate Limiting**: Should be implemented at API gateway level for login attempts
5. **HTTPS**: All authentication endpoints should be served over HTTPS in production

## Testing

### Unit Tests
- Email validation
- Password strength validation
- Password hashing and verification
- JWT token generation and verification
- User tag generation format

### Integration Tests
- Complete authentication flow
- JWT token expiration validation
- Error type handling

## Dependencies Added

```toml
argon2 = "0.5"           # Password hashing
totp-lite = "2.0"        # TOTP implementation
base32 = "0.4"           # Base32 encoding for TOTP secrets
qrcode = "0.14"          # QR code generation
```

## Environment Variables Required

```bash
JWT_SECRET=your-secret-key-here
JWT_EXPIRATION_HOURS=24  # Optional, defaults to 24
```

## Future Enhancements

1. Refresh token mechanism for long-lived sessions
2. Password reset functionality
3. Email verification on registration
4. Account lockout after failed login attempts
5. Session management and revocation
6. OAuth2/OpenID Connect integration
7. Biometric authentication support
8. Hardware security key (WebAuthn) support

## Privacy Features

- User tags provide anonymity in public contexts
- Email addresses are never displayed publicly unless user opts in
- No collection of personal information beyond email
- Wallet addresses stored separately with user consent
