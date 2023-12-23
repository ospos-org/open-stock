docker compose up -d --scale open-stock=0 && DATABASE_URL=mysql://root:root@localhost:3306/stock \
ACCESS_ORIGIN=http://localhost:3001 \
ROCKET_SECRET_KEY=e5c63abf90fb076d7037a32d8dc2951c4b402e7357ca84b0da8e03595f84b30f \
DEMO=1 \
RUST_BACKTRACE=1 \
RUST_LOG=debug \
ROCKET_ADDRESS=0.0.0.0 \
ROCKET_PORT=8000 \
ROCKET_ENV=dev cargo run