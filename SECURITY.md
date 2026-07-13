# Security Policy

## Project maturity

Cage is pre-alpha, experimental, and not production ready. There are no
supported releases or published packages. The project must not be relied on as
a security boundary.

## Supported versions

| Version | Supported |
|---|---|
| Published releases | No releases exist |
| `main` | Security reports are accepted for current code |

Because Cage has no release yet, fixes are made only on the current development
line. This table will be replaced with version-specific support information
before the first release.

## Reporting a vulnerability

Do not report suspected vulnerabilities in a public issue, discussion, or pull
request.

Use GitHub's private vulnerability reporting form:

<https://github.com/Saber5656/cage/security/advisories/new>

Include the affected revision, environment, reproduction steps, expected
impact, and any suggested mitigation. Avoid including real credentials,
personal data, or other secrets in the report.

If private vulnerability reporting is unavailable, do not publish exploit
details. Open a public issue that contains no sensitive technical details and
asks the maintainer to establish a private contact channel.

## Response expectations

There is no guaranteed response or remediation SLA during pre-alpha. The
maintainer will make a best effort to acknowledge and assess private reports.
Security fixes may require coordinated disclosure; please allow time for a fix
and regression tests before publishing details.

## Scope

Useful reports include sandbox escapes, unsafe host mounts, command injection,
credential exposure, privilege escalation, and bypasses of intended resource
or security controls. Reports about missing pre-alpha features without a
security impact belong in the public issue tracker.
