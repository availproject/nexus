# To setup DB

```shell
# To setup db :

# 1. Run postgres on docker

docker run --name my-postgres \
  -e POSTGRES_USER=user \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=db_name \
  -p 5432:5432 \
  -d postgres:17

export DATABASE_URL=postgres://user:password@localhost:5432/db_name

# 2. Run migration script

cd host/
sqlx migrate run

# 3. Prepare offline mode builds when database is not available
cd core/
cargo sqlx prepare -- --lib --features native
```