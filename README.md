# actix-crud

## Installation
```rust
#Step Run 01
1 docker-compose up -d <= for start postgres
2 cargo watch -x run  <= for run actix in local development


#Step Run 02
1 docker-compose -f docker-compose.yml -f docker-compose.dev.yml up  <= run everything in docker

#In .env change 
Local Step Run 01
DATABASE_URL=postgres://admin:admin@localhost:5432/actix_crud
Docker Step Run 02