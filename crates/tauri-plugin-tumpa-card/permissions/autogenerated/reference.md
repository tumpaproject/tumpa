## Default Permission

Default permission set for the tumpa-card plugin.

Allows beginning a card session (NFC or USB), transmitting APDUs,
ending the session, and reading / writing biometric-gated secrets
(card PINs and on-disk key passphrases) in the platform keyring.

#### This default permission set includes the following:

- `allow-begin-session`
- `allow-transmit-apdu`
- `allow-end-session`
- `allow-save-secret`
- `allow-read-secret`
- `allow-clear-secret`
- `allow-clear-all-secrets`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`tumpa-card:allow-begin-session`

</td>
<td>

Enables the begin_session command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-begin-session`

</td>
<td>

Denies the begin_session command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-clear-all-secrets`

</td>
<td>

Enables the clear_all_secrets command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-clear-all-secrets`

</td>
<td>

Denies the clear_all_secrets command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-clear-secret`

</td>
<td>

Enables the clear_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-clear-secret`

</td>
<td>

Denies the clear_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-end-session`

</td>
<td>

Enables the end_session command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-end-session`

</td>
<td>

Denies the end_session command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-read-secret`

</td>
<td>

Enables the read_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-read-secret`

</td>
<td>

Denies the read_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-save-secret`

</td>
<td>

Enables the save_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-save-secret`

</td>
<td>

Denies the save_secret command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:allow-transmit-apdu`

</td>
<td>

Enables the transmit_apdu command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`tumpa-card:deny-transmit-apdu`

</td>
<td>

Denies the transmit_apdu command without any pre-configured scope.

</td>
</tr>
</table>
