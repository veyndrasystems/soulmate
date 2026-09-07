# Install the binary for your platform

The current source at reviewed snapshot `6911dbb` is an unreleased preview
candidate. Private CI for this snapshot passed native build, package, and
installed-archive checks for Linux x86_64 and macOS on Apple Silicon and Intel;
WSL exercised the Linux artifact. Public `v0.12.1-rc.3` archive retrieval and
installation remain unverified, and this page does not assert that public
release or its assets are available.

Use the pinned installer in the [README](../README.md#install-and-see-the-result).
It selects an archive from the operating system and architecture, checks its
SHA-256, and installs one executable. No Rust, Python, Node.js, or model account
is needed to run the installed binary or `soulmate benchmark`.

| Installed target | Release archive target | Validation boundary |
| --- | --- | --- |
| Linux x86_64 / amd64 | `x86_64-unknown-linux-gnu` | Private native CI passed build, tests, packaged installation and first use; public archive/install verification remains pending. |
| Ubuntu on WSL 2 | `x86_64-unknown-linux-gnu` | Private CI exercised the Linux artifact; keep project, agent host, and Soulmate in the same Ubuntu distribution. |
| macOS on Apple Silicon | `aarch64-apple-darwin` | Private native macOS 15 CI passed build, tests, packaging and installer exercise; public archive/install verification remains pending. |
| macOS on Intel | `x86_64-apple-darwin` | Private native macOS 15 CI passed build, tests, packaging and installer exercise; public archive/install verification remains pending. |

The [CI](../.github/workflows/ci.yml) and
[release workflow](../.github/workflows/release.yml) require both native Mac
architectures. Older macOS versions have not been validated by this matrix.
Cross-compilation alone is not installation evidence.

The candidate packaging path includes an archive checksum and GitHub artifact
provenance check. A future release publisher must recheck transferred archives
and match each separately published executable to its archive payload. These
checks are not Apple Developer ID signing or notarization.

Native Windows and other Linux architectures are unsupported. The installer
rejects unsupported operating-system/architecture pairs before fetching an
archive. The ledger's Unix no-follow file-opening requirement is retained;
[WSL 2](windows-wsl.md) is the supported Windows route. Building from source
does not turn an untested platform into supported installation.

After installation, run `soulmate version` and `soulmate benchmark` before
initializing your project. A checksum or platform error is a failed install;
do not bypass it. See [updates and removal](../REFERENCE.md#removal) for the
explicit managed-skill refresh and the records left by uninstalling.
