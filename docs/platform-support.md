# Install the binary for your platform

The public prerelease `v0.12.1-rc.3` is built from reviewed source snapshot
`4199ebd`. Its public release workflow validated native builds, packaged
installation, and public asset installation for Linux x86_64 and macOS on
Apple Silicon and Intel; WSL exercises the same Linux artifact inside Ubuntu.
See the [public release workflow](https://github.com/veyndrasystems/soulmate/actions/runs/34094355184).

Use the pinned installer in the [README](../README.md#install-and-see-the-result).
It selects an archive from the operating system and architecture, checks its
SHA-256, and installs one executable. No Rust, Python, Node.js, or model account
is needed to run the installed binary or `soulmate benchmark`.

| Installed target | Release archive target | Validation boundary |
| --- | --- | --- |
| Linux x86_64 / amd64 | `x86_64-unknown-linux-gnu` | Native build, packaged installation and public asset installation passed in the public release workflow. |
| Ubuntu on WSL 2 | `x86_64-unknown-linux-gnu` | Exercises the same Linux artifact inside Ubuntu; keep project, agent host, and Soulmate in the same Ubuntu distribution. |
| macOS on Apple Silicon | `aarch64-apple-darwin` | Native macOS 15 build, packaged installation and public asset installation passed in the public release workflow. |
| macOS on Intel | `x86_64-apple-darwin` | Native macOS 15 build, packaged installation and public asset installation passed in the public release workflow. |

The [CI](../.github/workflows/ci.yml) and
[release workflow](../.github/workflows/release.yml) require both native Mac
architectures. Older macOS versions have not been validated by this matrix.
Cross-compilation alone is not installation evidence.

The public release workflow rechecks transferred archive checksums and
provenance, then matches each separately published executable to the executable
inside its archive. These checks are not Apple Developer ID signing or
notarization.

Native Windows and other Linux architectures are unsupported. The installer
rejects unsupported operating-system/architecture pairs before fetching an
archive. The ledger's Unix no-follow file-opening requirement is retained;
[WSL 2](windows-wsl.md) is the supported Windows route. Building from source
does not turn an untested platform into supported installation.

After installation, run `soulmate version` and `soulmate benchmark` before
initializing your project. A checksum or platform error is a failed install;
do not bypass it. See [updates and removal](../REFERENCE.md#removal) for the
explicit managed-skill refresh and the records left by uninstalling.
