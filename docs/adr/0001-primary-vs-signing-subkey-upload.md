# ADR-0001: Primary Key vs Signing Subkey Upload to Smartcard

## Status

Accepted

## Context

When uploading keys to an OpenPGP smartcard, the card has a single signing slot. An OpenPGP key can have two sources of signing capability:

1. **Primary key** — the certification key itself, which can also have signing capability (`can_primary_sign`)
2. **Signing subkey** — a dedicated subkey created specifically for signing operations

The original Tumpa (PySide version) and tugpgp both uploaded the primary key to the signing slot unconditionally. However, some keys have both a signing-capable primary and a separate signing subkey, and users should be able to choose which one goes on the card.

## Decision

The upload dialog presents the signing slot as a separate section with two mutually exclusive options:

- **Primary key (signing)** — uploads the primary key to the signing slot
- **Signing subkey** — uploads the dedicated signing subkey to the signing slot

Selection logic:
- If only the primary key can sign (no signing subkey): primary is preselected, signing subkey is greyed out
- If only a signing subkey exists (primary cannot sign): signing subkey is preselected, primary is greyed out
- If both exist: signing subkey is preselected by default, primary is deselected but selectable
- User can select one or the other, never both

Mutual exclusion is enforced in both the frontend (Vue watchers) and backend (bitmask validation rejects bits 2 and 8 set simultaneously).

## Backend Bitmask

The `upload_key_to_card` command uses a bitmask for `which_subkeys`:

| Bit | Value | Meaning |
|-----|-------|---------|
| 0   | 1     | Encryption subkey → Decryption slot |
| 1   | 2     | Primary key → Signing slot |
| 2   | 4     | Authentication subkey → Authentication slot |
| 3   | 8     | Signing subkey → Signing slot |

Bits 1 and 3 are mutually exclusive.

## Consequences

- Users with both primary signing capability and a signing subkey can make an informed choice
- Default behavior (signing subkey preferred) matches the OpenPGP best practice of keeping the primary key offline
- The `SubkeyAvailability` struct now reports `primary_can_sign` and `signing_subkey` as separate fields instead of a single `signing` boolean
