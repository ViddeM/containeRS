rm -r registry
docker compose exec -i db psql -U registry-rs registry-rs -c "DROP SCHEMA public CASCADE;"
docker compose exec -i db psql -U registry-rs registry-rs -c "CREATE SCHEMA public;"
cargo sqlx migrate run
