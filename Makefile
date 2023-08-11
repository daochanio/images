build:
	cargo build --release

run:
	cargo run --release

docker-build:
	docker build -f ./Dockerfile  -t images .

docker-run:
	docker rm -f images && docker run -d -p 8081:8081 --env-file .env --name images images
