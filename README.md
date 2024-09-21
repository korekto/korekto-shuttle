# korekto-shuttle

Primary instance is available at https://korekto.shuttleapp.rs

## Use Just

Install just : https://github.com/casey/just?tab=readme-ov-file#installation

Then enter `just` to see the list of available recipes.

## Configuration

The app requires:

* A `public` GitHub app (1) that users will install to grant permissions for reading their repositories.  
  This GitHub app (1) also allows for the reception of webhook events to follow what is happening on the users side (
  push of new code for example).  
  This GitHub app (1) should have the following permissions:
    * Repository > Actions: Read-only
    * Repository > Contents: Read-only
    * Repository > Issues: Read and write
    * Repository > Metadata: Read-only
* A more confidential GitHub app (2) installed on the runner repository to grant permission to trigger workflow runs.  
  This GitHub app (1) should have the following permissions:
    * Repository > Actions: Read and write
    * Repository > Contents: Read-only
    * Repository > Metadata: Read-only

The app in itself have the following configuration parameters:

| Name                                |                                     | Description                                                                               | Example                               |
|-------------------------------------|-------------------------------------|-------------------------------------------------------------------------------------------|---------------------------------------|
| BASE_URL                            | Required                            | Used to compute callback urls (such as the one for oauth2 web flow)                       | http://localhost:8000                 |
| FIRST_ADMIN                         | Optional                            | Provider login of the user to set admin on first connection                               | http://localhost:8000                 |
| COOKIE_SECRET_KEY                   | Optional (defaults to a random one) | Use to cypher private cookies                                                             |                                       |
| GITHUB_APP_ID                       | Required                            | ID of the GitHub app (1)                                                                  | 12345                                 |
| GITHUB_APP_NAME                     | Required                            | Name of the GitHub app (1)                                                                | Korekto                               |
| GITHUB_APP_CLIENT_ID                | Required                            | Client ID of the GitHub app (1)                                                           | Abd.YTGB4541hj                        |
| GITHUB_APP_CLIENT_SECRET            | Required                            | Client Secret of the GitHub app (1)                                                       | ad45f12ccb5687                        |
| GITHUB_APP_PRIVATE_KEY              | Required                            | Private Key of the GitHub app (1)                                                         | -----BEGIN RSA PRIVATE KEY----- (...) |
| GITHUB_APP_WEBHOOK_SECRET           | Required                            | Secret needed to verify GitHub app (1) webhooks origin                                    | ad45f12ccb5687                        |
| GITHUB_RUNNER_APP_ID                | Required                            | ID of the GitHub app (2)                                                                  | 12345                                 |
| GITHUB_RUNNER_APP_PRIVATE_KEY       | Required                            | Private Key of the GitHub app (2)                                                         | -----BEGIN RSA PRIVATE KEY----- (...) |
| GITHUB_RUNNER_REPO_SLUG             | Required                            | GitHub repository slug hosting the grading job                                            | some_org/some_repo                    |
| GITHUB_RUNNER_INSTALLATION_ID       | Required                            | Installation ID of the GitHub app (2) on the runner repository                            | 12345678                              |
| GITHUB_RUNNER_CALLBACK_URL_OVERRIDE | Optional (defaults to $BASE_URL)    | Used to compute callback urls for runner jobs                                             | https://smee.io/machin                |
| GITHUB_RUNNER_WORKFLOW_ID           | Optional (defaults to grade.yml)    | Name of the workflow to trigger for grading a user assignment                             | something.yml                         |
| GITHUB_CLIENT_CACHE_SIZE            | Optional (defaults to 50)           | Size of the LRU cache hosting GitHub client instances                                     | 20                                    |
| SCHEDULER_INTERVAL_IN_SECS          | Optional (defaults to 15)           | Interval between scheduler jobs                                                           | 20                                    |
| MIN_GRADING_INTERVAL_IN_SECS        | Optional (defaults to 20 * 60)      | Minimum interval between two gradings of the same assignment of the same user             | 1800                                  |      
| GRADING_ORDERED_TIMEOUT_IN_SECS     | Optional (defaults to 5 * 60)       | Duration after which an `ORDERED` grading job with no received `STARTED` event times out  | 180                                   |
| GRADING_STARTED_TIMEOUT_IN_SECS     | Optional (defaults to 15 * 60)      | Duration after which a `STARTED` grading job with no received `COMPLETED` event times out | 600                                   |
| MAX_PARALLEL_GRADINGS               | Optional (defaults to 3)            | Maximum parallel grading jobs running in the Github runner                                |                                       |

## Configuration of the GitHub runner

// TODO

## Run it locally

This is a Rust project, using Docker for the PostgresSQL instance, and Shuttle as IFC environment.

* Install Rust and its ecosystem if you have not already
    * `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` cf https://rustup.rs/
* Install Docker
* Install Node (needed for building the frontend & using [smee](https://smee.io/))
    * `curl https://get.volta.sh | bash` cf https://docs.volta.sh/guide/getting-started
    * `volta install node@20`
* Install Just
    * `cargo install just` cf https://just.systems/man/en/chapter_4.html
* Install Shuttle
    * `cargo install cargo-shuttle` cf https://docs.shuttle.rs/getting-started/installation

* Clone and build
    * `git clone git@github.com:korekto/korekto-shuttle.git`
    * `git clone git@github.com:korekto/korekto-frontend.git`
    * `cd korekto-shuttle`
    * `./local_front.sh`

* Create and fill the `Secrets.toml` file with expected [configuration](#Configuration) parameters,
  see [docs](https://docs.shuttle.rs/resources/shuttle-secrets)
* Start smee for GitHub and runner webhooks
    * `just install-smee`
    * `just start-smee-gh`
    * `just start-smee-runner`

* Start the app
    * `clear && just run`

## Deploy your own instance

* Create an account on [shuttle](https://www.shuttle.rs/)
* Create a file `Shuttle.toml` with the content:

```toml
name = "my-project-name"
```

```bash
# Create the project in your Shuttle account
cargo shuttle init --name "my-project-name" --create-env --no-git
# Deploy
cargo shuttle deploy
```

Check the running service at https://my-project-name.shuttleapp.rs


## As a teacher

Get an installation token for a user, then
```bash
git clone https://x-access-token:TOKEN@github.com/owner/repo.git repo_owner
```
