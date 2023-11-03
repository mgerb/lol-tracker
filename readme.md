## Database Init

Run for refreshing the database:

```
rm db.sqlite && touch db.sqlite && cargo sqlx prepare
cargo sqlx migrate run
```

## SQLX

```
cargo install sqlx-cli

cargo sqlx prepare

cargo sqlx migrate add <name>

cargo sqlx migrate run
cargo sqlx migrate revert

```

# Roadmap

- fix champion image link
  - remove spaces and single quotes (') from champion names
- notifications
  - active game
    - fix date
    - show current rank/lp
    - show user's image in the "author" url
    - show if promotion/demotion game
    - hide role if "Unknown"
  - completed game
    - show if promoted or demoted
- list all users command
- logs command
- docker
- add autofill discord commands?
- game history command
- 24 hour snapshot command
