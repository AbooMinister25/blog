# My Blog

Written in Rust, and powered by Rocket and Svelte.

**Unfinished, in active development**

## Cloning and building

The backend of the blog is written in Rust, so you'll need the latest stable, as well as `cargo`. The blog also uses PostgreSQL as the database, so you'll need that installed too. Currently, this has only been tested on Linux. The following instructions will walk you through cloning the repository and building the server.

```
# Clone repository
git clone https://github.com/AbooMinister25/blog.git

# cd into the cloned repository
cd blog

# Set required environment variables (Database URL for migrations, and database URL for server, fill in your own credentials)
export DATABASE_URL="postgres://user:pass@localhost/my_db"
export ROCKET_DATABASES={blog_db={url="postgres://user:pass@localhost/my_db"}}

# Install diesel CLI (For database migrations)
cargo install diesel_cli

# Run migrations
diesel migrations run

# Build the server in release mode
cargo build --release

# Run the server
cargo run --release
```
