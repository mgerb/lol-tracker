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

- list all users command
- logs command
- docker
- add autofill discord commands?
- game history command
- 24 hour snapshot command
