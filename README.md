# Torii ⛩️

Torii is a simple, powerful and extensible open-source Internal Developer Portal where developers can find all the tools and
services they need to build, deploy, and manage their applications.

Torii focus on three principles:

- **Easily configurable**: Platform Engineers can easily define a catalog of tools and services available to developers. They can also
  define a scorecard and a workflow for each tool and service.
- **Easily usable**: Developers can easily find and use the tools and services they need to build, deploy, and manage their applications via
  a simple web interface.
- **Easily extensible**: Platform Engineers can easily extend Torii by adding new tools and services to the catalog.

That's it!

> ⚠️ Torii is in active development and not ready for production yet.
> But contributions are appreciated.

## Features

| Feature          | Status              |
| ---------------- | ------------------- |
| Self Service     | WIP                 |
| Catalog Services | WIP                 |
| Auth             | Not implemented yet |
| Audit            | Not implemented yet |

## Getting Started

### Prerequisites

- MacOSX / Linux / Windows
- Postgres

### Installation

#### Using Docker

Today you can run Torii using Docker Compose. In the future, we will provide a Helm chart to deploy Torii on Kubernetes.

It starts full environment with postgres instance.

```bash
docker-compose up

# Alternatively, you can use tilt
# https://tilt.dev
tilt up
```

#### Locally

```bash
# Start the backend
cd backend
# you need to install Cargo (Rust) if you don't have it
cargo run -- --config examples/config.yaml
```

When starting the backend - you should see the following output:

```bash
████████╗ ██████╗ ██████╗ ██╗██╗
╚══██╔══╝██╔═══██╗██╔══██╗██║██║
   ██║   ██║   ██║██████╔╝██║██║
   ██║   ██║   ██║██╔══██╗██║██║
   ██║   ╚██████╔╝██║  ██║██║██║
   ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚═╝╚═╝

2024-04-14T17:38:07.667049Z  INFO backend: connecting to database...
2024-04-14T17:38:07.756951Z  INFO backend: database initialized and up to date
2024-04-14T17:38:07.757015Z  INFO backend: -> self-service section 'empty-section' loaded
2024-04-14T17:38:07.757021Z  INFO backend: -> self-service section 'default' loaded
2024-04-14T17:38:07.757026Z  INFO backend:      |-> action 'new-testing-environment' loaded
2024-04-14T17:38:07.757033Z  INFO backend:      |-> action 'stop-testing-environment' loaded
2024-04-14T17:38:07.758885Z  INFO backend: Server listening on 0.0.0.0:9999
```

```bash
# Start the frontend
cd frontend
npm install
npm run dev
```

You can now access:

- the frontend at `http://localhost:3000`
- the backend at `http://localhost:9999`

## Configuration

### Simple Configuration

Torii is configured with a YAML file that contains the self-service sections.
A self-service section is a group of actions that developers can use to interact with tools and services.

Here is an example of a simple configuration that will create a self-service section with a single action to create a new testing
environment:

```yaml
self_service:
  sections:
    - slug: default
      name: Default
      description: Default section
      actions:
        - slug: new-testing-environment
          name: New Testing Environment
          description: spin up a temporary testing environment
          icon: target
          fields:
            - slug: name
              title: Name
              description: provide a name for your environment
              placeholder: testing-123
              type: text
              default: testing-123
              required: true
            - slug: description
              title: Description
              description: provide a description for your environment - what's good for?
              type: textarea
            - slug: ttl
              title: TTL
              description: Time to live for your environment (in hours)
              placeholder: 24
              type: number
              required: true
            - slug: seed
              title: Seed
              description: Do you want to seed your environment with some data?
              type: boolean
              default: true
          post_validate:
            - command:
                - python
                - my-scripts/environment_management.py
                - create
                - --name
                - {{ name }} # this is a variable that will be replaced by the value of the field 'name'
          delayed_command:
            - command:
                - python
                - my-scripts/environment_management.py
                - delete
                - --name
                - {{ name }}
              delay:
                hours: {{ ttl }} # this is a variable that will be replaced by the value of the field 'ttl'
```

In this example, we define a self-service section with a single action called `new-testing-environment`. This action has four fields:

- `name`: a text field that is required
- `description`: a textarea field
- `ttl`: a number field that is required
- `seed`: a boolean field with a default value of `true`

When the developer fills the form and submits it, Torii will run the `post_validate` script.
If the script exits with a non-zero exit code, the action will fail.
If the script exits with a zero exit code, Torii will run the `delayed_command` script after the specified delay.

[//]: # "### Advanced Configuration"
[//]: #
[//]: # "#### Autocomplete Fetcher"
[//]: #
[//]: # "An autocomplete fetcher is a script that must print a JSON on standard output. The JSON must contain a `results` key that contains a list of"
[//]: # "values."
[//]: #
[//]: # "```json"
[//]: # "{"
[//]: # '  "results": ['
[//]: # '    "val 1",'
[//]: # '    "val 2",'
[//]: # '    "val 3"'
[//]: # "  ]"
[//]: # "}"
[//]: # "```"
[//]: #
[//]: # "Example of autocomplete fetcher in python:"
[//]: #
[//]: # "```python"
[//]: # "import json"
[//]: #
[//]: #
[//]: # "def get_data_from_fake_api():"
[//]: # "    return ["
[//]: # "        'val 1',"
[//]: # "        'val 2',"
[//]: # "        'val 3',"
[//]: # "    ]"
[//]: #
[//]: #
[//]: # "if __name__ == '__main__':"
[//]: # "    # do your stuff here"
[//]: # "    results = get_data_from_fake_api()"
[//]: #
[//]: # "    data = {'results': results}"
[//]: #
[//]: # "    # print json on standard output"
[//]: # "    print(json.dumps(data))"
[//]: # "```"
[//]: #
[//]: # "#### Validation Script"
[//]: #
[//]: # "A validation script can be any kind of script. It can be a bash script, a python script, a terraform script, etc. The script must exit with"
[//]: # "a non-zero exit code if the validation fails."
[//]: #
[//]: # "```bash"
[//]: # "#!/bin/bash"
[//]: #
[//]: # "set -e # exit on error"
[//]: # "# print error on standard error output"
[//]: #
[//]: # "# do your stuff here"
[//]: # "exit 0"
[//]: # "```"
[//]: #
[//]: # "#### Post Validation Script"
[//]: #
[//]: # "An post validation script can be any kind of script. It can be a bash script, a python script, a terraform script, etc."
[//]: #
[//]: # "- The script must exit with a non-zero exit code if the validation fails."
[//]: # "- The script must be idempotent. It can be executed multiple times without side effects."
[//]: # "- The output of the script must be a JSON that contains the defined model keys with their values. (Torii will update the model with"
[//]: # "  the values returned by the script)"
[//]: #
[//]: # "```json"
[//]: # "{"
[//]: # '  "status": "success",'
[//]: # '  "url": "https://my-service.com",'
[//]: # '  "username": "my-username",'
[//]: # '  "password": "my-password"'
[//]: # "}"
[//]: # "```"
[//]: #
[//]: # "```bash"
[//]: # "#!/bin/bash"
[//]: #
[//]: # "set -e # exit on error"
[//]: # "# print error on standard error output"
[//]: #
[//]: # "# do your stuff here"
[//]: # "exit 0"
[//]: # "```"

## Design

Please refer to the [DESIGN.md](./DESIGN.md) file for more information.

## Contributing

Torii is still in early development. If you want to contribute, please open an issue or a pull request. We will improve the
contribution guidelines as soon as possible.

### Linting

Use the rust formatter for linting the code

```bash
cargo fmt --all
```

## Motivation

Today you have the choice between three options to build your Internal Developer Portal:

1. Use [Backstage](https://backstage.io) from Spotify. It's a great solution, but it's hard to extend and customize. You need to be a
   TypeScript/React expert to extend it.
2. Use a proprietary SaaS solution like Port, Cortex, OpsLevel, etc. It's a great solution, but it's expensive and you don't have control
   over the codebase.
3. Build your own solution from scratch. It's a great solution, but it's hard to maintain and scale. You need to be a full-stack developer
   to build it.

Torii is a simple, powerful, and extensible open-source Internal Developer Portal that aims to be the best of all worlds. It's easy to
extend and customize, it's free, and you have control over the codebase.

---

Curious to understand in more detail the motivation behind Torii? Read these articles:

- [Why Backstage is So Complex? Is There An Alternative?](https://romaricphilogene.substack.com/p/platform-tips-20-why-backstage-is)
- [What Could Be A Better Alternative To Backstage?](https://romaricphilogene.substack.com/p/platform-tips-21-what-could-be-a)

Curious to understand Torii is built? Read these articles:

- [Building a Better Alternative To Backstage - Part I](https://romaricphilogene.substack.com/p/platform-tips-22-building-a-better)
- Building a Better Alternative To Backstage - Part II (coming soon)
- Building a Better Alternative To Backstage - Part III (coming soon)

## FAQ

### What's the difference between Torii and Qovery?

The Qovery and Torii projects are two different projects with different goals:

- [Qovery](https://www.qovery.com) is an Internal Developer **Platform** focusing on the Software Development Lifecycle (build, deploy, run).
- Torii is an Internal Developer **Portal** focusing on unifying the experience of all engineering tools.

Here is a features table to help you understand the difference:

| Feature               | Qovery (Internal Developer Platform) | Torii (Internal Developer Portal) |
| --------------------- | ------------------------------------ | --------------------------------- |
| Build                 | ✅                                   | ❌                                |
| Deploy                | ✅                                   | ❌                                |
| Run                   | ✅                                   | ❌                                |
| Ephemeral Environment | ✅                                   | ❌                                |
| Self-Service          | ✅                                   | ✅                                |
| Catalogs Service      | ❌                                   | ✅                                |
| Scorecard Service     | ❌                                   | ✅                                |
| Workflow Service      | Partial with the concept of Pipeline | ✅                                |

### Why Torii?

Torii is a [Japanese gate](https://en.wikipedia.org/wiki/Torii) most commonly found at the entrance of or within a Shinto shrine, where it symbolically marks the transition from the mundane to the sacred.
