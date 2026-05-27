# `__COMPUTED_FQN__`

__DESCRIPTION__

## Input

**`placeholder`: [`String`]**

TODO: describe the real input port(s) once the schema is finalized.

## Output Variants & Ports

**`ok`**

TODO: describe the success variant.

- **`ok.result`: [`String`]** — placeholder.

**`err_upstream`**

Returned when the upstream service fails.

- **`err_upstream.reason`: [`String`]** — human-readable reason.

**`err_config`**

Returned when a required env var is missing or invalid.

- **`err_config.reason`: [`String`]** — human-readable reason.
